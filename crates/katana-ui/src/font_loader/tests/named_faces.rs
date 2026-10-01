use super::*;
use egui::{FontData, FontFamily};
use skrifa::{
    FontRef, MetadataProvider, Tag,
    attribute::Style,
    raw::{TableProvider, tables::os2::SelectionFlags},
};
use std::sync::Arc;

#[test]
fn bold_and_italic_os2_faces_do_not_register_regular_aliases() {
    for (weight, selection, italic, bold) in
        [(700, 0x0020, false, true), (400, 0x0001, true, false)]
    {
        let mut fonts = ubuntu_face_with_os2(weight, selection);
        let bytes = fonts.font_data["Ubuntu-Light"].font.as_ref();
        assert_os2_metadata(bytes, weight, italic, bold);

        NamedFontFamiliesOps::register(&mut fonts);
        assert!(
            !fonts
                .families
                .contains_key(&FontFamily::Name("Ubuntu".into()))
        );
    }
}

fn ubuntu_face_with_os2(weight: u16, selection: u16) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let original = fonts
        .font_data
        .get("Ubuntu-Light")
        .expect("default Ubuntu font");
    let mut bytes = original.font.to_vec();
    let os2_offset = os2_table_offset(&bytes);
    bytes[os2_offset + 4..os2_offset + 6].copy_from_slice(&weight.to_be_bytes());
    bytes[os2_offset + 62..os2_offset + 64].copy_from_slice(&selection.to_be_bytes());
    fonts
        .font_data
        .insert("Ubuntu-Light".into(), Arc::new(FontData::from_owned(bytes)));
    fonts
}

fn assert_os2_metadata(bytes: &[u8], weight: u16, italic: bool, bold: bool) {
    let font = FontRef::new(bytes).expect("modified real Ubuntu font");
    let metadata = font.attributes();
    let os2 = font.os2().expect("Ubuntu OS/2 table");
    assert_eq!(metadata.weight.value(), f32::from(weight));
    assert_eq!(matches!(metadata.style, Style::Italic), italic);
    assert_eq!(os2.us_weight_class(), weight);
    assert_eq!(os2.fs_selection().contains(SelectionFlags::ITALIC), italic);
    assert_eq!(os2.fs_selection().contains(SelectionFlags::BOLD), bold);
}

fn os2_table_offset(bytes: &[u8]) -> usize {
    let font = FontRef::new(bytes).expect("default real Ubuntu font");
    font.table_directory()
        .table_records()
        .iter()
        .find(|record| record.tag() == Tag::new(b"OS/2"))
        .map(|record| record.offset() as usize)
        .expect("Ubuntu OS/2 table")
}
