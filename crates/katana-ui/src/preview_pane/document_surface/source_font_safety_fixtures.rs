/* WHY: 実ZIP/XMLを生成する責務を分け、source入口の回帰テストを読みやすく保つ。 */
const CRC32_BITS_PER_BYTE: usize = 8;
const CRC32_POLYNOMIAL: u32 = 0xedb8_8320;
const ZIP_LOCAL_FILE_SIGNATURE: u32 = 0x0403_4b50;
const ZIP_CENTRAL_DIRECTORY_SIGNATURE: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY_SIGNATURE: u32 = 0x0605_4b50;
const ZIP_VERSION: u16 = 20;
const ZIP_CENTRAL_DIRECTORY_OFFSET_FIELD_COUNT: usize = 4;
struct ZipEntry {
    name: Vec<u8>,
    crc: u32,
    size: u32,
    offset: u32,
}
fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut value = u32::MAX;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..CRC32_BITS_PER_BYTE {
            value = if value & 1 == 1 {
                (value >> 1) ^ CRC32_POLYNOMIAL
            } else {
                value >> 1
            };
        }
    }
    !value
}
fn append_local_file(zip: &mut Vec<u8>, name: &str, content: &str) -> ZipEntry {
    let name = name.as_bytes().to_vec();
    let content = content.as_bytes();
    let size = u32::try_from(content.len()).expect("small fixture content");
    let entry = ZipEntry {
        crc: crc32(content),
        size,
        offset: u32::try_from(zip.len()).expect("small fixture archive"),
        name,
    };
    write_u32(zip, ZIP_LOCAL_FILE_SIGNATURE);
    for value in [ZIP_VERSION, 0, 0, 0, 0] {
        write_u16(zip, value);
    }
    write_u32(zip, entry.crc);
    write_u32(zip, size);
    write_u32(zip, size);
    write_u16(
        zip,
        u16::try_from(entry.name.len()).expect("short entry name"),
    );
    write_u16(zip, 0);
    zip.extend_from_slice(&entry.name);
    zip.extend_from_slice(content);
    entry
}
fn append_central_entry(zip: &mut Vec<u8>, entry: &ZipEntry) {
    write_u32(zip, ZIP_CENTRAL_DIRECTORY_SIGNATURE);
    for value in [ZIP_VERSION, ZIP_VERSION, 0, 0, 0, 0] {
        write_u16(zip, value);
    }
    write_u32(zip, entry.crc);
    write_u32(zip, entry.size);
    write_u32(zip, entry.size);
    write_u16(
        zip,
        u16::try_from(entry.name.len()).expect("short entry name"),
    );
    for _ in 0..ZIP_CENTRAL_DIRECTORY_OFFSET_FIELD_COUNT {
        write_u16(zip, 0);
    }
    write_u32(zip, 0);
    write_u32(zip, entry.offset);
    zip.extend_from_slice(&entry.name);
}
pub(super) fn office_zip(entries: &[(&str, &str)]) -> Vec<u8> {
    /* WHY: 圧縮なしの実ZIP構造でsource入口からXMLを読む回帰を保つ。 */
    let mut zip = Vec::new();
    let entries = entries
        .iter()
        .map(|(name, content)| append_local_file(&mut zip, name, content))
        .collect::<Vec<_>>();
    let central_offset = u32::try_from(zip.len()).expect("small fixture archive");
    for entry in &entries {
        append_central_entry(&mut zip, entry);
    }
    let central_size = u32::try_from(zip.len()).expect("small fixture archive") - central_offset;
    write_u32(&mut zip, ZIP_END_OF_CENTRAL_DIRECTORY_SIGNATURE);
    let count = u16::try_from(entries.len()).expect("few fixture entries");
    for value in [0, 0, count, count] {
        write_u16(&mut zip, value);
    }
    write_u32(&mut zip, central_size);
    write_u32(&mut zip, central_offset);
    write_u16(&mut zip, 0);
    zip
}
pub(super) fn docx_with_font(name: &str) -> Vec<u8> {
    const FONT_KEY: &str = "00112233-4455-6677-8899-aabbccddeeff";
    let xml = format!(
        r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:font w:name="{name}"><w:embedRegular w:fontKey="{{{FONT_KEY}}}" r:id="rId1"/></w:font></w:fonts>"#
    );
    let relationships =
        r#"<Relationships><Relationship Id="rId1" Target="fontdata.bin"/></Relationships>"#;
    office_zip(&[
        ("word/fontTable.xml", &xml),
        ("word/_rels/fontTable.xml.rels", relationships),
        ("word/fontdata.bin", "font bytes"),
    ])
}
pub(super) fn pptx_with_font(name: &str) -> Vec<u8> {
    let xml = format!(
        r#"<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><a:embeddedFontLst><a:embeddedFont><a:font typeface="{name}"/><a:regular r:id="rId1"/></a:embeddedFont></a:embeddedFontLst></p:presentation>"#
    );
    let relationships = r#"<Relationships><Relationship Id="rId1" Target="{00112233-4455-6677-8899-aabbccddeeff}.fntdata"/></Relationships>"#;
    office_zip(&[
        ("ppt/presentation.xml", &xml),
        ("ppt/_rels/presentation.xml.rels", relationships),
        (
            "ppt/{00112233-4455-6677-8899-aabbccddeeff}.fntdata",
            "font bytes",
        ),
    ])
}
