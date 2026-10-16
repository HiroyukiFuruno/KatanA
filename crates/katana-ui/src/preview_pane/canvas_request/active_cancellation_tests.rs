use super::tests::{RequestTest, TEST_TIMEOUT};
use super::{CanvasParseRequest, pool::CanvasPool};
use katana_core::canvas::NodeContent;
use katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES;
use std::time::Instant;

#[test]
fn maximum_memory_request_cancellation_does_not_block_next_real_model() {
    let source = String::from_utf8(vec![b' '; MAX_BINARY_DOCUMENT_BYTES]).expect("ASCII source");
    let pool = CanvasPool::start().expect("worker start");
    let request = CanvasParseRequest::submit(&pool, 1, source, None).expect("maximum source");
    RequestTest::wait_active(&pool, &request.request);
    let start = Instant::now();
    drop(request);
    let source = r#"{"nodes":[{"id":"next","type":"text","x":0,"y":0,"width":1,"height":1,"text":"Ready"}]}"#;
    let followup = RequestTest::submit(&pool, 2, source);
    let result = RequestTest::receive(&followup);
    let document = result.result.expect("follow-up real model");
    assert_eq!(result.generation, 2);
    assert_eq!(document.nodes[0].id, "next");
    assert_eq!(
        document.nodes[0].content,
        NodeContent::Text {
            text: "Ready".into()
        }
    );
    assert!(start.elapsed() < TEST_TIMEOUT);
    println!(
        "maximum_input_bytes={} cancel_to_next_model_us={}",
        MAX_BINARY_DOCUMENT_BYTES,
        start.elapsed().as_micros()
    );
}
