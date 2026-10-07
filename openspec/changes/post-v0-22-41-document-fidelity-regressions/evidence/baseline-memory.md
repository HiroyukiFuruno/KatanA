# Empty-workspace memory baseline

## Build and launch condition

- Packaged KatanA Desktop v0.22.41
- Fresh temporary `KATANA_CONFIG_DIR`
- Empty workspace; no document opened
- macOS Apple Silicon

## Measured process footprint

- Resident set: 776,048 KiB
- Physical footprint: 734.7 MiB
- Peak physical footprint: 836.2 MiB
- IOSurface: 53.4 MiB
- CoreGraphics images: 8.2 MiB
- `MALLOC_LARGE` active: 423.1 MiB
- `MALLOC_LARGE` empty/resident: 200.3 MiB
- Heap live malloc bytes: 461,717,160
- Heap live non-object bytes: 456,066,731
- Child processes: none

## Ownership attribution

`vmmap` reported two active 187,632 KiB large allocations and one empty/resident
187,632 KiB allocation. `/System/Library/Fonts/Apple Color Emoji.ttc` is
192,123,488 bytes, matching those allocations after allocator rounding.

The startup call chain installs `FontDefinitions` once from `main.rs`, then the
first settings poll calls `ThemeBridgeOps::apply_font_family` with a standard
family and installs the same definitions again. The preset includes Apple Color
Emoji, while egui's default definitions already include compact emoji fallbacks.
This explains the two active payloads and the retained empty allocation.

Because no document was opened, the empty-workspace measurement owns no KDV/KRR
document worker, rendered frame, document texture, PDF page cache, Office
conversion artifact, or HTML browser session. The measured GPU allocations are
the baseline application surfaces above, not document-preview resources.

## Acceptance target

After excluding oversized UI emoji candidates and avoiding the redundant
standard-family reinstall, remeasure the packaged application with the same
launch condition. Emoji and CJK rendering must remain covered by deterministic
font-family tests.

## Fixed development-build measurement

The patched development binary was launched with a second fresh temporary
`KATANA_CONFIG_DIR` and an empty workspace. At 18 seconds and again at 61
seconds it measured:

- Resident set: 214,864 KiB
- Physical footprint: 176.9 MiB
- Peak physical footprint: 177.5 MiB
- IOSurface: 53.4 MiB
- CoreGraphics images: 8.2 MiB
- `MALLOC_LARGE` active: 53.1 MiB
- `MALLOC_LARGE` empty/resident: 4.0 MiB
- Heap live malloc bytes: 72,588,792

The 187,632 KiB allocations are absent. Compared with the packaged v0.22.41
baseline, RSS fell by 561,184 KiB and physical footprint fell by 557.8 MiB.
This confirms the owned-font root cause and fix locally; the packaged-app
memory gate was then exercised against the locally built release executable:

- Peak resident set: `241,328 KiB`, below the `524,288 KiB` gate.
- KatanA-owned font bytes: `27,161,036`, below the `134,217,728` gate.
- Office worker processes while idle: `0`.
- Startup heartbeat reached the idle marker within the configured timeout.

A subsequent release-binary smoke after the native-runner contract was added
reached first frame on arm64 with `248,864 KiB` peak RSS, `27,161,036` owned
font bytes, and zero Office workers. The release workflow now downloads the
same signed universal ZIP onto separate `macos-15` arm64 and
`macos-15-intel` x86_64 clean runners and executes the matching native slice;
Linux and Windows retain their own packaged startup checks. The static startup
contract and seven architecture/minimum-OS verifier tests pass locally. Actual
clean-runner execution remains required before tasks 1.6 and 6.2 can close.

After the dependency audit, the native arm64 release-binary smoke was repeated
with another fresh configuration directory. It reached first frame with
`243,360 KiB` peak RSS, `27,161,036` owned font bytes, and zero Office workers.

After the self-review responsibility split, both release binaries were rebuilt
from the current source. The fresh native arm64 smoke reached first frame with
`229,744 KiB` peak RSS, the same `27,161,036` owned font bytes, and zero Office
workers.

Task 4.1 is therefore locally verified. Clean-machine release artifacts on all
declared OS/CPU targets remain separately tracked by task 1.6/6.2.
