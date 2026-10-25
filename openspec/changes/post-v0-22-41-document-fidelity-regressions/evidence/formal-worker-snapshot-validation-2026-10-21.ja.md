# 正式coverageのworker固定を実製品で検証

## 対象と実結果

署名済みHEAD `0f6120872811ebafb870089300a3be59151ca397` に対し、既存 `just coverage` を新しい専用targetで一回実行した。session `75161`、wrapperの開始はUTC `2026-10-20T18:13:29Z`、終了は `18:39:22Z`。子coverageと終了観測のexitはともに0だった。

| 既存工程 | exit | command秒 |
| --- | --- | --- |
| workspace-lib-bin | 0 | 540.466 |
| workspace-integration | 0 | 463.760 |
| ui-parallel | 0 | 52.976 |
| ui-serial | 0 | 9.038 |

全4manifestのsource前後は同HEAD・同tree・cleanで、両lock SHAも一致した。開始終了の共有Git設定ファイルSHAも不変。debug/stripの4環境、incremental=0、jobs/threads=1を維持し、元のselector・既存skip・assert・report・strict gateを変更していない。通常testの重複再実行は行わなかった。

既存reportは成功し、strict document surfaceは100%・未実行行0。上記command時間はworker準備・全report・診断・外部待ちを含む全体速度の比較ではなく、削減効果の証明へ流用しない。

## worker境界の検証

4manifestはすべて同じ絶対path（repo-relative表記では `tmp/canonical-0f6120-20261021-target/tmp/coverage-worker.8j7hpga6/kdv-office-worker`）とSHA `410071c9a4fc9219ba99cd02d5062c31fc9f5fb39a6be471cb36f480c1adbf04` を記録した。主査の途中観測・終了後の実ファイルSHAも一致した。

同じ専用targetの `debug/kdv-office-worker` は終了後SHA `10556f9a836159ae9e6e629dd7381cb8ff3a59fe64e51e6c494a4aa4f9468c13` で異なる。Cargo本体の再linkが起きてもdocument sessionへ渡すsnapshotのpath/bytesは固定された。旧repo root targetの別binary SHAと取り違えない。これは実入力と実ファイルの検証であり、各test子processを個別追跡した証明ではない。

## 証跡と残範囲

生ログは `tmp/oct21-coverage-0f6120-canonical.log`、4manifestは `tmp/test-inventory/coverage-0f6120-20261021/coverage/`。実行wrapper・準備・生ログ・4manifest・両workerの実binaryを `tmp/oct21-coverage-0f6120-preserved/` に保全した。主査が9ファイルのSHAをstrict-checkし、両binaryの元/保存copyもcmp一致を確認した。

終了後は専用targetだけを既存のprofile排他付き `just clean` で回収した。session `89692` はactual exit0、13,533 files・11.7GiB回収。主査がtarget不存在と保全9SHA再一致を確認し、空き容量15,089,548KiB（約14.39GiB）で共有実行枠を返却した。rawは `tmp/oct21-coverage-0f6120-owned-clean.log`。旧root・screenshot・他担当のcacheは操作していない。

manifestの `complete=false` / `reuse_authorized=false` は維持する。実行同等性・trusted reuse・Windows cold/warm・実重複削減・最終graphの全gate/current review・packaged受入・全公開の完了ではない。上流修正版採用後の正式push gateを省略する根拠にしない。
