# Windowsのtarget lease権限テスト

## 一次情報

[Windowsの実ジョブログ](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40166290854/job/121087914508)で、9838e374のCIは18件中2件が失敗した。保存した元ログはtmp/oct25-9838e374-windows-failure.log。

- test_distinct_targets_are_independent_and_bad_lock_refusesのmode 0700拒否期待。
- test_original_runtime_components_reject_world_writable_and_symlink_pathsのworld-writable拒否期待。

製品のcargo_target_lease.pyはWindowsではPOSIXの所有者・mode検査を行わない。WindowsのchmodからPOSIXの0700/0777を前提とする拒否を期待していたテストと、既存製品契約の不一致だった。PRをDraftへ戻し、実行中CIの取消しを要求した。取消しは成功扱いにしない。

## 修復範囲と検証

対象はテストのOS前提のみ。製品の拒否条件、ロック方式、symlink/reparse pointと非通常ファイル拒否、異なるtargetの独立性は変更しない。POSIX権限の検査はmacOS/Linuxで維持し、Windowsの残る契約も継続する。

macOSで既存のjust test-cargo-sweep-contractを実行し、全18件成功・実exit0を回収した。ログはtmp/oct25-windows-lease-contract-full.log。独立レビュー、通常commit/pushと新HEADのReady CIの実結果を続けて回収する。Windowsでの修復成功は新しいCI結果が出るまで未検証。
