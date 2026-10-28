# 3機能の新H実行証拠

- reviewed H: `a6584bcef524d4d26d42f5176b0cb343ceca0599` / merge M: `8c89a1544e9ac693d5ba388792a843baa55c9363`
- 両tree: `07a3931a8227a74a46afea095f29d9c83b8eb7f2`
- 実行はCI40660526599のsynthetic merge2aa64b46（H/Mと同一tree）。Mの実行へ名称を付け替えない。正式CD40672472894の全build/native/Publishと全5公開実取得が成功。cleanupだけ失敗/run全体failureはrootで別追跡する。

| テスト | Windows | Linux | macOS |
| --- | --- | --- | --- |
| local_png_is_decoded_off_request_and_composited | PASS `tmp/oct28-a6584bce-windows-ci.log:5859` | PASS `tmp/oct28-a6584bce-linux-ci.log:6204` | PASS `tmp/oct28-a6584bce-macos-ci.log:3609` |
| reset_discards_old_generation_result | PASS `tmp/oct28-a6584bce-windows-ci.log:5861` | PASS `tmp/oct28-a6584bce-linux-ci.log:6206` | PASS `tmp/oct28-a6584bce-macos-ci.log:3611` |
| missing_parent_recovers_after_directory_and_png_are_created | PASS `tmp/oct28-a6584bce-windows-ci.log:5855` | PASS `tmp/oct28-a6584bce-linux-ci.log:6200` | PASS `tmp/oct28-a6584bce-macos-ci.log:3605` |
| failed_decode_retries_at_watch_deadline_without_watch_success | PASS `tmp/oct28-a6584bce-windows-ci.log:5840` | PASS `tmp/oct28-a6584bce-linux-ci.log:6185` | PASS `tmp/oct28-a6584bce-macos-ci.log:3590` |
| invalidate_path_does_not_leave_fifo_decode_blocking_followup | NOT RUN (Unix限定) | PASS `tmp/oct28-a6584bce-linux-ci.log:6180` | PASS `tmp/oct28-a6584bce-macos-ci.log:3585` |
| split_background_uses_explorer_window_role_only_for_markdown | PASS `tmp/oct28-a6584bce-windows-ci.log:6273` | PASS `tmp/oct28-a6584bce-linux-ci.log:6621` | PASS `tmp/oct28-a6584bce-macos-ci.log:4027` |
| focused_page_arrow_events_dispatch_viewer_commands_for_pdf_and_pptx | PASS `tmp/oct28-a6584bce-windows-ci.log:5654` | PASS `tmp/oct28-a6584bce-linux-ci.log:5994` | PASS `tmp/oct28-a6584bce-macos-ci.log:3398` |
| page_navigation_ignores_unfocused_widgets_other_formats_modifiers_and_boundaries | PASS `tmp/oct28-a6584bce-windows-ci.log:5657` | PASS `tmp/oct28-a6584bce-linux-ci.log:5995` | PASS `tmp/oct28-a6584bce-macos-ci.log:3399` |

旧H0464/Meb12のraw証拠は保持し、今回の実行と区別する。PPTX15秒・Office95点・配布版受入の成功へ拡大しない。公開後のClose/archiveは実証跡で判定し、cleanupの未完はroot台帳で継続する。
