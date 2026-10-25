# Linux配布版のX11実行時依存

## 実際の失敗

[公開run40192198622のLinux job121167342873](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40192198622/job/121167342873) はビルドと配布identity生成に成功した後、配布物を展開した起動検査で最初の描画前に終了した。実ログは `tmp/oct25-canonical-linux-package-failure.log` に保存した。

`xkbcommon-dl 0.4.2` のX11初期化が `Library libxkbcommon-x11.so could not be loaded` でpanicした。公開workflowのLinux依存には `libxkbcommon-dev` があるが、X11用の実行時パッケージ `libxkbcommon-x11-0` がない。KDV/KRRの修正待ちとして扱わず、KatanA公開環境の欠落を修正する。[Issue363](https://github.com/HiroyukiFuruno/KatanA/issues/363)。

## 修正と判定

Linux配布ビルドjobの依存へX11実行時パッケージを追加し、既存の配布起動契約検査で同jobへの配置を確認する。元の起動heartbeat、process identity、保護、timeout、全OS一括公開条件は変更しない。

対象契約検査・通常commit/push・新HEADレビュー・必須CI・修正版配布起動・公開は順に証跡を回収する。元runのLinux失敗を、ビルド成功や後続sourceの合格に書き換えない。

## 対象検証

Cargo registryの `xkbcommon-dl 0.4.2/src/x11.rs:47-53` は `libxkbcommon-x11.so.0`、次に `libxkbcommon-x11.so` を動的ロードする。実行時パッケージの収録ライブラリは[Debian公式ファイル一覧](https://packages.debian.org/trixie/amd64/libxkbcommon-x11-0/filelist)、Ubuntuの提供は[公式パッケージ](https://packages.ubuntu.com/ca/noble/libxkbcommon-x11-0)でも確認した。

既存startup契約検査にbuild_linux内だけの依存検査を追加。元HEADのworkflowへ同じ抽出判定を適用してexit1、修正後の `bash scripts/release/test-packaged-startup-contract.sh` はexit0、`git diff --check` はexit0。別jobへの同名依存追加では合格にならない。実GitHub Linux配布起動の再検証と全OS公開は未完であり、この対象検査を代用しない。

独立最終レビューで上記2sourceのSHA256一致、P0/P1なし、起動保護・閾値・timeout・公開条件の弱体化なしを確認した。全OS公開契約 `python3 scripts/release/test_all_os_release_contract.py` は4件成功/exit0。初回の同Pythonファイルをbashで呼んだ誤実行はexit2であり、修正後の成功へ隠さず区別する。

修正後SHA256: workflow `c794fb6304efe917bcc15a752cb4559c7ee87f998cd03a5893fb3e70dbb8d5eb`、startup契約 `a341769855b4e0f9add89a9b267d3140b60e9e289cddf151432493edb65847b3`。

## 修正後の配布起動で確認した次の不足

`d352db0d` の[必須CI40204301916](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40204301916)は全8ジョブ、[Readiness40204301912](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40204301912)も成功した。全レビューの未解決0件を再取得し、[PR365](https://github.com/HiroyukiFuruno/KatanA/pull/365)を通常mergeした。merge `5cb787cf7c8a4d10101cd0f823b0aa574080a20a` は元headと完全Git treeが一致する。

このsourceの[全OS公開run40212332637 / Linux job121228922921](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40212332637/job/121228922921)は、Linuxビルド・tar構成・payload identity生成の後、最初のフレーム前に `Wgpu(CreateSurfaceError(...FailedToCreateSurfaceForAnyBackend({})))` でexit1になった。rawは `tmp/oct26-canonical-linux-surface-failure.log`。以前のX11ライブラリ欠落のエラーは出ていないが、Linux起動全体の成功ではない。

実依存の `wgpu-core 30.0.1` は、成功したHAL instanceごとにsurface生成を試みる。このエラーの空のmapは、その試行対象のbackendが0件だったことを示す。個別のdriver初期化が失敗した理由のdebugログは得られておらず、すべての原因を確定したとはしない。

通常のLinux CIは `mesa-vulkan-drivers` / `vulkan-tools` を導入し、LavapipeのICD存在とadapterを確認して、描画受入にsoftware Vulkanを明示している。一方、公開jobにはその準備と選択がなかった。この公開環境の不足を補い、同じ実配布版の起動検査を再実行する。製品のrenderer fallback、保護、timeout、heartbeat/RSS/font/worker/identity条件は変えない。

`vulkaninfo --summary` はadapterの事前診断であり、実際のnative window surface成功の証明にはしない。surface成功は変更しない配布起動検査の初回フレームと継続フレームで確認する。新sourceの通常commit/push・current review・必須CI・全OS配布起動・公開は未完である。

公開jobの依存を補い、build前のadapter診断と、実配布起動stepの両方で同じLavapipe ICDを選択するようにした。環境変数は各stepに限定し、Rustビルドや通常起動には設定しない。既存契約検査は `build_linux` 内の該当stepだけを抽出するため、別job・別stepに同じ文字があっても代用できない。

元workflowのfixtureでMesa依存・診断stepがないことを同じ検査で検出し、それぞれexit1を回収した。元fixtureは `tmp/oct26-linux-vulkan-contract-red.log`、実際の2件の終了結果は `tmp/oct26-linux-vulkan-contract-red-result.log`（SHA256 `b05f80a82f574035c88ac9bb0e8cb7e120623029586bd36ace2d1bb4ec29b841`）へ分離して保存した。修正後は既存startup契約exit0、全OS公開契約4件/exit0、bash構文・diff検査exit0（`tmp/oct26-linux-vulkan-contract-green.log`）。fixture本文だけを実行結果の証明にはしない。この構成検査をnative surface成功へ読み替えない。

対象source SHA256はworkflow `d2ed0964736f53fcee657f644fbbd7c0d387e7b423d664e19d3fc58adb341e4f`、startup契約 `c75de4bcf45581968251fb07a79d5dd99222fae9e69199ac9824b94799eeaef9`。実配布起動のshell、製品Rust、元のtimeout・観測回数・各閾値は変更していない。
