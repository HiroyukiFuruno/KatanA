const RESTORED: usize = 0;
const MAXIMIZED: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResizeIdentity {
    pub(super) kind: usize,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) dpi: u32,
    pub(super) monitor: isize,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ResizeHistory {
    last: Option<ResizeIdentity>,
}

impl ResizeHistory {
    pub(super) fn invalidate(&mut self) {
        self.last = None;
    }

    pub(super) fn suppress(&mut self, identity: Option<ResizeIdentity>) -> bool {
        /* WHY: 同じ通知でもGPU同期を伴う再設定が起きるため、表示条件が同じ場合だけ抑える。 */
        let identity = identity.filter(|value| {
            matches!(value.kind, RESTORED | MAXIMIZED)
                && value.width > 0
                && value.height > 0
                && value.dpi > 0
                && value.monitor != 0
        });
        let duplicate = identity.is_some() && self.last == identity;
        self.last = identity;
        duplicate
    }
}

#[cfg(test)]
mod tests;
