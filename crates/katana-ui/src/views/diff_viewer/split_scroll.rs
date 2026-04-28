use eframe::egui;

const H_EPS: f32 = 1.0;
const H_STRICT_RATIO: f32 = 5.0_f32;

#[derive(Debug, Clone, Copy)]
pub(super) struct SplitOffsets {
    pub(super) left: f32,
    pub(super) right: f32,
}

impl SplitOffsets {
    pub(super) fn load(ctx: &egui::Context, keys: &SplitScrollKeys) -> Self {
        Self {
            left: ctx
                .data(|data| data.get_temp::<f32>(keys.left))
                .unwrap_or(0.0_f32),
            right: ctx
                .data(|data| data.get_temp::<f32>(keys.right))
                .unwrap_or(0.0_f32),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SplitScrollKeys {
    left: egui::Id,
    right: egui::Id,
}

impl SplitScrollKeys {
    pub(super) fn new(file: &crate::diff_review::DiffReviewFile) -> Self {
        Self {
            left: egui::Id::new(("diff_viewer_h_left_offset", file.path.as_path())),
            right: egui::Id::new(("diff_viewer_h_right_offset", file.path.as_path())),
        }
    }
}

pub(super) fn suppress_diagonal_horizontal_scroll(ui: &egui::Ui, ctx: &egui::Context) {
    if !ui.rect_contains_pointer(ui.max_rect()) {
        return;
    }
    let delta = ctx.input(|input| input.smooth_scroll_delta);
    if delta.x.abs() <= delta.y.abs() * H_STRICT_RATIO {
        ctx.input_mut(|input| input.smooth_scroll_delta.x = 0.0);
    }
}

pub(super) fn sync_offsets(
    ctx: &egui::Context,
    keys: SplitScrollKeys,
    previous: SplitOffsets,
    left_x: f32,
    right_x: f32,
) {
    let left_delta = (left_x - previous.left).abs();
    let right_delta = (right_x - previous.right).abs();
    let next = next_offsets(previous, left_delta, right_delta, left_x, right_x);
    ctx.data_mut(|data| {
        data.insert_temp(keys.left, next.left);
        data.insert_temp(keys.right, next.right);
    });
}

fn next_offsets(
    previous: SplitOffsets,
    left_delta: f32,
    right_delta: f32,
    left_x: f32,
    right_x: f32,
) -> SplitOffsets {
    if left_delta > H_EPS && left_delta > right_delta {
        return SplitOffsets {
            left: left_x,
            right: left_x,
        };
    }
    if right_delta > H_EPS && right_delta > left_delta {
        return SplitOffsets {
            left: right_x,
            right: right_x,
        };
    }
    SplitOffsets {
        left: left_x.max(previous.left.min(left_x)),
        right: right_x,
    }
}
