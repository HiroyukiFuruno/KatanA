# Local Asset Preview

## Purpose

TBD (Displaying local image assets in markdown preview)

## Requirements

### Requirement: Markdown内ローカル画像の遅延読み込み表示

Markdown内の相対パスで参照されたローカル画像ファイルを、プレビューで表示する。初期表示を遅くしないため遅延読み込み（lazy load）とする。 The system SHALL conform.

#### Scenario: 相対パス画像の表示

- **WHEN** Markdownに `![alt](./images/photo.png)` のようなローカル相対パス画像参照がある
- **THEN** プレビューでその画像がインラインで表示される

#### Scenario: 遅延読み込みによる初期表示パフォーマンス維持

- **WHEN** 多数の画像を含むMarkdownファイルを開く
- **THEN** テキスト部分は即座に表示され、画像はビューポートに入った時点で読み込まれる

#### Scenario: 画像ファイルが存在しない場合

- **WHEN** 参照された相対パスの画像ファイルが存在しない
- **THEN** プレースホルダー（altテキスト + 不明アイコン）が表示される

#### Scenario: 対応画像フォーマット

- **WHEN** PNG, JPG, GIF, SVG のいずれかの画像が参照される
- **THEN** 全てのフォーマットが正しく表示される

### Requirement: Markdown内ローカル画像の非同期読込はファイル更新世代を尊重する

KatanA MUST Markdownから参照されたローカルPNGをUI要求呼出しの外でdecodeし、既存repaint経路で完了を反映する。ファイル更新世代が変わった場合、古い処理結果は既存の協調的read/decode checkpointで破棄し、後続する現世代の画像がready状態へ進めるようにする。親directoryや画像が欠落している場合、実ファイルを読み込めるretryまで失敗状態を可視に保つ。このrequirementは、任意のOS-blocked readやcodec operationを強制中断することを約束しない。

#### Scenario: PNGをUI要求呼出しを塞がずdecodeする

- **WHEN** Markdown sectionが有効なローカルPNGを要求する
- **THEN** UI call上で同期decodeせずに要求が戻る
- **AND THEN** 既存の非同期完了とrepaint経路を通じて画像を利用できる

#### Scenario: 更新前の画像処理が後続PNGを塞がない

- **WHEN** 古い世代のread中にローカル画像pathがinvalidateされ、そのreadが既存の協調的checkpointへ戻る
- **AND WHEN** 現世代の有効なPNGが書き込まれて要求される
- **THEN** 古い結果を現世代の結果としてpublishしない
- **AND THEN** 既存の協調的checkpointで古い処理を破棄した後、現世代PNGがready状態へ進める

#### Scenario: 欠落directoryと画像の作成後に復旧する

- **WHEN** Markdown画像が最初は欠落した親directoryまたはfileを参照する
- **AND WHEN** 後からdirectoryと有効なPNGが作成される
- **THEN** retryまで失敗状態を観測可能にする
- **AND THEN** retryが実PNGを読み込み表示する
