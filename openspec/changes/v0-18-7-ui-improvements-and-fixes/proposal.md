## Why

v0.18系の開発過程で見つかった複数のUI/UX上の課題、バグ、および機能不足を解消し、製品品質を向上させるため。特に、既存のメタ情報表示やサイドバーの挙動、タブ操作のUXが標準的なエディタ体験に比べて低いため、macOSネイティブに近い操作感と視認性を実現する。

## What Changes

- **検索結果のノイズ削減**: 検索結果に表示される `#[allow(...)]` などのアトリビュートをフィルタリングまたは整理し、ノイズを排除する。
- **メタ情報UIの刷新**: メタ情報表示（ファイル情報）をmacOSのFinder「情報を見る」ウィンドウのような、整理されたレイアウトに変更する。
- **ダイアグラム全画面表示の改善**: 図を全画面表示する際の背景を不透明化し、コンテンツに集中できるようにする。
- **サイドバー挙動の改善**:
  - ワークスペース/履歴パネルを開いてもエクスプローラーを閉じないように維持する。
  - サイドバーアイコン（検索、ワークスペース等）をクリックした際に、アイコンから「生えてくる」ようなアニメーションを伴うモーダルポップアップを表示する（popup APIは不使用）。
- **タブ操作のUX向上**:
  - タブ作成/リネーム時にEnterキーまたはBlurで確定・クローズする。
  - エクスプローラーのコンテキストメニューに、ディレクトリやファイルから直接タブグループを作成・追加する機能を追加する。
- **ヘルプ・ドキュメントの拡充**:
  - 「ようこそ」画面と「操作ガイド」をタブ形式で表示できるようにする。
  - 「操作ガイド」用のMarkdownコンテンツを新規作成する。
- **リンク自動検出のバグ修正**: `https://dummy.com` などの平文URLがリンクとして検出されない問題を修正する。
- **Windows配布とwinget再申請の整備**:
  - `v0.18.7` の Windows `.msi` / `.zip` 成果物、WiX installer UI、winget 申請導線を点検し、初回再申請で詰まりやすい dependency / submission flow の曖昧さを除去する。

## Capabilities

### New Capabilities

- `meta-info-finder-ui`: macOS Finder風の整理されたファイル情報表示機能。
- `sidebar-modal-popups`: サイドバーアイコンから発生するアニメーション付き非ブロッキングポップアップUI。
- `help-enrichment-tabs`: ようこそ画面と操作ガイドのタブ表示。
- `tab-group-explorer-integration`: エクスプローラーのコンテキストメニューからのタブグループ操作。

### Modified Capabilities

- `workspace-search`: 検索結果におけるアトリビュートノイズ（allow等）のフィルタリング。
- `diagram-fullscreen`: 全画面表示時のオーバーレイ不透明化による集中度向上。
- `markdown-rendering`: 平文URLの自動リンク検出精度の向上。
- `desktop-release-distribution`: Windows `.msi` / winget 再申請導線を `v0.18.7` の配布方針に合わせて整理。

## Impact

- `katana-ui`: 大幅なUI変更（サイドバー、モーダル、メタ情報、パネル挙動）。
- `katana-core`: 検索ロジックおよびMarkdownレンダラー（リンク検出）の修正。
- `katana-platform`: 設定（初回起動フラグなど）への影響。
- `scripts/release/*`, `.github/workflows/build-and-release.yml`, `crates/katana-ui/wix/main.wxs`: Windows 配布、installer、winget 再申請導線への影響。
