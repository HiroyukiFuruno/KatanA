# macOS配布版の文書操作・終了の実測

## 固定した成果物と実行

source `c05b7f03588c2e631ece377a9238cb6e6659b953` の既存 `just package-mac` から生成し、ZIPから新展開したuniversal配布版を使用した。main SHA256は `5870a94b9bba7b3400ec261da4364d8ffea1f24388e8da3e68cf2303d1dcb227`、workerは `fae899338034234d964aab41430691cc9014a6bb1d093f822f3c0b5c57e07131`。署名・配布契約・両architecture検査は成功済み。通常利用の設定は検証入力にしない。

最初の隔離実行 `tmp/macos-isolated-document-qa.daSs7N` は本人が規約に同意した。画像の実切替10回、PPTXのスライド1→2→1・Escape復帰を記録した。最初の単一クリックだけの10記録は切替になっておらず、成功数に含めない。終了後に画面取得するとツールがアプリを通常設定で再起動したため、そのインスタンスを終了し、以降は終了後の画面取得を行わない。通常設定・文書を操作する検証には利用しなかった。

本人受諾済みの同じ隔離設定を複製せず再利用した `tmp/macos-isolated-resume.sfZHWy`、PID99326で文書操作を実行。前面に出す本人操作後、XLSXのDashboard→Notes、DOCXの1→2ページ、PDF描画、HTMLのスクロール・折りたたみ操作を確認した。所有するHTML複製だけに変更文字列を挿入し、表示更新とrenderer epoch 1→4を記録、元SHA256へ復元した。ファイル監視も動作しており、この更新を再読込ボタン単独の因果証明とは扱わない。

## 回収結果

原本Office6件・HTML1件を各10回開閉した70件と、本人報告のDOCX/PDF6件を各1回表示・終了した6件の計76件で、実frameとactive文書、閉じた後の新しいreceiptを確認した。各closeでhostのpreview/surface/frame/texture/cache、文書worker、KDVの全8資源値が0。`validated-functional-results.json`、各AX/画像/result/receiptを上記runに保存。画像は拡張子がpngでも実データはJPEGであり、PNG画質・画素一致の評価には使用していない。14件では撮影中にreceiptが1frame進んでおり、初回readyと撮影時のsequenceを区別して保存した。個人文書の画像・本文は公開しない。

両隔離実行は通常の終了メニューから終了し、helper実exit0とアプリexit0を回収した。source・入力の不変性も確認した。これはDEBUG付きの機能・資源検証であり、通常OFF性能、Chrome比較、独立95点、Intel/他OSの実行、公開後の最終配布受入の証明ではない。

## 未解決のメモリ観測

資源値0でも、同PIDのphysical footprintは原本反復前198184296bytesから10回後376182592bytesへ増加した。RSSも296763392→482426880bytes。予算の成功扱いにはしない。`vmmap.after-cycle10.txt` はMalloc Large(empty)167.8MiBを示す。後続のheap診断はallocatorへ影響するため、診断後の減少を通常検証の成功へ代用しない。

別の同成果物・同隔離設定の実行 `tmp/macos-isolated-resume.0F4yIi`、PID63479でもHTML単独10回の新しいreceipt/close0を確認し、physical footprint173360280→277598408bytes、RSS193232896→324812800bytesを観測した。続けてOfficeを一件開閉しても使用量は減らず、原因をHTMLだけと確定しない。この実行も通常exit0。記録先を閉包が取り違えた最初の失敗試行は除外し、run ID/PIDを固定した10件だけを採用する。

既存の返却条件がHTML単独closeを対象にしないことはソースと実caller回帰のREDで確認した。HTML終了を既存回収へ接続する候補は別差分で検証中であり、配布版での改善は未証明。KDV/KRRの追加実装・閾値緩和・保護機能無効化は行わない。

## b0ba969bの新しいuniversal配布版

`b0ba969b7355f8d9a1af1cce60af01d721073d22` の通常package工程は実終了0。`tmp/macos-isolated-package.wjrLbB` にZIPと新展開app、配布契約・deep strict ad-hoc署名・本体/worker両architecture・source/両lock不変・展開前後cmpを保存した。ZIP SHA256は `fe2ccc0fc01071809ed97bb729e383a546780f33a30f1dfebb7574420c74af7e`、mainは `ee5ff01c9908949407a5a93e2644b772ccdd5ca46be1006de5afad3ec331a128`、workerは上記と同じ。

本人受諾済みの同じ隔離設定で、この新展開appの実mainを起動した。run `tmp/macos-isolated-resume.o4v0RJ`、PID48537で原本HTMLとPPTXを各1回表示し、実frame/URI、新しいclose receipt、全host/KDV資源0を確認した。観測されたready時間はそれぞれ1189.65ms・3224.07ms。通常メニューでアプリを終了し、アプリとwrapperの実終了0、実行前後のHEAD不変を回収した。撮影画像はJPEGとして保存した。

`validated-functional-results.json` はPID/run ID・入力SHA・実frame・新close sequence・資源0・画像signature・通常exit・HEAD一致を再照合している。2件だけの機能結果であり、旧c05の76件をこの新配布版へ付け替えていない。10cycleメモリ予算、独立比較、clean-machine、Intelでの実行、公開済み配布物の受入は別途必要。

別のARM専用診断appでは原本18件の描画/close/正常exit0も回収したが、universal最終配布版や通常性能の成功には代用しない。候補フォントを字体照合前に全量読むKatanA側の不要IOを特定し、元の字形選択を維持して必要metadataだけ読む修復を続ける。これはリークの確定や性能合格ではない。
