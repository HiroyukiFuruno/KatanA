use katana_ui::app_state::ScrollState;
use katana_ui::preview_pane::PreviewPane;

#[test]
fn test_preview_search_scroll_sync_does_not_use_naive_row_height() {
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
            20.0, // row_height
        );

    /* WHY: If it uses the naive row_height calculation, it returns Some(200.0)
     * We want to verify it does NOT aggressively guess the offset if scroll_sync is enabled,
     * so it should return None and let the editor handle it.
     * Assert that forced_offset is None.
     */
    assert!(
        forced_offset.is_none(),
        "Preview should not compute forced offset using naive row_height when scroll_sync is true!"
    );
}
