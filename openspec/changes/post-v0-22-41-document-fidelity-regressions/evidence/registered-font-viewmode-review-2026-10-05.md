# 登録済みUnicode書体と直接表示操作の追加レビュー

PR346のc62644f2 current review5406778273に対するP2二件。Draftのまま修正・再検証を継続する。

- 4178163032 / PRRT_kwDORm09y86oztRA: document leaseで解決できない設定済みカスタム書体の登録名Écoleと要求écoleをfallbackのASCII比較が不一致にする。
- 4178163035 / PRRT_kwDORm09y86oztRC: toggle経路にだけavailabilityがあり、直接SetViewMode(Split/CodeOnly)がHTML/Officeの表示モードを変更する。

## 実回帰と最小修正

FontDefinitionsに既存の実フォントchainをName("École")で登録し、leaseなしfont選択を検査。旧コードはProportionalを返し1件RED（tmp/unicode-registered-fallback-lib-red-2026-10-05.log）。Unicode lowercaseを既存resolver/consumerと統一し、実font_id callerの同回帰はGREEN。

実AppState/dispatch_secondaryで直接actionを実行。旧コードではHTMLがPreviewOnlyからSplitへ変わりRED、Markdownの全mode保持は元からPASS（tmp/direct-viewmode-lib-red-2026-10-05.log）。PreviewOnly以外に既存Tools availabilityを適用。HTML/HTM/DOCX/XLSX/PPTXのSplit/CodeOnly拒否、CodeOnlyからPreviewOnly復帰許可、Markdown全mode保持を検査。追加2件GREEN。

追加focused3件、公式fmt-check、AST23件は成功。既存with-office-test-worker入口で実開発workerをbuildした後、全UI --lib検査成功（RTK集計1048pass/既存ignore2、child検査を含む）。tmp/current-p2-ui-worker-all-retry-2026-10-05.log。supply-chainもadvisories/bans/licenses/sources全4分類成功。impacted strict ClippyとUI all-target strict Clippyは共にexit0。自己レビュー後、書体662c9059・直接action a7555b9dへ関心事別に正式commit。全coverage、通常push、各reply/resolveとfresh current review、変更後fresh原本/配布受入は残す。

初回の全workspace build/worker link/Clippyはdisk fullで失敗し、製品REDや成功検証に流用しない。最初の全UI1045pass/1failは、整理後のprofile隣接workerが未buildだったため。製品テストやworker探索を変更せず、既存入口で再buildして全件を実行した。

所有確認済み・終了済みの当WT生成物のみ公式package/profile限定dry-run→cleanで整理（native UI3.1GiB、coverage core1.8GiB、coverage KRR依存4.5GiB）。すべて再生成可能。旧全coverage raw JSONとソース・原本・release binaryを保全し、sibling repoは変更していない。差分cacheだけCARGO_INCREMENTAL=0、warning/profile/テスト・coverage・受入基準は不変。

原本Office5のRSS超過二標本（200832/200720 >196608KiB）は別の未解決事項。今回の回帰修正をRSS修正やリリース完了として扱わない。

## 公開ブランチとレビュー照合

aab3ddf8759c76909cdcdbf30aa965e2b2c81dffまで通常push exit0、GitHub HEAD一致・upstream0/0。指摘へのreply4178280760/4178280877後、各threadをresolve。mainが全reviewThreads/commentsを再取得し両resolved・全page取得済み・既存manual-target方針P2一件だけ未解決と確認した（tmp/current-p2-review-all-2026-10-05.json）。新HEADへcurrent review5981677904を依頼、Draftを維持。

全coverageは通常push後に別途実行中。hookはjust checkであり、coverageを実行したと推測しない。native検査終了後の署名clean checkpointとnative Cargoロック解放、Linuxの専用target volumeを確認し、native UI/core生成物2562files/6.2GiBだけ公式clean（再生成可能）して空き10GiBを確保した。source・原本・release binary・siblingは不変。

その後の公式just coverageはexit0。UI1046/既存ignore2、main17、実export13（259.49s）、parallel143/既存ignore2、serial18が成功。meaningful未実行0、strict document surface100%/未実行0、raw JSON15443630bytesをtmp/current-p2-full-coverage-2026-10-05.jsonへ保全した。元coverage regex・閾値・fixtureの扱いは不変。新sourceのfresh release worker/runner・原本/配布・95点・上流公開採用・KatanA公開の残DoDは完了にしない。
