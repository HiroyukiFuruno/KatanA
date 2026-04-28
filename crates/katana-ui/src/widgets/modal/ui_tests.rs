#[cfg(test)]
mod tests {
    use crate::widgets::Modal;
    use crate::widgets::modal::ui::DEFAULT_BAR_WIDTH;

    #[test]
    fn test_modal_builder_defaults() {
        let modal = Modal::new("t", "Title");
        assert_eq!(modal.progress, None);
        assert!(!modal.show_pct);
        assert!((modal.bar_width - DEFAULT_BAR_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn test_modal_builder_with_progress() {
        let modal = Modal::new("t", "T")
            .progress(0.5)
            .show_percentage(true)
            .bar_width(200.0);
        assert_eq!(modal.progress, Some(0.5));
        assert!(modal.show_pct);
        assert!((modal.bar_width - 200.0).abs() < f32::EPSILON);
    }
}
