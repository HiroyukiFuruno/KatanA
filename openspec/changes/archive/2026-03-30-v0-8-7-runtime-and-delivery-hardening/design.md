## Context

今回の change は、`v0-8-4-rust-code-refactoring` のような構造分割ではなく、KatanA の「壊れにくさ」と「配布の一貫性」を改善するための設計です。

現状の主な課題は以下です。

- `settings_window.rs` や `shell.rs` では `let _ = settings.save();` が多用され、保存失敗が UI から見えない
- `JsonFileRepository::save()` は `std::fs::write()` の直接上書きであり、途中失敗時に部分書き込みや空ファイル化のリスクがある
- `prepare_update()` 後の relaunch script は `rm -rf "{target}"` → `mv "{extracted}" "{target}"` という破壊的な置換であり、途中失敗時の rollback がない
- `scripts/release/release.sh` の preflight と `.github/workflows/release.yml` の preflight が揃っておらず、同じ version でもローカルと CI で異なる条件で release できてしまう

## Goals / Non-Goals

**Goals:**

- 設定永続化を atomic write 化し、破損設定の退避と保存失敗の可視化を導入する
- in-app update の置換手順を staged swap + rollback に変更し、既存アプリの可用性を優先する
- ローカル release と GitHub Actions release の preflight と artifact 契約を単一化する
- 失敗系を含む自動テスト、smoke test、CI 検証を追加する

**Non-Goals:**

- auto-update 機能そのものの UI 再設計
- Windows / Linux 向け配布フローの新規実装
- `settings_window.rs` や `shell.rs` の大規模な責務分割
- notarization や署名戦略全体の刷新

## Decisions

### 1. 設定保存は temp file + rename を基本とする

`JsonFileRepository::save()` は、対象ファイルと同一ディレクトリに一時ファイルを書き出してから rename で差し替える方式にする。これにより、保存失敗時に「壊れた新ファイルだけが残る」状態を避ける。

- 採用理由:
  - 既存の `SettingsRepository` API を保ったまま実装できる
  - macOS 上のローカル設定ファイルには十分現実的で、既存コードへの変更面積も小さい
- 代替案:
  - `std::fs::write()` 継続 + ログのみ: 失敗時のデータ保全が弱く、今回の目的に合わない
  - SQLite 等への移行: スコープ過大

### 2. 設定ロード/保存失敗は UI から recoverable error として扱う

設定ファイルの破損や保存失敗は、単なる `tracing::warn!` ではなく、ユーザーに再試行可能な失敗として伝える。UI 層では `let _ = settings.save();` を廃止し、共通の通知ヘルパー経由で status message と log を揃える。

- 採用理由:
  - 現状の silent failure を止められる
  - `shell.rs` と `settings_window.rs` に散っている保存処理の UX を揃えられる
- 代替案:
  - 各 call site で個別に文言を組み立てる: 表示とログの一貫性が壊れる

### 3. update relaunch は target 隣接の staged swap に変える

更新アーカイブから展開した `.app` は、そのまま既存 app を消して差し替えるのではなく、target と同じ親ディレクトリ配下に `*.new` として staging してから swap する。swap 中に失敗した場合は `*.backup` から復元する。

- 採用理由:
  - 現在の `rm -rf` ベースの破壊的置換を避けられる
  - temp directory と `/Applications` が別パスであっても、最終 swap を target 近傍で完結できる
- 代替案:
  - 現状維持: 途中失敗時にアプリが消える可能性を残す
  - 完全な installer 導入: スコープ過大

### 4. release preflight は単一 entrypoint に集約する

release に必要な version/changelog/OpenSpec/artifact 契約の検証は、`scripts/release/preflight.sh` のような単一入口へ寄せる。`make release` と GitHub Actions の両方が同じ入口を呼ぶ構成にする。

- 採用理由:
  - ローカルと CI のズレを設計上なくせる
  - smoke test / dry-run の入口としても再利用できる
- 代替案:
  - shell script と workflow YAML に同じ条件を重複記述する: 将来的に再びズレる

### 5. release helper scripts には publish しない検証モードを持たせる

`publish-github.sh` や `update-homebrew.sh` 自体は本番 publish を行うが、CI ではそれらを直接叩かずに preflight/artifact verification のみを dry-run で検証できる構成にする。

- 採用理由:
  - secrets や外部 publish を使わずに release 導線の回帰を検出できる
  - release.yml の安全性を保ったまま検証レイヤーを増やせる
- 代替案:
  - 本番 publish でしか確認しない: 回帰発見が遅い

## Risks / Trade-offs

- [Risk] backup / broken settings ファイルが増える
  - Mitigation: 命名規則を統一し、最新の失敗理由を log/status に残す

- [Risk] release preflight の厳格化で既存の手動運用が一時的に失敗しやすくなる
  - Mitigation: エラーメッセージで不足条件を明示し、dry-run で事前確認できるようにする

- [Risk] staged swap script の分岐が増えてメンテナンスコストが上がる
  - Mitigation: script 生成テストと failure path テストを追加し、ログ出力を標準化する

## Migration Plan

1. settings 保存処理を atomic write 化し、破損ファイル退避と UI 通知を追加する
2. update relaunch script を staged swap + rollback 方式へ切り替え、既存の destructive path を廃止する
3. release preflight を共通 script に抽出し、`make release` と `.github/workflows/release.yml` の両方から呼ぶ
4. release helper smoke test を CI に追加してから、次の `v0.8.6` release で新フローを使う

## Open Questions

- update ZIP に checksum/署名検証まで含めるか、今回はアーカイブ構造検証までに留めるか
- cache/document save にも同じ atomic write helper をこの change で適用するか、follow-up に分けるか
