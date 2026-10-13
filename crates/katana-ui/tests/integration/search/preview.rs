use katana_ui::app_state::ScrollState;
use katana_ui::preview_pane::PreviewPane;

#[test]
fn test_preview_search_scroll_sync_does_not_use_naive_row_height() {
    /* WHY: Verify that the preview pane doesn't use a simplistic 'line * row_height' logic
     * when scroll synchronization is active, as the preview has variable-height rich blocks
     * and must rely on the editor's anchors instead. */
    let mut scroll = ScrollState {
        scroll_to_line: Some(10),
        ..Default::default()
    };

    let preview = PreviewPane::default();

    let forced_offset =
        katana_ui::views::panels::preview::types::PreviewLogicOps::compute_forced_offset(
            true, // scroll_sync = true
            &mut scroll,
            &preview,
            20.0,   // row_height
            1000.0, // inner_height
        );

    assert!(
        forced_offset.is_none(),
        "Preview should not compute forced offset using naive row_height when scroll_sync is true!"
    );
}
