use crate::shell_ui::TreeRenderContext;

pub(crate) struct TreeContextMenu<'a, 'b, 'c> {
    pub path: &'a std::path::Path,
    pub is_dir: bool,
    pub children: Option<&'a [katana_core::workspace::TreeEntry]>,
    pub entry: Option<&'a katana_core::workspace::TreeEntry>,
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

pub struct TreeLogicOps;
