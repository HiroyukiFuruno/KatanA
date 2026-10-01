use super::*;
use egui::{FontData, FontFamily};
use std::sync::Arc;
use ttf_parser::{Face, Tag};

#[test]
fn bold_and_italic_os2_faces_do_not_register_regular_aliases() {
    for (weight, selection, italic, bold) in
        [(700, 0x0020, false, true), (400, 0x0001, true, false)]
    {
        let mut fonts = ubuntu_face_with_os2(weight, selection);
        let bytes = fonts.font_data["Ubuntu-Light"].font.as_ref();
        let face = Face::parse(bytes, 0).expect("modified real Ubuntu font");
        assert_eq!(face.weight().to_number(), weight);
        assert_eq!(face.is_italic(), italic);
        assert_eq!(face.is_bold(), bold);

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

fn os2_table_offset(bytes: &[u8]) -> usize {
    let face = Face::parse(bytes, 0).expect("default real Ubuntu font");
    face.raw_face()
        .table_records
        .into_iter()
        .find(|record| record.tag == Tag::from_bytes(b"OS/2"))
        .expect("Ubuntu OS/2 table")
        .offset as usize
}
