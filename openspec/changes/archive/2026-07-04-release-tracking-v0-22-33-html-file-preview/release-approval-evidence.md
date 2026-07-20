# v0.22.33 Release Approval Evidence

## Release Boundary

- KatanA release target: `v0.22.33` only.
- Allowed adjacent update: `v0.22.32 -> v0.22.33` only.
- Withdrawn target: `v0.29.0`; no release or tag exists for it.
- Runtime: KRR in-process Rust/V8 only.
- Forbidden runtime paths: Chromium, Chrome, WebView, external browser helpers,
  and browser runtime archives.
- KDV owns only the worker-backed KRR session adapter. KatanA owns source
  acquisition, complete-frame display, input forwarding, navigation/history,
  and reload.
- KatanA remained uncommitted, unpushed, unpublished, and unreleased until the
  user explicitly approved the evidence below on 2026-07-21.

## Published Dependency Chain

- KRR `0.4.3` is published and verified from crates.io and GitHub Release.
  It provides the in-process Rust/V8 HTML runtime, system-font fallback, and
  complete fragment-origin handling used by this evidence.
- KDV `0.3.1` is published and verified from crates.io and GitHub Release. It
  coalesces adjacent scroll and resize commands without reordering input and
  remains a KRR session adapter only.
- KatanA `Cargo.lock` and `scripts/screenshot/Cargo.lock` resolve KDV `0.3.1`
  and KRR `0.4.3` from crates.io with registry checksums.
- KatanA declares KDV `0.3.1` and KRR `0.4.3` as its minimum compatible
  registry versions; the release guard rejects the stale `0.3.0` / `0.4.0`
  manifest requirements even when a lock happens to contain newer patches.
- KatanA has no KDV/KRR path dependency, git dependency, or crates.io patch.

## Headless Acceptance Evidence

- Date: 2026-07-21.
- Execution: headless egui/KatanA process harness.
- Native KatanA window, Chromium, Chrome, and WebView were not started. Docker
  and Colima are not part of the acceptance execution.
- Request:
  `scripts/screenshot/examples/v0-22-33-html-headless-preview.json`.
  The release runner has no native-window execution branch.
- The harness used the published crates.io dependency chain directly with
  `--locked`; no local dependency injection or Cargo source override was used.
- Output:
  `tmp/v0-22-33-html-headless-registry-20260721-release-candidate/`.
- Result: **60/60 steps passed**.
- All 11 release-candidate PNG files are byte-for-byte identical to the
  screenshots approved by the user: ImageMagick reported `AE=0` and `RMSE=0`
  for every capture, and every SHA-256 remained unchanged after the final
  persistent-session/history and refresh-coalescing fixes.
- The fixture loads relative `style.css` and `actions.js` from the original
  file document origin. Inline-only HTML cannot satisfy this evidence.
- RGB click targeting derives physical search bounds from the live HTML surface
  rectangle and clips them to the captured image. The release contract rejects
  fixed HTML click bounds, so explorer or toolbar colors cannot satisfy an HTML
  interaction step after layout or DPI changes.

### Visual States

| Capture | Contract evidence |
| --- | --- |
| `01-initial-render.png` | External CSS and startup V8 script are visible; the initial semantic region matched 100,081 pixels. |
| `02-accordion-open.png` | Native details state and the V8 click listener changed the complete frame; the opened semantic region matched 126,431 pixels and 369,660 pixels changed. |
| `03-button-action.png` | Button listener mutated DOM and style; the click target matched 25,338 pixels and the post-action semantic region matched 101,621 pixels. |
| `04-text-input.png` | Focused input received `日本語 IME入力`; the input target matched 23,652 pixels and the Japanese post-input semantic region matched 102,133 pixels without tofu. |
| `05-prevented-navigation.png` | `preventDefault()` changed the DOM; the link target matched 99,492 pixels, the post-action region matched 99,878 pixels, and `index.html` stayed active. |
| `05-scrolled-content.png` | Explicit document scroll reached the KRR tail target, which matched 126,219 pixels; `index.html` stayed active and the return scroll restored the controls. |
| `06-fragment-navigation.png` | KRR handled `#fragment-target` inside the existing session, preserved `日本語 IME入力`, reported the complete origin ending in `index.html#fragment-target`, matched 123,671 raw-frame pixels and 123,979 composed pixels, and kept `index.html` active. |
| `07-link-navigation.png` | KRR-confirmed relative navigation activated `linked-panel.html#linked-target`; complete origin matched and both raw and composed target regions matched 871,190 pixels. |
| `08-reloaded-linked-panel.png` | Reload retained `linked-panel.html#linked-target`; complete origin and both 871,190-pixel raw/composed target regions remained intact. |
| `09-resized-linked-panel.png` | Resize retained `linked-panel.html#linked-target`; complete origin matched, with 680,390 raw-frame and composed pixels proving target alignment after reflow. |

Original-resolution inspection found no overlapping or clipped text, tofu
glyphs, leaked metadata/style/script source, missing unchanged content, or
partial-frame damage.

### SHA-256

- `01-initial-render.png`:
  `7fa977c95566df7de109958b1c24a1b195faac3e61962868650ccf8172e774f6`
- `02-accordion-open.png`:
  `4887fcc4eb5fa3491861c5cdc4e9ee39f215a5ecdf3f07bb15973ce818769732`
- `03-button-action.png`:
  `4e8a898ac2dc5d9fef969cafbdb8ba70d74a4e0b314e9a116f16fb8f72a15b12`
- `04-text-input.png`:
  `ae1004983971dbec01d0425eb255ab3ce9f4730af4f951ffef5a775c3c6a259c`
- `05-prevented-navigation.png`:
  `3e88195a249685728b942447ec15c9cb1503d8cdc6022709838d49d6972c2e13`
- `05-scrolled-content.png`:
  `9314b58c1041149d2018bbfaab9b0e1556925e28d67f20fd2089f94585d6e715`
- `06-fragment-navigation.png`:
  `017dd80eee8a824c39910ecd64bc4c0ceba40d9699c252c3bc958629ec87dd03`
- `07-link-navigation.png`:
  `924bebd99e2c6203d837738110b8944ea695373908d544e00bfe83235cb3a9a7`
- `08-reloaded-linked-panel.png`:
  `23474b931a903f7ab4fe4c2d522b1d791446553dd0cdd06d66db374c39f6f24c`
- `09-resized-linked-panel.png`:
  `d2714766af2432757a6ddeae3d487adee165529ff08c402f968304185caf3d0c`

Captures `01` through `08` are 2560x1600 physical pixels. The resized capture
`09` is 2200x1400 physical pixels.

## Mechanical Evidence

- KRR `just VERSION=0.4.3 release-check`: strict Clippy, AST lint, 100% line
  coverage with zero uncovered lines, package-size gate, package verification,
  and publish dry-run passed. GitHub Release and crates.io publication are
  verified.
- KDV `just VERSION=v0.3.1 release-check`: 1,579 tests passed and 1 ignored;
  strict Clippy, AST lint, 20,801/20,801 production lines covered, package
  verification, and publish dry-run passed. GitHub Release and crates.io
  publication are verified.
- `cargo test -p katana-ui image_html_surface --locked`: 16 passed. Coverage
  includes physical-pixel mapping, complete-frame scaling, typed-error
  handling, stale-frame removal after runtime errors, mutating-input repaint,
  outside-surface pointer release, focus loss, egui-to-browser scroll
  direction, committed IME text, preedit de-duplication, persistent adapter
  reuse, and per-tab navigation history.
- `cargo test -p katana-ui html_navigation --locked`: 2 passed. The production
  document replacement path moves the existing KDV browser session to the
  navigated path and retains the initial and target origins in one tab history.
- `cargo test -p katana-ui html_preview_observer --locked`: 7 passed. Watcher
  events remain queued while another app action is pending, and rapid saves
  replace one deferred refresh with the latest saved source.
- `cargo test -p katana-ui url_source --locked`: 6 passed. Coverage includes
  raw HTML and final redirect origin preservation plus invalid redirect
  rejection.
- `cargo test --manifest-path scripts/screenshot/Cargo.toml --locked`: 14
  passed. Coverage includes live HTML-surface physical bounds and image-edge
  clipping for deterministic RGB targeting.
- `scripts/release/test-html-browser-release-contract.sh`: passed and
  separately rejects
  stale KDV `0.3.0` and KRR `0.4.0` manifest requirements, a stale KDV `0.3.0`
  lock, and a stale KRR `0.4.2` lock while accepting only KDV `0.3.1` / KRR
  `0.4.3` minimums in both application and headless harness locks. It also
  rejects missing crates.io checksums, an external browser source marker
  anywhere in KatanA UI, an external browser dependency in a runtime manifest,
  and fixed HTML click search bounds in release evidence.
- `scripts/release/check-html-browser-release-contract.sh 0.22.33` currently
  passes with the registry-only KDV `0.3.1` / KRR `0.4.3` locks. Its negative
  cases still reject stale patch floors, external browser sources, runtime
  manifests with external-browser dependencies, and fixed HTML click bounds.
- `just coverage`: passed with the unchanged strict meaningful-line gate. No
  threshold reduction or coverage exclusion was added.
- `cargo check --workspace --locked`: passed.
- `just lint`: passed with zero Clippy issues.
- `just ast-lint`: 23 tests passed.
- `just check-linux`: passed in the headless Colima Docker environment; Colima
  was stopped after the check.
- `scripts/release/preflight.sh 0.22.33` and
  `GITHUB_ACTIONS=true scripts/release/check-pr-ready.sh 0.22.33`: passed. The
  SemVer guard accepts only `v0.22.32 -> v0.22.33`, and the branch guard
  confirms `release/v0.22.33`.
- `git diff --check`: passed.

## Release Status

- GitHub Release/tag `v0.22.33`: absent.
- Withdrawn GitHub Release/tag `v0.29.0`: absent.
- PR #320: open and unchanged by this local recovery work.
- Native-window validation was not started. The release evidence uses the
  deterministic headless harness. Docker/Colima was used only for the Linux
  workspace test and was stopped afterward.
- The published KRR `0.4.3` / KDV `0.3.1` registry-only rerun is complete and
  the pixel-identical release-candidate captures above are the final KatanA
  release-approval evidence.
- **Approval status: approved by the user on 2026-07-21.**

## Superseded Evidence

Earlier static-parser, static-image, Chromium, and native-window captures are
rejection history only. They do not satisfy the current release contract and
are intentionally excluded from this approval record.
