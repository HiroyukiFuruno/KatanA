# Registered image directory recovery

## Current review

- PR346 review5433854835, inline4199818144, threadPRRT_kwDORm09y86popFe, reviewed public8f7ae2b0fa48734137ccd8bb469e1d6b0cdf3dc6.
- Linux/inotify removes a watched directory descriptor when its directory is deleted. The coordinator previously retained that directory in `watched_dirs`. Subsequent registration after recreation incorrectly skipped the native `watch` call.
- This is distinct from initially missing-parent retry and from the previous unrelated-subscriber error broadcast repair.

## Reproduction

- Private real `RecommendedWatcher`, real temporary PNG files, native callback channel, already registered parent deletion/recreation, subsequent overwrite, existing five-second observation deadline.
- Initial12133 exit101 was a fixture import compile error, not product RED.
- Corrected17331 exit101: subsequent native overwrite event timed out, one genuine failing regression, 5.02seconds. Raw`tmp/deleted-directory-recovery-linux-red-final.log`.
- Initial recovery81431 exit0: same regression passed in0.01seconds. Raw`tmp/deleted-directory-recovery-linux-green.log`. Subsequent concern separation and additional lifecycle regressions still require final validation.
- Final concern-separated candidate: official native loader35810 exit0,34passed across40workspace suites,1.63seconds; Linux full loader84146 exit0,35passed including native overwrite recovery,1.28seconds. Raw`tmp/deleted-directory-recovery-native-loader.log` and `tmp/deleted-directory-recovery-linux-loader-final.log`.
- Cross-platform real-watcher unit boundary verifies exact/descendant directory removal, affected loader registration state cleared, unrelated subscriptions retained without Failed, ordinary file removal preserving its parent, and genuine backend error visibility. This unit boundary is not a native packaged-app or rendering receipt.

## Repair boundary

- Remove only affected directory entries, including watched descendants of a removed ancestor.
- Backend `WatchNotFound` after inotify auto-removal is expected. Other unwatch errors remain observable through diagnostics.
- Send affected subscribers through the existing Failed/bounded-registration retry route; Changed alone does not clear loader registration state.
- Preserve unrelated targets and their actual failure diagnostics, ordinary file-removal behavior, and the existing FSEvents topology-overflow contract after successful backend mutation.
- No public API, allocator, threshold, fixed-wait extension, new allow, upstream edit, or application operation.

## Remaining release obligations

Official static36706 exit0: AST23passed, impacted strictClippy and fmt-check passed. Source was integrated by the normal hook with verified G signature as8f461241c18c5eff045eb3d081c13f6b48c680ab. The unpublished local commit message was corrected to the required `fix:` format using normal amend; no hook bypass or force push.

Normal documentation commit/push, direct review reply/resolve/fresh query, current-head CI/coverage, current package and all five distribution/real-host receipts, publication and postprocessing remain pending. The prior8f7 coverage and68db package do not prove the repaired candidate.

## Self-review

PASS for the changed source and its callers: private struct/impl responsibilities separated within the existing method-size rule; coordinator-only recovery, affected subscriptions only, genuine backend errors visible, no public contract change. Native34/Linux35 regression and official AST/strict lint/fmt evidence above support the candidate. Linux-specific native descriptor behavior is complemented by the all-platform unit boundary, not falsely claimed as native all-OS application acceptance. Original quality/retry/timeout/coverage criteria unchanged.
