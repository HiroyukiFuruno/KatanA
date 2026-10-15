# 固定HEADの原本Office・PPTX操作・HTML受入

## 実行条件

既存release worktreeのHEAD `f8c78b9725615f7da883fe5be98190a89673e76a`、clean treeを開始・終了時に確認した。既存`just test-document-fixtures`をJOBS=1/CARGO_BUILD_JOBS=1で一回実行し、同session36713からactual exit 0を回収した。実行中にsourceを編集せず、本人アプリ・設定JSON・Termsを操作していない。

入力は本人提供の`/Users/hiroyuki_furuno/Desktop/KatanA-不具合`と同ディレクトリの`要件定義書_v1.0.html`。元原本を変更していない。raw logは`tmp/original-fixtures-f8c78b97-2026-10-15.log`。

| Identity | SHA256（開始・終了でlock一致） |
| --- | --- |
| Cargo.lock | `a6035ec9364f734fd132d58ca909cfed0b9c9e13c25ffc894165d79060db2af4` |
| scripts/screenshot/Cargo.lock | `ef7151771258e762d1962647ccf4625dd668d48ec3a5246b60bcb48734955896` |
| 明示release Office worker（終了時） | `dc334210b1b1e466a9a961fed4cd69eb1643120b51400ccd3d5e8e484420480b` |

## 結果

公式scriptが`cargo build --locked --release -p katana-ui --bin kdv-office-worker`を実行し、生成workerの絶対pathを明示して原本受入を行った。release buildは4分08秒。別のdebug workerや暗黙fallbackではない。

| Office原本 | 形式 | item数 | first_frame_ms |
| --- | --- | ---: | ---: |
| libre-chat_vs_loom.pptx | Pptx | 9 | 19188 |
| librechat_entra_oidc_vs_saml.pptx | Pptx | 11 | 14678 |
| shopchannel_analysis.xlsx | Xlsx | 5 | 13067 |
| 【681303_発注依頼書】…_2021年1Wk2＿モードコム.xlsx | Xlsx | 1 | 13187 |
| 【チャット型AIエージェント】フェーズ1 …PoC提案_r3_20260616.pptx | Pptx | 26 | 17394 |
| 視聴購入data_セグ別201805_20260803.xlsx | Xlsx | 1 | 20712 |

Office exact test: 1PASS、98.27秒。全6件active_index=0、PptxはPage、XlsxはGrid。

PPTX exact test: 1PASS、9.68秒。原本3件それぞれready/next/stale_rejected/closeが全true。

HTML exact test: 1PASS、23.64秒。元入力の`#s15`でscroll_y=3048.824、content_height=9539、first_frame_ms=10492、sticky_sidebar_ratio=0.9352。

## 判定範囲

実原本を使った現在HEADのin-process host受入であり、独立各95点、Chrome完全一致、速度改善、RSS/footprint、アプリ正常終了、全OS配布版、本人実機確認の成功ではない。Office初回13～21秒の遅延を隠さず、次期KDV #65の課題と区別する。PPTXテストのclose=trueも文書操作の検証であり、アプリ終了時間の証拠に代用しない。

## 同HEADのreview

依頼6187783624に対応するbot応答6187837805はreviewed commit `f8c78b9725`でmajor issueなし。空本文のCOMMENTEDレビューを承認とは扱わない。全REST reviews159/comments177/conversation139（各2pages）、GraphQL88threads（outer終端・nested欠落0）を取得し、未解決は既存manual-target方針P2の3threadsのみ。Ready、required CI、merge、Release、全Issueの完了を意味しない。
