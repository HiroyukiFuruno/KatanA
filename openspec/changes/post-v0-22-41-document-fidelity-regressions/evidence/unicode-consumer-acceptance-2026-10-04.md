# Unicode consumer修正後の実受入

GitHub作業ブランチのsource c62644f20cf5666f84d4cb6ea8fbb4e0954c296b、KDV=0.5.11/KRR=0.4.22/KUC=0.4.1/office2pdf=0.8.1 registry lock。KatanAの正式リリースではない。locked release worker再build1m16s・runner再build1m44s、共にexit0。

- worker SHA256: 819016885f855e3e56f99ff07a6a3e9ba6d992824c8d0d4a7bcbd87153d4543a
- runner SHA256: 6b857cc9b60d792db8fc167640b7fe63c2d78c86120b4d4ceaa6dfa9158e68f4

DEBUG=false、実release worker絶対パス、重いローカルbuild終了後に逐次実行。CLIはexecution_mode=in_process/packaged_binary_tested=falseと明示。配布main/全OS/独立95点/上流新KRR原本HTML正常終了を代替しない。

## 失敗を残す

tmp/unicode-consumer-office-five-2026-10-04.logはexit1。first frameはshop XLSX1.306s/order XLSX0.096s/proposal PPTX4.386s/loom PPTX2.295s/entra PPTX1.486s、全close5s以内・最後の資源カウンタ全0。ただしbaseline RSS104816→終了305648KiB、delta200832が元196608上限を4224KiB超過。成功へ読み替えず、元上限を変更しない。

前435sourceはdelta178496だった。差はshop_openから現れ、font payload/lease release経路そのものは変更されていないが、KDV資源カウンタはegui font payloadやallocator常駐量の全てを表さない。Unicode修正の一時allocation・解決faceの変化・allocator/run順を、現在のログだけで因果確定しない。KDV #59担当へ新sourceの失敗と未確定の帰属をhandoff。同binaryのcold/warm反復を続行。

## 独立した成功

同source/binaryで次の逐次batchはexit0（session25454）。失敗したOffice5の後続を止めず別batchで実行した。

- xlsx-sheet-tab-input: Notes/Dashboard実クリック、active0→1→0、bottom rail bounds[250,828,1250,872]を検証。tmp/unicode-consumer-sheet-input-2026-10-04.log。
- docx-data-descriptor-regression: legal DOCX2pages、実frame/描画とclose、元15s/5s条件を維持。tmp/unicode-consumer-legal-docx-2026-10-04.log。
- v0-22-38-multi-format-documents: 全38手順成功。XLSXの実Notesクリック・bounds・active1、PDF/DOCX/PPTXの既存page navigationとdirect URL recovery等を維持。tmp/unicode-consumer-multi-format-2026-10-04.log。独立Sheet scenarioの成功をこの統合fixture成功へ流用していない。

4.14原文はmulti-format内の実Sheet入力・active assertion・geometry・page navigation維持であり、同fixture内0→1→0を新たな必須条件へ増やさない。配布main入力は4.11/4.13/4.4等に未完として残す。

## 反復と再現確認

同じfresh binaryでcold1+warm10のHTML/Office混合開閉を逐次実行し、3件ともexit0（session77910）。元cold196608/warm65536KiB上限、11回実frame、最終資源カウンタ全0を維持した。

- PPTX: cold+75264/warm+10080KiB。tmp/unicode-consumer-pptx-cycle-2026-10-04.log。
- XLSX: cold+160208/warm+672KiB。tmp/unicode-consumer-xlsx-cycle-2026-10-04.log。
- 原本large loom PPTX: cold+156208/warm+9248KiB。tmp/unicode-consumer-loom-cycle-2026-10-04.log。

canonical interaction契約はcontrols-on拒否と同frame controls-off両captureがPASS。tmp/unicode-consumer-canonical-2026-10-04.log。独立95点検査の代替ではない。

同binary・同原本Office5の別process再現（session48484）は再度exit1。baseline104928→305648KiB、delta200720 >196608（4112KiB超過）、最終資源カウンタ全0。tmp/unicode-consumer-office-five-repro2-2026-10-04.log。初回失敗を消さず2標本とも失敗として残す。反復3件の成功とOffice5の超過は別の入力・順序であり、同一受入成功へ読み替えない。原因は引き続き未確定。

## DEBUG診断の境界

同じc626 release runnerをDEBUG=trueで実行した診断は104928→293200KiB、delta188272・exit0（tmp/unicode-consumer-office-five-font-diag-2026-10-05.log）。通常条件の失敗二標本を取り消す受入証跡にはしない。最初の診断試行はfixture copyでdisk fullとなったため除外し、終了済み試行の再実行結果と区別する。

診断の書体探索は2回、requests2/3・採用faces0、130候補readの合計236961000bytes、metadata366faces・selected0、resolve71.6/74.7ms。resolverは候補bytesを読み、metadata照合後に不採用payloadをdropする。今回のsampleで新しい永続lease/payload保持は観測されず、一時読込・Unicode正規化allocation・allocator残存・実行順が検証候補である。ただし資源カウンタは全heapを説明せず、これだけで原因は確定しない。既存KDV #59担当へ通常失敗と診断境界を共有し、OPENを維持する。
