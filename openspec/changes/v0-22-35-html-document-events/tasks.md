# Tasks: v0.22.35 HTML document events

## Branch Rule

- KRR: `release/v0.4.5`
- KDV: `release/v0.3.3`
- KatanA: `release/v0.22.35`
- 公開順序: KRR v0.4.5 -> KDV v0.3.3 -> KatanA v0.22.35

## 1. KRR v0.4.5

- [x] 1.1 `document` / `window` EventTarget、`readyState`、`DOMContentLoaded`、microtask、`load` を実装する
- [x] 1.2 lifecycle mutation、listener options、handler property、listener error location の回帰テストを追加する
- [x] 1.3 DOM runtime JS を formatter/linter と crates.io package gate の対象にする
- [x] 1.4 617 tests、strict line coverage 100% / uncovered 0、package verify、publish dry-run を通す
- [x] 1.5 KRR v0.4.5 PR CI、merge、GitHub Release、runtime/CLI crates.io 公開を確認する

## 2. KDV v0.3.3

- [x] 2.1 公開済み KRR v0.4.5 を registry checksum 付きで解決する
- [x] 2.2 KDV が HTML semantics を持たず browser-session adapter に留まる contract を通す
- [x] 2.3 full check、strict coverage、release-check、publish dry-run を通す
- [x] 2.4 PR CI、merge、GitHub Release、crates.io 公開を確認する

## 3. KatanA v0.22.35

- [x] 3.1 公開済み KDV v0.3.3 / KRR v0.4.5 を workspace と screenshot runner で registry-only 解決する
- [x] 3.2 SemVer guard が v0.22.34 -> v0.22.35 のみを許可し、v0.29.0 と非隣接版を拒否する
- [x] 3.3 `DOMContentLoaded` で初期化する headless fixture に accordion、button、IME input、fragment、別文書 navigation を通す
- [x] 3.4 HTML runtime error が stack または script location を含むことを KatanA surface まで検証する
- [x] 3.5 check-full、strict coverage、OpenSpec strict validation、release-preflight readiness を通す
- [x] 3.6 headless screenshots と machine assertions を保存して描画・操作を確認する

Release execution is tracked separately because push, CI, publication, and
post-release verification can only run after this preflight succeeds.

| Step | State |
| --- | --- |
| Push `release/v0.22.35` with normal hooks | Pending |
| Pass required PR CI and merge | Pending |
| Verify GitHub Release v0.22.35, assets, latest release, and absence of v0.29.0 | Pending |
| Verify local/remote branch and worktree hygiene | Pending |

## 4. User feedback ledger

- [x] 4.1 `JavaScript expecption: TypeError: document.addEventListener is not a function` を根本解決する

`4.2` の未完了停止禁止、機械検証、headless 実機証跡、release 完了は上記の
release execution table と本タスクの最終完了条件として追跡する。
