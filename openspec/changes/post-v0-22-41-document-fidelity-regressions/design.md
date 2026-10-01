## Context

v0.22.41 は KatanA を薄いホスト、KDV を PDF/Office セッション、KRR を HTML ブラウザー意味論の owner として公開したが、実ファイルではホスト配置と上流機能の両方に回帰が残った。確認済みの KatanA 不具合は、XLSX の表面が残り高さを全消費した後で Sheet タブを描くためタブが画面外へ出ること、起動時に約 183 MiB の `Apple Color Emoji.ttc` を複数回読み込むことで空 workspace の footprint が約 735 MiB になることである。指定 HTML は `position: sticky; top: 0; height: 100vh` を使うが、KRR 0.4.17 の `CssPosition` は static/relative/absolute/fixed だけで sticky を受理しない。

Office の ZIP、フィルター、表示品質、変換時間は KDV/office2pdf が所有する。KatanA に OOXML parser、表計算 engine、Office layout、HTML/CSS の例外処理を追加してはならない。

## Goals / Non-Goals

**Goals:**

- 指摘された9項目を独立した受入項目として追跡し、未検証項目を完了扱いしない。
- KatanA 所有のメニュー活性条件、Sheet タブ配置、フォント初期化、診断・ホストハーネスを修正する。
- KRR の sticky layout と HTML互換性、KDV/office2pdf の ZIP、フィルター、Office品質・時間を owner 境界で修正し、公開版を KatanA から検証する。
- 空 workspace と文書反復切替の両方でメモリ回帰を検知する。
- Office/PDF の表示対象追加で増えたツリーノード数に比例してエクスプローラーの各フレームが劣化しないようにする。
- 配布アセットが宣言した CPU/OS 組合せで、main binary と Office sidecar を含めて起動可能であることを保証する。

**Non-Goals:**

- Chromium/WebView を導入しない。
- KatanA に独自の Office/HTML renderer や XLSX filter engine を実装しない。
- アクティブコンテンツを実行したり、安全上限を無効化して ZIP を通したりしない。
- 見た目だけのSheetタブやフィルターを実装しない。
- 有償の Apple Developer Program を前提とする Developer ID 署名・notarization を導入しない。

## Decisions

### 1. 適用可能性を文書種別ごとの共通契約にする

目次だけの個別判定を増やさず、アクティブ文書から目次、export/download、story/slideshow、tools/view の利用可否を算出する。サイドバーの `add_enabled_ui` と action dispatch の双方が同じ判定を使い、hover だけで非対応 panel が開く経路も閉じる。

### 2. Sheet タブの高さを表面より先に予約する

XLSX の時だけ下端 panel を先に確保し、その残り領域へ grid surface を割り当てる。単にタブ描画順を前へ移して上端に表示する案は Excel の操作モデルを満たさないため採用しない。

### 3. sticky は KRR の layout primitive として実装する

`position: sticky` を fixed へ読み替える例外処理は、親の包含領域下端を越えて固定されるため採用しない。KRR は通常 flow の位置を保持し、viewport scroll と inset に応じて表示位置を clamp し、包含 block の末尾で解除する。指定 HTML の `#s15` へ移動した後も左目次が viewport 上端に残る完全フレームを回帰証拠にする。

### 4. XLSX filter は KDV の型付き操作と状態として追加する

KDV が OOXML の AutoFilter/Table filter metadata、候補値、適用状態、行可視性を所有し、KatanA は surface の filter affordance と型付き command/event だけを投影する。KatanA が表示文字列から列値を再解析する案は採用しない。

### 5. OOXML ZIP と表示品質は共通変換入口で直す

DOCX/XLSX/PPTX の data descriptor、local header と central directory の合法な差分を実フィクスチャで分類する。安全上限を維持したまま標準準拠 archive を受理し、不正・過大・暗号化 entry は型付き失敗にする。Office fidelity は unsupported feature の個別握り潰しではなく、office2pdf/KDV の共通 artifact とフォント・配置・style mapping を改善する。

### 6. 起動時フォントを一度だけ所有する

egui の既定フォントには Noto Emoji と emoji icon font が含まれるため、約 183 MiB の Apple Color Emoji を通常起動で追加読込しない。初期 `set_fonts` の直後に同じ family を再適用する経路も止める。CJK proportional/monospace と既定 emoji の描画契約は維持し、任意の custom font 選択だけを明示的な再構築理由にする。

### 7. 診断は `DEBUG=true` で構造化出力する

通常リリースでは追加出力を行わない。`DEBUG=true` の時だけ source read、ZIP preflight、Office conversion、KDV open/frame、KRR parse/style/script/layout/paint、texture upload、session close の開始・終了・経過時間・主要 byte 数・generation を共通 helper から出力する。

### 8. メモリゲートは決定論的契約と実プロセス観測を併用する

単発RSS値だけでは macOS allocator/GPU の残留をリークと誤判定するため、通常起動で読み込む owned font bytes、KDV page cache bytes、保持 frame/texture 数、live worker/session 数を決定論的に検査する。加えて packaged macOS app の空 workspace 定常 footprint と文書10回切替後の増分を採取し、基準超過を失敗にする。

### 9. エクスプローラーは不変ツリーを借用し、表示行だけを処理する

現行の `ExplorerContent::show_active_workspace` は毎フレーム `ws.tree.clone()` で再帰ツリー全体を複製する。Office/PDF を標準表示拡張子へ加えるとノード数が増え、文書 viewer が非アクティブでもフレーム時間と allocator 負荷が増える。ツリー revision が変わらないフレームでは不変データを借用し、検索 projection は revision 単位でキャッシュし、描画は viewport と展開状態に含まれる行へ限定する。

### 10. 配布契約は存在確認ではなく実行可能性を検査する

v0.22.41 の汎用名 `KatanA-macOS.zip` 内の main binary と `kdv-office-worker` はともに arm64 thin binary だった。現行 asset contract はファイル名と同梱だけを確認するため、この不一致を検知できない。各アセットは対応 CPU/最低 OS を明示し、main/sidecar の architecture 一致と Windows/Linux を含む clean-machine 起動 smoke test を公開前ゲートにする。macOS は現在の無償配布方針どおり ad-hoc 署名を維持し、Apple Developer Program の資格情報を CI の必須条件にしない。Gatekeeper の手動許可が必要になり得る制約は配布案内へ正確に残す。

## Risks / Trade-offs

- sticky は包含 block と overflow の組合せが複雑 → 指定実ファイルに加え、上端・中間・親下端の小さいKRR layout testを追加する。
- Apple Color Emoji を外すと一部カラー絵文字の外観が変わる → 既定 Noto Emoji の主要絵文字描画を検証し、巨大フォント常駐より明示的なfallbackを優先する。
- Office品質の「本家に近い」が主観化する → 同一viewportの参照画像、文字・セル・図形の位置、欠落要素数を fixture ごとに数値化する。
- RSS はOSとGPUで揺れる → 絶対値だけでなく、同一実行内baselineからの増分と内部資源カウンタを主判定にする。
- sibling 修正の公開順でKatanA検証が待つ → KRR/office2pdf/KDVを先に公開し、KatanAはregistry exact dependencyだけで最終検証する。
- ツリー借用への変更で UI の可変状態と borrow が衝突する → workspace tree と展開・検索・action 状態を分離し、全ツリー clone へ戻さず revision projection を使う。
- 対応 architecture を曖昧にすると一部端末だけ起動不能になる → asset 名・release notes・機械可読 manifest を一致させ、宣言対象ごとに実バイナリを起動する。
- ad-hoc 署名では Gatekeeper の手動許可が必要になり得る → 有償署名を暗黙の要件へ追加せず、配布案内と Homebrew の quarantine 対応を維持する。

## Migration Plan

1. KatanA 所有の配置・活性条件・フォント初期化・診断ハーネスを修正する。
2. KRR sticky/HTML差分、office2pdf/KDV ZIP・filter・fidelity・cache/latencyを各ownerで修正する。
3. 上流を順番に公開し、KatanAをregistry依存へ更新する。
4. 指定HTMLと全Officeフィクスチャ、空workspace、反復切替をpackaged appで検証する。
5. 回帰時は下流公開を進めず、直前の公開依存へ戻せる状態を保つ。

## Open Questions

- 現在のフォルダにはZIPエラーが発生するDOCX実体がないため、KatanAの診断ログから対象entryを採取するか、該当DOCXをフィクスチャへ追加する必要がある。
- Office fidelity の参照を Microsoft Office、LibreOffice、office2pdf本家のどれに固定するかは、既存の本家比較手順を確認して確定する。
- macOS Intel を継続対応する場合は universal binary と architecture 別 asset のどちらを採用するか、公開済みの対応方針と利用端末を確認して確定する。
