## Context

katana は Rust（egui）製のデスクトップ Markdown エディタ。3 crate 構成（`katana-core`, `katana-platform`, `katana-ui`）。

現状の品質インフラ：
- **CI**: `cargo check` + `cargo test` + `cargo fmt --check` + `cargo clippy -- -D warnings` + CodeQL
- **pre-push hook**: `cargo-husky` 経由で fmt / clippy / test を実行
- **clippy.toml**: `too-many-lines-threshold = 30`, `cognitive-complexity-threshold = 10`
- **テスト**: 55 UT（全 pass）。ただしカバレッジ未計測。`settings.rs`, `shell.rs`, `main.rs` はテストゼロ
- **UI テスト**: 無し — 手動確認のみ

問題：
1. Clippy で `render_edge`（too_many_lines）と `border_point`（too_many_arguments）が error/warning 状態
2. `settings.rs`, `shell.rs`（ロジック部分）, `preview_pane.rs`（テスト関数なし）にテストが無い
3. UI の視覚的な正しさを自動検証する手段がない
4. カバレッジが計測されていないため、テストの抜け漏れが定量的に把握できない

## Goals / Non-Goals

**Goals:**

- Clippy warning/error をワークスペース全体でゼロにし、CI + pre-push で強制する
- テスト未実装モジュールにテストを追加し、ロジック部分のカバレッジ 100% を目指す
- `cargo-llvm-cov` による行カバレッジを CI に統合し、100% を必須ゲートとする
- egui の UI 検証を自動化するテスト基盤を構築する
- `egui_kittest` によるユーザーシナリオ E2E テストを導入する
- タスク完了定義を厳格化し、品質ゲートを明文化する

**Non-Goals:**

- Fuzzing / プロパティベーステストの導入
- パフォーマンスベンチマークの導入
- ブラウザベースの UI テスト（Playwright 等）— katana はネイティブアプリのため `egui_kittest` を使用する

## Decisions

### 1. Clippy 厳格化方針
**決定**: 各 crate のルートに `#![deny(clippy::too_many_lines, clippy::cognitive_complexity)]` を配置し、ワークスペース全体で warning をエラーとして扱う。

**理由**: `clippy.toml` による閾値設定は既にあるが、`deny` アトリビュートが一部ファイルにしかない。ワークスペースレベルの `Cargo.toml` に `[workspace.lints.clippy]` を追加するか、各 `lib.rs` / `main.rs` に `#![deny(warnings)]` を追加する。

**代替案**: `RUSTFLAGS="-D warnings"` を CI のみで設定 → ローカル開発時に見逃す可能性があるため不採用。

### 2. カバレッジツール選定
**決定**: `cargo-llvm-cov` を採用する。

**理由**:
- Rust 公式ツールチェーンと親和性が高い
- `--fail-under-lines` オプションで CI ゲートとして機能する
- GitHub Actions での導入実績が豊富
- ソースベースカバレッジ（source-based coverage）で精度が高い

**代替案**: `grcov` → LLVM ベースだが設定が複雑。`tarpaulin` → Linux 限定で macOS CI に不適合。

### 3. UI テスト戦略
**決定**: ロジック分離 + ユニットテスト（Phase 1）と `egui_kittest` による E2E テスト（Phase 2）の 2 段階で進める。

**理由**:

- `shell.rs` の 936 行はロジックと描画が密結合しており、まず分離が必要
- `egui_kittest` は AccessKit ベースの要素検索（`get_by_label` 等）とユーザー操作シミュレーション（`click()` 等）を提供する
- `snapshot` + `wgpu` feature でスナップショット比較テストも可能
- Harness を使えばヘッドレスで egui アプリ全体を駆動できる

**Phase 1**: `shell.rs` から純粋ロジック（`hash_str`, `relative_full_path`, action 処理, tab navigation logic）を抽出してテスト
**Phase 2**: `egui_kittest::Harness` を使った E2E テスト

- ユーザーシナリオ: ワークスペースを開く → ファイル選択 → プレビュー表示
- ウィジェット検証: ボタン・タブ・パネルの存在と操作可能性
- スナップショットテスト: UI レンダリング結果の回帰検知

### 4. カバレッジ閾値
**決定**: 行カバレッジ 100% を必須とする。一切の例外を認めない。

**理由**: カバレッジの妥協はテストの抜け漏れを許容することと同義であり、品質ゲートとして機能しなくなる。UI 描画コードについてもロジックを適切に分離すれば 100% は達成可能。

### 5. shell.rs のリファクタリング方針
**決定**: プレゼンテーションロジックと描画コードを分離する。

- `shell_logic.rs`: `hash_str`, `relative_full_path`, `process_action`, tab navigation ロジック等の純粋関数・メソッド
- `shell.rs`: egui 描画コードのみ（`shell_logic` を呼び出す）

**理由**: 描画コードは egui Context に依存するため UT が困難。ロジックだけを抽出すればテスト可能になる。

### 6. テストの src / tests 分離
**決定**: すべてのテストコードを `src/` 内の `#[cfg(test)] mod tests` から `tests/` ディレクトリに移行する。`src/` 内にテストコードを残すことを禁止する。

**理由**:

- テストとプロダクションコードの関心事を物理的に分離する
- テスト対象の公開 API が適切に設計されていることを強制する
- crate の外部利用者と同じ視点でテストできる

**「private だから tests/ からテストできない」は設計の問題であり、テスト分離を阻む理由にならない。** private な関数や構造体を tests/ ディレクトリからテストできないということは、モジュールの公開 API が不十分であることを意味する。正しい対処は：

1. テスト可能な粒度で `pub` な API を設計する
2. 内部実装の詳細ではなく、振る舞い（入力→出力）をテストする
3. 必要に応じて内部モジュールを `pub(crate)` で公開する

この原則に例外はない。「テストの都合で visibility を変えたくない」という主張は、テスト容易性を無視した設計を正当化する口実に過ぎない。

## Risks / Trade-offs

- **[リスク] shell.rs リファクタリングがリグレッションを引き起こす** → 既存テスト + 手動確認で検証。リファクタ前にスクリーンショットを撮影して比較。
- **[リスク] cargo-llvm-cov が macOS CI で遅い** → `--no-clean` オプションの活用、キャッシュの活用で緩和。
- **[前提] カバレッジ 100% を必須とする** → UI 描画コードのロジック分離を徹底し、テスト不可能なコードを最小化する。
- **[トレードオフ] egui スナップショットテストは Phase 2** → MVP では手動確認が残るが、ロジック分離により手動確認の範囲は大幅に縮小する。
