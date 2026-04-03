## Context

現在の verification 経路は `Makefile` で定義されており、`check` は format / clippy / UI integration / coverage をまとめて扱う。
`katana-ui` の integration test は `crates/katana-ui/tests/integration_tests.rs` に module 集約されているため、同一 process の test thread 上で実行される。
この構造だと、`MERMAID_MMDC` / `PLANTUML_JAR` のような環境変数を書き換える test や、固定 temp path を使う test、process-global な i18n 状態へ触る test を個別に隔離できない。

workspace 依存は概ね以下:

- `katana-core -> katana-platform -> katana-ui`
- `katana-linter -> katana-ui`

したがって差分実行では「変更 crate のみ」ではなく「変更 crate + 逆依存 closure」を対象にする必要がある。

## Goals / Non-Goals

**Goals**

- local / pre-push の検証時間を短縮する
- serial が必要な test だけを隔離し、それ以外は並列実行できるようにする
- coverage の full gate を維持する
- 既存の workspace 依存に対して unsafe な skip をしない

**Non-Goals**

- CI 全体を nextest ベースへ置き換えること
- coverage を差分 gate に変更すること
- 既存 test の大規模な書き換えで完全 parallel-safe 化を目指すこと

## Decisions

### 1. impacted 判定は source hash ではなく git diff + reverse dependency closure を使う

初期実装では独自 fingerprint DB を持たない。
理由は、workspace 依存が小さく、`cargo metadata` から逆依存 closure を計算すれば十分安全だから。

判定ルール:

- `crates/<pkg>/...` 変更: 該当 package を起点に reverse dependency closure を追加
- `Cargo.toml`, `Cargo.lock`, verification script, workflow 定義変更: workspace 全体を impacted
- `assets/**` 変更: `katana-ui` を impacted
- `docs`, `openspec` など verification に無関係な変更のみ: impacted test を skip 可

### 2. `katana-ui` integration は 3 binary に分割する

- `ui_integration_parallel`
- `ui_integration_serial`
- `ui_integration_fixture`

`integration_tests.rs` の module 集約はやめ、top-level test target から既存 module file を読み込む。
これにより process 分離が効き、serial が必要な test を一部 binary に閉じ込められる。

### 3. serial bucket は `--test-threads=1`、parallel bucket は通常並列を使う

serial bucket には以下を含める。

- env var を変更する diagram test
- 固定 temp path を多用する大規模 integration test
- 固定 temp path と process-global state を併用する tree layout test

fixture bucket は重さで分離し、通常 local check では回さない。

### 4. local の既定 check は impacted fast path に切り替える

`check` は日常用途として以下を実行する。

- `fmt-check`
- impacted `clippy`
- impacted test
- `katana-ui` fixture は除外
- coverage は実行しない

full gate は `check-full` として残す。

### 5. coverage は multi-invocation merge に切り替える

`cargo llvm-cov --no-report` を複数回使い、

- workspace lib/bin
- workspace integration (parallel-safe)
- `katana-ui` serial integration

を別 invocation で計測し、最後に `cargo llvm-cov report` で gate 判定する。
これにより test split 後も coverage 意味論を維持できる。

## Risks / Trade-offs

- impacted 判定は git diff ベースなので、untracked file や verification 対象外の資産変更をどう扱うかのルールが必要
- `integration.rs` のような巨大 file は serial bucket に残るため、ここ自体の高速化は限定的
- `check` の意味を local fast path に寄せるため、full gate は `check-full` を明示的に使う運用へ変わる
