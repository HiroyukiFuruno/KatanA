use super::{ResizeHistory, ResizeIdentity};

const BASE_WIDTH: u32 = 1024;
const BASE_HEIGHT: u32 = 697;
const BASE_DPI: u32 = 96;

fn identity() -> ResizeIdentity {
    ResizeIdentity {
        kind: 0,
        width: BASE_WIDTH,
        height: BASE_HEIGHT,
        dpi: BASE_DPI,
        monitor: 1,
    }
}

#[test]
fn first_resize_passes_and_identical_notification_is_suppressed() {
    let mut history = ResizeHistory::default();
    assert!(!history.suppress(Some(identity())));
    assert!(history.suppress(Some(identity())));
}

#[test]
fn changed_width_and_height_are_forwarded() {
    let mut history = ResizeHistory::default();
    history.suppress(Some(identity()));
    assert!(!history.suppress(Some(ResizeIdentity {
        width: 1025,
        ..identity()
    })));
    assert!(!history.suppress(Some(ResizeIdentity {
        height: 698,
        ..identity()
    })));
}

#[test]
fn maximize_and_restore_are_forwarded_at_identical_dimensions() {
    let mut history = ResizeHistory::default();
    history.suppress(Some(identity()));
    assert!(!history.suppress(Some(ResizeIdentity {
        kind: 2,
        ..identity()
    })));
    assert!(!history.suppress(Some(identity())));
}

#[test]
fn changed_dpi_and_monitor_are_forwarded() {
    let mut history = ResizeHistory::default();
    history.suppress(Some(identity()));
    assert!(!history.suppress(Some(ResizeIdentity {
        dpi: 144,
        ..identity()
    })));
    assert!(!history.suppress(Some(ResizeIdentity {
        monitor: 2,
        ..identity()
    })));
}

#[test]
fn minimize_and_zero_size_invalidate_the_previous_identity() {
    for invalid in [
        ResizeIdentity {
            kind: 1,
            ..identity()
        },
        ResizeIdentity {
            width: 0,
            ..identity()
        },
        ResizeIdentity {
            height: 0,
            ..identity()
        },
    ] {
        let mut history = ResizeHistory::default();
        history.suppress(Some(identity()));
        assert!(!history.suppress(Some(invalid)));
        assert!(!history.suppress(Some(invalid)));
        assert!(!history.suppress(Some(identity())));
    }
}

#[test]
fn unavailable_native_identity_is_never_suppressed() {
    let mut history = ResizeHistory::default();
    history.suppress(Some(identity()));
    assert!(!history.suppress(None));
    assert!(!history.suppress(Some(identity())));
    for invalid in [
        ResizeIdentity {
            dpi: 0,
            ..identity()
        },
        ResizeIdentity {
            monitor: 0,
            ..identity()
        },
        ResizeIdentity {
            kind: 3,
            ..identity()
        },
        ResizeIdentity {
            kind: 4,
            ..identity()
        },
    ] {
        assert!(!history.suppress(Some(invalid)));
        assert!(!history.suppress(Some(invalid)));
    }
}

#[test]
fn explicit_native_invalidation_forwards_the_next_same_size() {
    let mut history = ResizeHistory::default();
    history.suppress(Some(identity()));
    history.invalidate();
    assert!(!history.suppress(Some(identity())));
}
