# 最新書体・直接表示修正後の受入

## 対象と境界

公開作業HEAD 8f21230522f2daeac5bebf4df52bab1427af456f。製品実装はaab3ddf8と同一で、KatanA正式リリースではない。KDV=0.5.11/KRR=0.4.22/KUC=0.4.1/office2pdf=0.8.1のregistry graphを使用。

locked release worker再build1m19s、runner再build1m48sともexit0。worker SHA256は819016885f855e3e56f99ff07a6a3e9ba6d992824c8d0d4a7bcbd87153d4543a、runnerは1823e9b97ca5a602852afb1cfc175437bc6acf709a99946e7ae5c8121fdd707f。DEBUG=false、実release workerの絶対パス、逐次実行。CLI in-processでありpackaged main/all OS clean-machine受入ではない。

## 通常原本と反復

- 原本Office5第一回exit0: RSS105008→300320KiB、delta195312。第二回もexit0:104800→299584、delta194784。元196608上限は変更しない。第二回初回表示は順に0.091/0.101/3.836/2.153/1.570s。
- 実multi-format全38step exit0。実Notesクリック・active sheet・bounds・ページ移動・直接URLを維持。
- legal data-descriptor DOCX、実Sheetクリック0→1→0ともexit0。
- PPTX cold1+warm10混合開閉exit0: cold delta75648、warm8896KiB。
- XLSX同検査exit0: cold160400、warm5968KiB。
- 原本large PPTX同検査exit0: cold156048、warm-5504KiB。
- 3反復とも実HTML/document各11frame、終了surface/worker/session/workspace/artifact/cache/frame/texture全0。cold196608/warm65536とclose期限を変更しない。

ログはtmp/current-p2-{office-five,office-repro2,multi-format,legal-docx,sheet-input,pptx-cycle,xlsx-cycle,loom-cycle}-2026-10-05.log。実行batch14048/65725はいずれもexit0。

旧c626の通常原本RSS二回超過200832/200720とDEBUG診断188272は別証跡に保持する。今回二回PASSでも余裕は1296/1824KiBに留まり、RSS専用修正や原因確定の証明ではない。KDV#59を原因解決済みにはしない。

## canonical候補

実controls-on拒否と同frame controls-offの両capture契約はexit0。Typography PNG SHA256は394f828d1a89f89d0f15f6b4a62a6d704f4b275b9222f0e2b1225d2930ccfa3d、Diagramsはf77baa764224dcc653ed0288263dab16c292d8a353933d4278e6343c8ed67bf1。候補はtmp/current-p2-canonical-2026-10-05/{typography,diagrams}/のfull PNGとpreview-geometry JSON。

Typography実geometry: full2565×4774、crop2374×4450+(88,268)、pixels_per_point2、ui frame743、controls/selection/hover全0、pointer(1,1)でcontent外。既存referenceは上書きしない。これは独立visual/semantic/interaction/performance各95点評価ではない。KDV#58既存担当へ候補provenanceを渡す。

## 未完了

current HEADレビュー結果、manual-target方針P2、KRR#95修正版の公開registry採用と原本HTML正常終了、独立95点、packaged全OS/全5配布/公開・後処理は未完了。原本OfficeのPASSをこれらの代用にしない。
