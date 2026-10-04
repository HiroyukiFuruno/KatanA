# 追加レビュー修正後の公開依存・実ファイル再受入

## 実行identity

- KatanA source修正435ac5b2、文書同期HEAD761439e1a69ed7a907cc4501a9d7b0b6cc663323。最新HEAD通常push exit0、PR346はDraft、review5980503748を依頼。
- 公開registry固定KDV0.5.11/KRR0.4.22/KUC0.4.1/office2pdf0.8.1。旧runnerを流用せずlocked releaseを再build、30m48s/exit0。
- runner SHA256 `27683e34cec556ac0cbbd8dfe799b78ef73c0096c15960ffc63c4f6d2078919c`。
- worker SHA256 `819016885f855e3e56f99ff07a6a3e9ba6d992824c8d0d4a7bcbd87153d4543a`。変更後の再build exit0。
- DEBUG=false、実worker絶対pathを指定。重いCargo処理終了を確認してから順次測定。

## 原本・実入力

原本Office5件の既存requestでexit0。shop XLSX1.473s、order XLSX0.100s、proposal PPTX4.401s、loom PPTX2.214s、entra PPTX1.538sで実frameを確認。各closeは元の5秒以内。cold RSS104672→283168KiB（差178496、元196608以内）、UI76→497、実document frame5件。終了後session/worker/workspace/artifact/cache/frame/texture全0。

生ログ `tmp/review-pair-office-five-2026-10-04.log`。5s close、15s first-frame、196608KiB cold budgetは変更なし。原本sourceは既存requestの実ファイルを使用し、私有原本をコミットしない。

実SheetタブNotes/Dashboard0→1→0とlegal data-descriptor DOCX requestもexit0。ログ `tmp/review-pair-sheet-input-2026-10-04.log`、`tmp/review-pair-legal-docx-2026-10-04.log`。viewport/expected bounds/page count/元assertは変更なし。

## 反復開閉

既存PPTX、XLSX、大きい原本loom PPTX requestを順に実行し、全コマンドexit0。各cold1回+warm10回のHTML/Office混合反復で実frameを各11件確認し、終了resource counter全0。

| 入力 | cold RSS差KiB | warm10 RSS差KiB | ログ |
| --- | ---: | ---: | --- |
| PPTX | 75296 | 9952 | tmp/review-pair-pptx-cycle-2026-10-04.log |
| XLSX | 143984 | 14464 | tmp/review-pair-xlsx-cycle-2026-10-04.log |
| 原本loom PPTX | 157936 | -8128 | tmp/review-pair-loom-cycle-2026-10-04.log |

元cold196608/warm65536KiB上限と条件付きclose期限を維持。これはin-process実host検査であり、packaged_binary_tested=false。配布main、全OS clean-machine、原本HTML正常終了、独立source-renderer95点、現HEADレビュー/required checks、公開/後処理の代替ではない。

## 最新上流条件

同じfresh runnerで公式canonical interaction契約もexit0。実controls-onを拒否し、Light typography/Dark diagrams両方のsame-frame controls-offと元crop geometryを確認。full PNG SHAはtypography `394f828d1a89f89d0f15f6b4a62a6d704f4b275b9222f0e2b1225d2930ccfa3d`、diagrams `f77baa764224dcc653ed0288263dab16c292d8a353933d4278e6343c8ed67bf1`。生資料 `tmp/review-pair-canonical-2026-10-04/`。これは95点採用・baseline置換・配布受入の代替ではない。

最新HEAD761439e reviewの追加P2 comment4177866298/threadPRRT_kwDORm09y86oy_ZOは未対応として取得。非ASCII大小文字のalias比較（École/école）を回帰再現・修正する。過去の二件resolveをもって現review全完了と扱わない。全thread/comments queryは各page next=false、既存manual-target公開方針P2も未解決。KRR側も最新Draft reviewで証跡Cargo linker検査P1を検出し、既存担当で修正中。

KRR担当の後発human「設計変えてないなら取り消さないで良いです」に従い、既存JNI/JVM設計非変更の検査修正を継続。PR105 HEAD9f06f14の通常push/P1 reply/resolve後にDraftレビュー中。公開latestはまだ0.4.22。KatanAは未公開版をpath/gitで採用せず、必要な公開版と残受入を待つ間も独立検証を継続する。
