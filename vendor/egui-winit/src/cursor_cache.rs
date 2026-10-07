use egui::CustomCursorImage;
use std::sync::Arc;

pub(super) struct CursorImageKey(CustomCursorImage);

impl CursorImageKey {
    pub(super) fn new(image: &CustomCursorImage) -> Self {
        Self(image.clone())
    }

    pub(super) fn matches(&self, image: &CustomCursorImage) -> bool {
        Arc::ptr_eq(&self.0.rgba, &image.rgba)
            && self.0.size == image.size
            && self.0.hotspot == image.hotspot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> CustomCursorImage {
        CustomCursorImage {
            rgba: Arc::from([255_u8; 16]),
            size: [2, 2],
            hotspot: [0, 0],
        }
    }

    #[test]
    fn shared_bitmap_and_geometry_reuse_key() {
        let image = image();
        assert!(CursorImageKey::new(&image).matches(&image.clone()));
    }

    #[test]
    fn shared_bitmap_with_changed_hotspot_does_not_reuse_key() {
        let image = image();
        let key = CursorImageKey::new(&image);
        let mut changed = image.clone();
        changed.hotspot = [1, 1];
        assert!(!key.matches(&changed));
    }

    #[test]
    fn shared_bitmap_with_changed_dimensions_does_not_reuse_key() {
        let image = image();
        let key = CursorImageKey::new(&image);
        let mut changed = image.clone();
        changed.size = [1, 4];
        assert!(!key.matches(&changed));
    }

    #[test]
    fn separately_allocated_equal_pixels_do_not_reuse_key() {
        let image = image();
        let changed = CustomCursorImage {
            rgba: Arc::from(image.rgba.to_vec()),
            ..image.clone()
        };
        assert!(!CursorImageKey::new(&image).matches(&changed));
    }

    #[test]
    fn key_retains_bitmap_until_evicted() {
        let image = image();
        let weak = Arc::downgrade(&image.rgba);
        let key = CursorImageKey::new(&image);
        drop(image);
        assert!(weak.upgrade().is_some());
        drop(key);
        assert!(weak.upgrade().is_none());
    }
}
