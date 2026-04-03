## Context

現在の KatanA は UI レイヤーと配布導線の両方で macOS 前提が残っている。

- `crates/katana-platform/src/os_theme.rs`
  - non-macOS では dark mode 検出が常に `None`
- `crates/katana-ui/src/main.rs`
  - 初期ロケール検出が macOS FFI のみ
- `crates/katana-platform/src/os_fonts.rs`
  - macOS の font directory しか探索しない
- `crates/katana-ui/src/native_menu.rs`
  - macOS では Objective-C native menu を使うが、non-macOS では `AppAction::None` の no-op
- `crates/katana-ui/src/shell_ui.rs`
  - `Cmd+P` など `egui::Modifiers::COMMAND` 固定の shortcut がある
- `crates/katana-core/src/update/version.rs`
  - release asset 名が `KatanA-macOS.zip` に固定
- `crates/katana-core/src/update/installer.rs`
  - `.app` bundle、`osascript`、`open`、`xattr` を前提とした macOS 専用 install path
- `scripts/package-mac.sh`、`scripts/dmg.sh`、`.github/workflows/release.yml`
  - macOS artifact のみを生成・公開
- `README.md`、`README.ja.md`、`docs/development-guide*.md`
  - macOS only と明記されている
- `openspec/specs/i18n/spec.md`
  - 辞書初期化の要件はあるが、初回起動時の OS locale 適用契約は未定義

一方で、次の基盤は既に cross-platform 寄りである。

- `eframe` による windowing / rendering
- `rfd` による file dialog
- `dirs::config_dir()` による config directory 解決
- `DiagramColorPreset` に Windows / Linux font candidate が既に含まれている

この change では、既存の cross-platform 基盤を活かしつつ、macOS 固定の integration と delivery を対応 OS 向けに整理する。

## Goals / Non-Goals

**Goals:**

- `v0.17.0` の正式サポート対象を macOS に加えて Windows x86_64 / Linux x86_64 へ広げる
- 対応 OS で workspace / editor / preview の主要フローを起動可能にする
- テーマ、ロケール、ショートカット、メニュー、フォントの OS 依存点を明示的な platform contract へ寄せる
- update dialog / release asset / docs / CI が macOS only 前提で壊れないようにする
- macOS を主開発環境とする maintainer でも Windows / Linux の完了判定を再現できる検証レーンを用意する
- 他の AI エージェントが会話履歴なしで読んでも、runtime / release / docs のどこをどう触るか分かる状態にする

**Non-Goals:**

- Windows ARM64 / Linux ARM64 対応
- Windows MSI、winget、Linux AppImage / deb / rpm の native installer
- Windows / Linux 向け in-app auto-install
- OS ごとの UI 最適化や見た目調整の深掘り
- local LLM 機能の追加

## Decisions

### 1. `v0.17.0` の support matrix は `macOS + Windows x86_64 + Linux x86_64` に限定する

`Windows / Linux 対応` を曖昧にすると artifact 名、CI target、更新導線が確定しない。そこで `v0.17.0` は次の matrix に固定する。

- macOS: 既存サポートを維持
- Windows: `x86_64-pc-windows-msvc`
- Linux: `x86_64-unknown-linux-gnu`

- 採用理由:
  - CI と release artifact 名を具体化できる
  - x86_64 を先に固める方が Windows / Linux 初回対応として現実的
- 代替案:
  - ARM を同時に含める: 検証範囲が広がりすぎるため不採用

### 2. OS 依存点は `katana-platform` の platform facade に集約する

UI / core から生の `#[cfg(target_os = "macos")]` を散らしたまま広げると、Windows / Linux 対応で条件分岐が破綻する。そこで `katana-platform` に次の責務を持つ facade を置く。

- current platform の識別
- primary modifier (`Command` / `Ctrl`) 解決
- system theme / locale 検出
- native menu の有無
- update install mode (`AutoInstall` / `ManualDownload`)

UI / core はこの contract を呼び、OS 固有の FFI や dependency は facade の内側へ閉じ込める。

- 採用理由:
  - 実装者が OS 依存点を追いやすい
  - platform 拡張時の diff を限定できる
- 代替案:
  - 各 callsite で `cfg` を足す: 変更が散り、検証漏れを誘発するため不採用

### 3. macOS native menu は維持し、Windows / Linux は in-app command surface を持つ

macOS では既存の native menu を残す。一方で Windows / Linux では no-op にせず、top bar または同等の in-app command surface から `OpenWorkspace`、`SaveDocument`、`ToggleSettings`、`ShowReleaseNotes`、`CheckForUpdates`、language change へ到達できるようにする。

- 採用理由:
  - non-macOS で command が消える問題を解消できる
  - `AppAction` をそのまま再利用できる
- 代替案:
  - Windows / Linux も native menu を同時に実装する: 初回対応として過剰なため不採用

### 4. shortcut は `primary modifier` abstraction へ置き換える

`Cmd+P` のような macOS 固定 shortcut は、Windows / Linux では `Ctrl+P` に読み替えられなければならない。`egui::Modifiers::COMMAND` を直書きせず、platform facade から primary modifier を取得して shortcut を定義する。

- 採用理由:
  - 検索やタブ復元が各 OS の慣習に沿う
  - 今後 shortcut が増えても同じ abstraction を使える
- 代替案:
  - 各 shortcut に個別分岐を書く: 重複が増えるため不採用

### 5. theme と locale は「検出できれば追従、できなければ既定値」方針にする

theme / locale の OS integration は quality を上げるが、検出 failure で起動不能になるべきではない。そこで方針を次に固定する。

- dark/light:
  - 検出成功時は初回起動で OS mode に追従
  - 検出不能時は `KatanaDark`
- language:
  - 検出成功時は初回起動で system locale を適用
  - 検出不能時は `en`

必要なら軽量 dependency を追加してもよいが、その導入先は `katana-platform` に限定する。

- 採用理由:
  - UX を改善しつつ failure contract を単純に保てる
- 代替案:
  - 検出不可ならエラーにする: desktop app として不自然なため不採用

この方針により、初回起動時の locale 適用は `i18n` capability の requirement として明示し、辞書構造の話と混ざらないようにする。

### 6. font / emoji は graceful degradation を前提にする

`DiagramColorPreset` には Windows / Linux 向け candidate があるが、`OsFontScanner` は macOS 固定で、`font_loader` は emoji candidates を未使用のままにしている。`v0.17.0` では次を満たす。

- Windows / Linux の標準 font directory を探索対象に含める
- default candidate から最初に見つかった font を editor / preview に適用する
- emoji font が見つからない場合でも app は crash せず、既定 renderer にフォールバックする
- Apple Color Emoji 専用 raster path は macOS に残し、non-macOS は degrade gracefully で扱う

- 採用理由:
  - 起動品質と文字表示品質を両立できる
- 代替案:
  - macOS と同じ emoji pipeline を各 OS で新規実装する: `v0.17.0` では重すぎるため不採用

### 7. update は `asset selection` と `install mode` を分離する

現在の update flow は「release tag を見る」「macOS zip を取る」「.app を swap する」が密結合している。`v0.17.0` ではこれを 2 段に分ける。

- asset selection:
  - current platform / arch に一致する release asset を選ぶ
- install mode:
  - macOS: 既存の auto-install / relaunch
  - Windows / Linux: manual download / release page open

これにより Windows / Linux で update button が壊れた install path へ進むことを防ぐ。

- 採用理由:
  - cross-platform 対応時に update UX を壊さず拡張できる
- 代替案:
  - Windows / Linux も auto-install まで同時実装する: installer 差分が大きいため不採用

### 8. release artifact は portable 形式を採用する

`v0.17.0` では installer の整備よりも、まず配布可能な artifact を揃えることを優先する。release artifact は次に固定する。

- macOS: `.dmg` + `.zip`
- Windows: `.zip`
- Linux: `.tar.gz`

artifact 名も platform-aware に固定する。

- `KatanA-Desktop-<version>.dmg`
- `KatanA-macOS.zip`
- `KatanA-windows-x86_64.zip`
- `KatanA-linux-x86_64.tar.gz`

- 採用理由:
  - release workflow と updater asset selection を単純化できる
  - Windows / Linux の初回対応として実装可能性が高い
- 代替案:
  - MSI / AppImage / deb を最初から用意する: 配布形式ごとの保守コストが高いため不採用

### 9. CI は `macOS + Windows + Ubuntu` matrix に広げる

runtime support を担保するには、`cargo check` や `cargo test` の実行面も対応 OS を含む必要がある。CI は少なくとも次を持つ。

- macOS: 既存 test / coverage
- Windows: build / smoke test
- Ubuntu: build / smoke test / lint

Windows / Ubuntu の CI job は **自動トリガーしない**。macOS と同様に `workflow_dispatch`（手動実行）を基本とし、必要なタイミングで maintainer が明示的に実行する。push / pull_request のたびに 3 OS matrix を回すとビルド時間とコストが高頻度で発生するため、これを避ける。

platform-specific test expectation は `cfg` で吸収し、macOS 専用 FFI を non-macOS で誤ってリンクしないことを保証する。

- 採用理由:
  - 対応したつもりで非macOSを壊す事故を減らせる
  - 手動トリガーにより CI コストを制御できる
- 代替案:
  - ローカル確認だけに頼る: 継続的な保証にならないため不採用
  - push / PR ごとに全 OS matrix を自動実行する: ビルド時間とコストが高頻度で発生するため不採用

### 10. Windows / Linux の検証レーンは `target OS CI` と `承認済み manual verification` の二段で固定する

`cargo check --target` は build gate にしかならず、GUI アプリである KatanA の runtime support をそれだけで証明することはできない。さらに主開発環境が macOS の場合、Windows / Linux を「手元でたまたま動いた」で済ませると再現性がない。そこで `v0.17.0` の検証レーンを次に固定する。

- build / automated smoke gate:
  - GitHub Actions の Windows / Ubuntu runner で build と automated smoke verification を実行する
- evidence review gate:
  - Windows / Ubuntu job は log と生成 artifact を保持し、macOS 上の maintainer が結果をレビューできるようにする
- runtime manual gate:
  - CI だけで GUI runtime を証明しきれない観点は、target OS 上の VM、remote machine、physical machine のいずれかで manual verification を実施する
- required evidence:
  - startup 完了
  - workspace を開けること
  - Markdown を選択・編集できること
  - preview が表示されること
  - update / release 導線に手を入れた場合は、その表示と遷移結果

manual verification の記録方法は screenshot または log を最低限とし、Windows / Linux ごとに同じ checklist で判定できるようにする。

- 採用理由:
  - macOS しか手元にない実装者でも、Windows / Linux の完了判定を再現できる
  - cross-compilation と author assertion だけで正式サポート扱いになる事故を防げる
- 代替案:
  - `cargo check --target` と CI build 成功だけを根拠にする: runtime evidence が欠けるため不採用
  - 実機確認だけを必須にする: 継続性と再現性が弱く、handoff に不向きなため不採用

詳細な検証手順、AI エージェント向け判断ルール、リリース判定チェックリストは [`cross-platform-verification.md`](./cross-platform-verification.md) を参照。

### 11. `正式サポート` の完了条件を CI / artifact / smoke verification の 3 点で固定する

`Windows / Linux 対応` は「コンパイルが通る」だけでは曖昧で、実装者ごとに完了条件が割れる。そこで `v0.17.0` での正式サポートは次をすべて満たす状態と定義する。

- CI 上で target OS の build または automated smoke verification が通る
- GitHub Releases に target OS 向け artifact が載る
- Windows / Linux ごとに runtime smoke checklist と required evidence が文書化され、結果をレビューできる

- 採用理由:
  - handoff 時の Done 判定がぶれない
  - runtime / release / docs の 3 面が揃って初めて「使える対応」になる
- 代替案:
  - build success のみを基準にする: 実利用導線が未確認のままになるため不採用

## Risks / Trade-offs

- **[Risk] Windows / Linux で native menu がない分、macOS より体験が一段落ちる**  
  -> Mitigation: `AppAction` 等価の in-app command surface と shortcut を先に保証する
- **[Risk] theme / locale 検出の dependency 導入が platform crate を肥大化させる**  
  -> Mitigation: facade の裏側だけで使い、UI / core へ依存を漏らさない
- **[Risk] Linux の font / emoji 品質差が distro ごとにばらつく**  
  -> Mitigation: graceful degradation を requirement にし、missing font でも crash しない契約にする
- **[Risk] updater を Windows / Linux で manual download にすると機能差が残る**  
  -> Mitigation: `v0.17.0` では broken auto-install を避けることを優先し、native installer は後続 change に分離する
- **[Risk] release workflow の matrix 化で build 時間が伸びる**  
  -> Mitigation: coverage は macOS 維持、Windows / Linux は smoke/build 中心に留める
- **[Risk] GUI runtime は CI だけでは証明しきれない**  
  -> Mitigation: target OS CI を一次 gate にし、残りは VM / remote host / physical machine の manual verification checklist と evidence で補う

## Migration Plan

1. 先に platform facade と target-specific build breakage を解消し、`cargo check` が macOS / Windows / Linux で通る状態を作る
2. その後に menu / shortcut / theme / locale / font の runtime contract を揃える
3. `ci.yml` と `release.yml` に Windows / Ubuntu の build / smoke / evidence review 導線を追加する
4. target OS で使う manual verification checklist と required evidence を定義する
5. update dialog と release asset naming を platform-aware に切り替える
6. release workflow と Makefile に Windows / Linux artifact を追加する
7. README / development guide / badge / support matrix を更新する

Rollback は OS ごとに独立できるようにし、Windows / Linux artifact 追加後に問題が出た場合は release workflow から該当 artifact を外し、runtime support は feature-guard なしで残す方針とする。

## Open Questions

- Linux の将来 artifact は `tar.gz` のまま維持するか、後続で AppImage / deb を追加するか
- Windows の将来配布を portable `.zip` のままにするか、後続で MSI / winget を追加するか
