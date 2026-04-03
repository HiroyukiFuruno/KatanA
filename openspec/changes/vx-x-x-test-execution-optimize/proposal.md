## Why

`make check` が workspace 拡大とともに重くなり、日常の検証ループと pre-push の待ち時間が伸びている。
現状は `test-integration` が 1 つの test binary に UI integration を集約しており、不安定化要因を含む一部の test のために全体を直列寄りで扱う構造になっている。
さらに local verification では full coverage gate まで毎回同じ経路で考えやすく、変更のない層や重い fixture 群まで同じ温度感で回りやすい。

## What Changes

### ローカル検証の高速化

- 変更ファイルから impacted crate を判定し、逆依存 closure を含めた必要最小限の package のみ test / clippy を実行する
- 変更が Rust verification に無関係な場合は impacted test を skip できるようにする

### UI integration の分離

- `katana-ui` の integration test を parallel-safe / serial / fixture の 3 系統に分割する
- env var、固定 temp path、process-global state を使う test は serial bucket に隔離する
- fixture 系は重い bucket として独立させ、通常ローカル check から切り離せるようにする

### Coverage gate の維持

- coverage は full gate のまま維持する
- ただし parallel-safe 群と serial 群を別 invocation で集約できるようにし、local test split と両立させる

## Capabilities

### Modified Capabilities

- 開発用の検証ワークフロー
- `katana-ui` integration test 実行構成
- coverage 実行スクリプト

## Impact

- `Makefile`
- `scripts/coverage.sh`
- `scripts/` 配下の新しい impacted-runner
- `crates/katana-ui/tests/`
