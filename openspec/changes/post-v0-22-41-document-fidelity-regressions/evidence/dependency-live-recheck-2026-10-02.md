# 依存関係の公開状態再確認

前回の正式更新後にread-onlyで再確認した。63 unique直接依存と補足4 crateの
公開crates.io API照会は67成功/0失敗、適用先lockに存在しない新規直接更新候補は0件。
この追加監査ではCargo update・source/lock変更・兄弟repo編集を行っていない。
全67照会のraw集計は担当実行出力のみで、保存JSONを独立した検証成果物とは扱わない。

main側でも主要3 crateの公開version API checksum/non-yankedと両lockを再照合した。

| crate | registry版 | checksum |
| --- | --- | --- |
| katana-document-viewer | 0.5.8 | d77723a4ae0fb4dfe267810a23c77ba558c2d7dd4293539228d1f62c5511b5c0 |
| katana-render-runtime | 0.4.21 | 79f308cbef4468ba66a2b2f98a2b03ce320d6eace081985d966f06da95e1c424 |
| katana-ui-core | 0.4.0 | 9f2e0ae7eb5d0706dd289c3121058132c042be778f659c071c25cf4fa0a621d0 |

対象3 crateはregistry sourceで、path/git overrideを残していない。
主要3版が公開されていることは、残るKRR #95・KDV #58/#59の受入完了を意味しない。

公開crypto-common 0.1.7のdependencies APIはgeneric-array `=0.14.7`を要求し、
両lockも0.14.7。0.14.9へのpatch更新と1.xへのmajor移行を無条件には適用できない。
前回更新の既存制約と全ゲートを維持する。過去の引継ぎで「0.14.9維持」と書かれた
箇所は候補版と採用版の混同であり、現lock/APIを一次情報とする。

一次情報は `https://crates.io/api/v1/crates/{name}/{version}` と
`https://crates.io/api/v1/crates/crypto-common/0.1.7/dependencies`。
