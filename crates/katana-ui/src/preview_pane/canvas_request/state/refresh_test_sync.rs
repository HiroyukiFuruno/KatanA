use std::sync::atomic::Ordering;

use super::super::CanvasParseRequest;

const COMPLETION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(in crate::preview_pane) fn wait_until_completed(request: &CanvasParseRequest) {
    let deadline = std::time::Instant::now() + COMPLETION_TIMEOUT;
    /* WHY: 完了フラグは旧本文の読込・解析後に立つため、結果を消費せず更新競合を再現できる。 */
    while !request.request.completed.load(Ordering::Acquire) {
        assert!(
            std::time::Instant::now() < deadline,
            "Canvas worker did not complete"
        );
        std::thread::yield_now();
    }
}
