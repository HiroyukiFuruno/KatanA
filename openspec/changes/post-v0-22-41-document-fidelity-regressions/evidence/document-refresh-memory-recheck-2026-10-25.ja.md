# 再読込修復後のメモリ再測定

## 候補の由来と判定

実装`08ec21a1`を含むclean source `d668a32f2933581a3ae20364f7a2eda01b22881f`から、native releaseとARM64 main/workerのビルドをそれぞれ終了0で回収した。中間universalビルドは、後続の観測修正前にx64の重複生成を続けないため、所有実行へSIGINTして終了130を保持した。

その実ARM64 binariesと同sourceのbundleから作った診断候補はstrict ad-hoc署名検査に成功。全構成package・最終source・公開版受入の成功ではない。full source tree SHAは`846752b0c4f8811070af409cf92f75b35234ed27ea185fc9bf5303362aa4efb6`、main SHAは`fb59f96a5abc501b888efdc9fd2f21fdd776ccb4b618b74384556cb182cd4112`、worker SHAは`19b5d2956ee80fa4956634de5dd45a9bacfbbda9a5d711828bc6d1f4d38c713c`。候補生成後の作業treeに適用した未commit観測コードを、この既存binaryに含むとは扱わない。

同じ本人同意済み隔離設定をworkspace/cacheだけ可逆退避して使い、各runを別processで起動した。DEBUG=trueの読取専用receipt、heap/vmmap inspector OFF、実外部mainメモリ全観測を保持した。10反復はHTMLとOfficeを交互に実表示・実closeし、各closeでhost/KDV資源とworkerが0であることを確認した。

| 入力cohort | 機能・終了 | cold RSS / physical増分KiB | steady RSS / physical増分KiB | 元判定 |
| --- | --- | --- | --- | --- |
| 大型XLSX＋原本HTML | 各10、計20描画・close、main通常終了0 | 171040 / 127584.914 | 97680 / 79058.414 | cold成功、steady未達 |
| 大型PPTX＋原本HTML | 各10、計20描画・close、main通常終了0 | 244960 / 227936.984 | -976 / -21789.844 | cold未達、steady成功 |

元予算はcold196608KiB、steady65536KiBをRSSとphysicalへ独立に適用。各closeとapp終了5000msも維持した。大型XLSX runは1246観測、PPTX runは1055観測、collectorは両方終了0。各判定CLIは機能成功でもメモリ未達により終了1である。

大型XLSXの全量`document_file_read/source_intake`は旧候補42回から新候補10回へ減り、各handoff/dropも10回だった。30回のrevision probeはすべてunchanged。無駄な全量読込の解消は確認できたが、残る予算未達の解消とは別である。

ローカル正本は`tmp/macos-isolated-resume.eCN41J`と`tmp/macos-isolated-resume.znMCz3`の全receipt・外部観測・実capture・結果JSON。原本名・本文・画像を本公開文書へ転記しない。これら私的rawがCI環境にも存在するとは扱わず、正式host admissionは別途実producerから生成する。

## 別runの保持内訳診断

同じ実binaryで、別の2processを使い、開始・初回close・10回目closeに公開heap/vmmapツールを実行した。各captureでPID/start/main SHA、安定した資源0 receipt、tool実終了0・raw SHAを前後で照合した。NoCompact/stack loggingを追加していないが、inspectorを実行した別runであるため、上の通常性能値や合格の代用にしない。

| cohort・区間 | zoneの生存heap正確値の増分 | 実外部RSS増分 | 解放済みLarge領域の常駐量 |
| --- | --- | --- | --- |
| XLSX 初回→10回目 | +127488bytes（約0.122MiB） | 約+44.20MiB | 約86.3→124.6MiB |
| PPTX 開始→初回 | +10504848bytes（約10.02MiB） | 約+238.36MiB | 開始未掲載→約180.1MiB |
| PPTX 初回→10回目 | +146048bytes（約0.139MiB） | 約-46.05MiB | 約180.1→122.8MiB |

両runともIOSurface、CGImage、live Largeは反復中に増えていない。XLSXの初回/10回目の全zone生存heap合計は実入力1本の86321258bytes未満で、当該System allocator経路の元source Vecが丸ごと生存する説明とは整合しない。ただし、native VM全体が0という意味ではなく、非ゼロの小さなlive増も保持する。

heap、RSS、physical、vmmapは異なる集計・観測時点であり、丸めたregion差分を正確な内訳比率へ換算しない。新runのfree-residentの具体的callerやKatanA/KRR/OSの帰属は未確定。旧NoCompactの別runのcallerを新runへ移して原因確定とはしない。診断のraw・exact値・hashと限界はローカル`tmp/read-only-action-observation-draft/qpVJnP-final.ja.md`、`XKZMh2-final.ja.md`と各JSONに保持する。

## 未完条件

通常2件の元メモリFAILは維持している。PPTX初回読込15秒に関する本人の既存例外は、メモリへ適用しない。今回2原本限定のメモリ未達を既知制限として公開するか、元予算への修復を続けるかを本人へasync確認済みだが、現時点で回答は未受領。承認済みとは記録せず、他の実装・検証・通常commit/push・review/CIは継続する。
