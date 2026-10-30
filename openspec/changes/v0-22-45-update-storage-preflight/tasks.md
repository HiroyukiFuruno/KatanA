# v0.22.45 Update Storage Preflight — Issue399

## 要件(Verbatim Requirements)

> 容量問題でした。strageの容量を確保後に更新できました。
> ただこれあれですね。更新前に空き容量を確認して、更新にはxxxMbyteのストレージの空き容量が必要です。ストレージの空き容量を確保してから、再度試して下さい。
> みたいな確認処理とメッセージ表示するように改善して？

### この要件から導出される制約(MUST)
- Check capacity before preparing the update and state required capacity with retry guidance.
- Treat insufficient capacity as the confirmed cause; do not substitute a GitHub outage explanation.

### 設計判断時の参照義務
Reference these requirements when changing budgets, UI wording, or retry handling.

## Definition of Ready

- Confirmed user recovery after freeing storage; reproduced old ENOSPC28 using the real public ZIP on an isolated APFS volume.
- Issue399 and current release/v0.22.45 checkout exist. No normal app/settings operations are needed.

## Branch Rule

Use the existing checkout and `release/v0.22.45` only. No new worktrees, normal stash, master edits, force pushes, hook bypass, or manual remote branch deletion. This follows the latest human policy over older per-task branch boilerplate.

## 1. Implementation and Targeted Validation

- [x] 1.1 Confirm public ZIP bundle/worker/architectures/checksum and old installer behavior.
- [x] 1.2 Preserve preparation stage and typed capacity/permission causes instead of stringifying into a generic check error.
- [x] 1.3 Check known download size before archive creation and inspected expansion/copy budget before extraction.
- [x] 1.4 Provide required/available MB and retry instructions in all10 locales.
- [x] 1.5 Collect final core/UI regressions and isolated low/sufficient-capacity results after review corrections.

Evidence: `tmp/update-check-failure/` contains real public-archive probes, constrained-volume ENOSPC and preflight rejection, core47, UI52 and final modal8 pass logs; corrected low-capacity probe rejects with retry-adjusted required1461218146/available333870120bytes, sufficient-capacity probe passes. Current-source AST23 passes. Full public/CI success is not inferred from local probes. PNG/layout/loader preview suite520 passes with the real Office worker; native cold-cache measurement remains separate and pending.

## 2. Self Review and Feedback

### Definition of Ready (DoR)
Implementation exists in this release checkout with target tests. No separate branch/PR cycle is added per subtask.

- [x] 2.1 Independent review identified and fixed unclassified IO detail loss and local-EOF network misclassification.
- [x] 2.2 Validate staging-space error numbers after temporary archive deletion so retry instructions describe pre-download capacity.
- [x] 2.3 Finish source/locale/AST/static checks and current independent review.

## 3. Final Verification & Release Work

### Definition of Ready (DoR)
All executable source/test feedback is addressed. User manual QA is a nonblocking follow-up, per explicit latest human policy.

- [ ] 3.1 Execute self-review against coding rules and keep normal concern-specific commits.
- [ ] 3.2 Format/lint updated documentation and synchronize version/changelog with the canonical release target.
- [ ] 3.3 Pass the normal push hook once as the complete local quality gate; do not duplicate its heavy checks manually.
- [ ] 3.4 Draft PR → full current reviews → repair/reply/resolve → fresh P0/P1 zero.
- [ ] 3.5 Ready → current required CI/coverage/security/3OS checks → normal merge.
- [ ] 3.6 Confirm public Release,5 payloads/checksums and external distribution outcomes; archive OpenSpec and safely clean local release state.

### Definition of Done (DoD)
Automated implementation/validation and the normal review/commit/PR/merge/publication sequence are evidenced. No user-only operation or impossible manual acceptance is added as an Issue Close condition.
