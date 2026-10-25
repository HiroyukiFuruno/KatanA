# User feedback: next-release startup recovery

The user reported macOS detecting a failed launch and offering Reopen after a
workspace hierarchy containing many Office/PDF files had been opened. Removing
per-workspace JSON files and excluding the heavy workspace from startup targets
restored launch. Both interventions changed state, so their individual causal
contributions are not established. A memory leak or specific upstream defect is
not proven, and current source fixes have not proven this scenario resolved.

- [ ] Obtain actual crash/termination evidence and phase/resource measurements
  for affected fresh-process startup/session restore. Distinguish crash, OS
  resource termination, worker failure, directory enumeration, retained memory
  and allocator high-water rather than labeling all failures as leaks.
- [ ] Next-release acceptance includes heavy persisted Office/PDF session and
  workspace cold startup, normal exit, restart and original resource budgets.
  In-process tests and successful configuration reset are insufficient.
- [ ] KatanA owns restoration, bounded automatic work, failure isolation and
  recovery UX. Diagnose confirmed KDV/KRR faults and hand them off, without
  editing sibling repositories in this task.
- [ ] Evaluate failed-start detection and a restricted launch that skips risky
  restoration while retaining settings, with visible retry/disable choices.
  Safe Mode is a candidate design, not immediate implementation approval.
- [ ] A verified cause fix plus affected restart acceptance may make Safe Mode
  non-urgent. Repeated startup failure without a recoverable launch cannot be
  indefinitely deferred. User settings/GUI/Terms actions are not delegated.

This feedback is explicitly for the next release; it does not silently expand
the current v0.22.42 release scope or turn upstream defects into current blockers.
Antipattern: reset success -> declare a fixed leak. Required sequence: actual
failure evidence -> root-cause repair -> fresh persisted-session regression ->
recovery assessment. Keywords: macOS Reopen, crash loop, heavy workspace,
Office PDF restore, safe mode, memory leak unconfirmed.
