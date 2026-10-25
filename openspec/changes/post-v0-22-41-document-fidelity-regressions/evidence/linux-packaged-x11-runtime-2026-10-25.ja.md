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
