# Linux x64実機受入の実行経路監査（2026-10-23）

## 結論

13:01 UTC時点、今回のKatanA作業に利用可能と確認できたLinux x64 native実行先はない。これはアプリの起動失敗や、上流だけを原因とする停止ではなく、実行環境・実行権限が揃っていない状態である。実機受入は未実施のまま保持する。新たな製品欠陥を根拠なく実装したり、既存DoDを完了へ変えたりしない。

## ライブ確認結果

| 経路 | 実結果 | 判断 |
| --- | --- | --- |
| Colima | default Running/aarch64/4CPU/8GiB/150GiB | ARM Linux。x64 user-mode QEMUをnativeへ読み替えない |
| Docker default | `linux/aarch64`、exit0 | Colimaと同じARM経路 |
| Docker Desktop context | socket不在、exit1 | 接続可能な別daemonではない |
| Codexの設定済みproject | hostIdはlocalのみ | 接続済みの別Linux x64 hostは確認できない |
| KatanA self-hosted Actions runner | `{"runners":[],"total_count":0}`、exit0 | このrepoに登録済みの専用runnerなし |
| 既存Codespaces | 一覧API403、codespace scope不足 | 存在・利用可否は未確認。存在しないと断定しない |
| GitHub-hosted CI | 既存workflowに`ubuntu-latest` | x64経路はあるがcurrent sourceで今実行できる状態ではない |

`~/.ssh/config`のIncludeには別案件向け設定があるが、今回のKatanA用接続先と確認できないため接続・認証・実行していない。権限不足を避けるためのtoken scope変更、新クラウド/VM作成、課金サービス利用も行っていない。

## GitHub Actionsを今起動していない理由

- PR346はfresh取得でOPEN/Draft、公開HEADは`4c3a9272a6e537d0fda4f6d4df9f9f5fa457fa95`。ローカル`a7ee0f3dbff9f08cf5faa88b10bb2489cf177819`とは異なる。旧公開HEADの起動を現候補のnative成功として流用しない。
- `test-and-build.yml`はDraftで重CIを起動しない。既存`workflow_dispatch`入口もない。今回だけ別workflowや別refを作り、この方針・通常push gateを迂回しない。
- `build-and-release.yml`のmanual dispatchは単独起動検証ではなく、version処理・3OS生成・GitHub Release公開へ進む。Linux確認のために公開処理を発火しない。
- 通常経路は安全な上流公開依存採用、通常push、current review修復、Ready、current必須CI、その後のRelease native packaged smokeである。既知の供給網拒否やレビュー順を無視しない。

## 独立して今できる作業と残条件

- 同一候補の構造/SHA、QEMU起動の原因実測、KatanA失敗診断の修復、実プロセス回帰macOS/Linux ARMは先行証跡へ実結果を記録済み。今回も製品・検査条件を変えていない。
- 今回の経路調査と未実施区分の修正は完了。Linux x64 native、Windows/MSI、Intel実機、clean-machine、本人受入、current公開レビュー/必須CI/公開は別の未完条件である。
- 直接実行する代替経路には、今回利用してよいLinux x64実機/VMの接続先・実行権限の提供が必要。接続先が得られれば、既存tar SHA `82eaa5ff384d1162483590bca73f698710aaeed7f8a5dd39987e370211a27427`を照合し、元smoke・元予算で検査する。QEMU結果で代用しない。

先行証跡: [同一性FAILの実測・診断修復](linux-startup-process-identity-2026-10-23.ja.md)、[配布候補の生成と限定受入](independent-distribution-candidates-2026-10-23.ja.md)。
