use katana_core::markdown::DiagramThemeSnapshot;

const MIN_TABLE_PIXELS: usize = 80;
const MIN_BORDER_PIXELS: usize = 20;
const TABLE_COLOR_ROLES: usize = 3;

pub(super) struct TableThemeColors;

impl TableThemeColors {
    pub(super) fn entries(
        theme: &DiagramThemeSnapshot,
    ) -> [(&str, usize, &str); TABLE_COLOR_ROLES] {
        let header = theme
            .table_header_background
            .as_deref()
            .expect("header token");
        let stripe = theme
            .table_even_row_background
            .as_deref()
            .expect("stripe token");
        let border = theme.table_border.as_deref().expect("border token");
        [
            (header, MIN_TABLE_PIXELS, "table header"),
            (stripe, MIN_TABLE_PIXELS, "table stripe"),
            (border, MIN_BORDER_PIXELS, "table border"),
        ]
    }
}
