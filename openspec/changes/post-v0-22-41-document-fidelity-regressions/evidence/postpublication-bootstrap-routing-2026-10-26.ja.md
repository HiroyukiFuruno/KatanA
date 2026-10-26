# 公開後の証跡更新に対する通常hookのphase選択

## 公開前に確認した不整合

source `7f7d940608f4fb908da2823546e58e4a784ada40` の通常pre-push / pre-prは、`check-pr-ready.sh --pr-bootstrap` を呼ぶ。公開後の証跡ブランチ `release/v0.22.42-public-release-evidence` でもmodeを変換せず、公開前の版増分検査へ進む。

この枝は既存のbranch-version helperで `0.22.42` として抽出され、task枝の早期終了にも、同版Cargoとの版差による早期終了にも該当しない。公開後にlatest releaseが0.22.42になると、CHANGELOGのprevious0.22.41との差で拒否される。

既存version checkerへ、サポートされている引数で `target=0.22.42 / latest=0.22.42` を渡した実unit結果はexit1、`Previous CHANGELOG.md release (v0.22.41) does not match latest published release (v0.22.42)` だった。ログは `tmp/oct26-postpub-version-unit.log`。これは公開後の入力条件を検証したunitであり、現在すでにReleaseが公開されたことや、実際の公開後pushを試したことは意味しない。

一方、Release Readiness workflowは既にこの固定枝を `--post-release-automated` へ正しく振り分けている。GitHubサービスではなく、local hook/controllerのphase選択不足である。監査は `tmp/oct26-postpublication-bootstrap-routing-audit.md`。

## 公開前に修正する理由

公開物のrecorderは実tag commitと現在の `SOURCE_ROOTS` を比較する。対象にはscripts・workflow・lefthookも含まれるため、公開後にcontrollerだけを修正すると公開物との来歴比較に差が生じる。hookを迂回したり、version checkerへ古い公開版を偽って渡したりせず、今回の公開sourceへ正しいphase選択を取り込む。

## 修正範囲と維持する条件

確定したbranch・target version・要求modeから選択する、外部I/Oのないselectorを用いる。固定の `release/v0.22.42-public-release-evidence`、target `0.22.42`、空modeまたは `pr-bootstrap` の組だけを `post-release-automated` へ振り分ける。他枝・他版・明示されたstrict/automated/artifact-pending等のmodeは変更しない。

controllerはtarget確定後、既存の版整合検査より前にselectorを使う。固定枝でも通常枝でも、Cargo版不一致は従来どおり既存の版整合検査で拒否する。

当初検討した「版差による早期終了がpost modeを迂回する」という仮説は、実fixtureでは確認できなかった。旧式の `[[ ... && ! is_ci ... ]]` は関数呼出しではなく文字列を評価し、通常枝の版差もexit1へ進んでいた。今回この既存条件を動かして新たにstaging成功を許可する変更は採用せず、元の内容・位置を維持する。この既存条件の整理は別の検討事項であり、主因のphase選択不足とは区別する。

既存の公開済みstable Release確認、通常release枝の版増分確認、preflight、Ready後の全5公開物検証は維持する。envによるmode上書き、hookの差替え・迂回、新しいtestport、閾値・timeout・coverage・ignoreの変更は行わない。

検証は実selectorへの入力表とcontrollerの配線契約を使い、GitHub・Git・checkerを偽装した新しいIT/E2Eは追加しない。公開後には通常hookと既存の実公開物検証を実行し、その結果を回収する。新HEADの通常commit/push・全review・必須CI・実配布公開と後処理は未完了として継続する。

## 検証結果

- 既存flow契約に、実Bashのselector入力表・実assignmentの配置・real Gitの版不一致拒否を追加し、20 testsがexit0。ログは `tmp/oct26-postpub-routing-target-final.log`。
- real Gitの一時repoで、固定証跡枝/Cargo0.22.43と通常release/v0.22.43枝/Cargo0.22.42の双方がexit1・版不一致で拒否されることを確認した。Git初期化前からhook由来の全 `GIT_*` と個人Git設定を隔離し、実作業repoへ操作が向かないようにしている。
- 継承された別Gitディレクトリ・worktree・index・config指定を入力した追加の実fixtureもexit0で、指定先が作成されないことを確認した。ログは `tmp/oct26-postpub-git-environment-isolation.log`。外部コマンドの置換やサービスのmockではない。
- 初期RED `tmp/oct26-postpublication-selector-red.log` は、新selector欠落7 errorとcaller配線assertionの失敗を記録したもの。公開後の実push失敗やWindows native不合格の再現とは扱わない。
- `tmp/oct26-postpublication-selector-shell-red.log` のordinary staging期待0/実1は、検討途中で撤回した仮説の記録である。今回の採用条件や修復成功の根拠へ流用しない。
