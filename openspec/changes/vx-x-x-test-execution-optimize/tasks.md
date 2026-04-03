# Tasks: Test Execution Optimization

- [x] 1. impacted package 判定 runner を追加する
  - `cargo metadata` から workspace package 依存を読み取り、reverse dependency closure を計算する
  - git diff から impacted package / verification scope を決める
  - impacted `clippy` と impacted test の実行経路を用意する

- [x] 2. `katana-ui` integration test を parallel / serial / fixture に分割する
  - `integration_tests.rs` の module 集約を廃止する
  - serial が必要な module を専用 binary に分離する
  - fixture 群を独立 binary として切り出す

- [x] 3. Makefile と coverage script を新構成へ合わせる
  - local default の `check` を impacted fast path にする
  - full gate を `check-full` として残す
  - coverage を multi-invocation merge に更新する

- [x] 4. 変更後のコマンドを検証する
  - impacted runner の summary / syntax check
  - `katana-ui` parallel / serial / fixture split の compile / list
  - `cargo llvm-cov --no-report` による serial target invocation
