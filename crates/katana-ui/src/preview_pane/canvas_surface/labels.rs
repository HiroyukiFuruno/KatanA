pub(crate) struct CanvasSurfaceLabels<'a> {
    pub fit: &'a str,
    pub empty: &'a str,
    pub unsupported_node: &'a str,
    pub unsupported_fields: &'a str,
    pub unsupported_link: &'a str,
    pub unsupported_background: &'a str,
    pub load_error: &'a str,
    pub loading: &'a str,
}

impl<'a> CanvasSurfaceLabels<'a> {
    pub(crate) fn new(messages: &'a crate::i18n::CanvasMessages) -> Self {
        Self {
            fit: &messages.fit,
            empty: &messages.empty,
            unsupported_node: &messages.unsupported_node,
            unsupported_fields: &messages.unsupported_fields,
            unsupported_link: &messages.unsupported_link,
            unsupported_background: &messages.unsupported_background,
            load_error: &messages.load_error,
            loading: &messages.loading,
        }
    }
}
