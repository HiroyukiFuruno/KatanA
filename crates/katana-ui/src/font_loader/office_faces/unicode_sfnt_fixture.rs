use skrifa::FontRef;

use super::sfnt_fixture::name_table_layout;

const NAME_RECORD_LENGTH: usize = 12;
const NAME_HEADER_LENGTH: usize = 6;
const FAMILY_NAME_ID: u16 = 1;
const TYPOGRAPHIC_FAMILY_NAME_ID: u16 = 16;
const PLATFORM_UNICODE: u16 = 3;
const NAME_ID_HIGH_OFFSET: usize = 6;
const NAME_ID_LOW_OFFSET: usize = 7;
const LENGTH_HIGH_OFFSET: usize = 8;
const LENGTH_LOW_OFFSET: usize = 9;
const STRING_OFFSET_HIGH_OFFSET: usize = 10;
const STRING_OFFSET_LOW_OFFSET: usize = 11;
const UTF16_CODE_UNIT_BYTES: usize = 2;

pub(super) fn with_ecole_family_aliases(mut bytes: Vec<u8>) -> Vec<u8> {
    with_family_aliases(&mut bytes, "École");
    bytes
}

pub(super) fn with_strasse_family_aliases(mut bytes: Vec<u8>) -> Vec<u8> {
    with_family_aliases(&mut bytes, "Straße");
    bytes
}

fn with_family_aliases(bytes: &mut [u8], family_name: &str) {
    let font = FontRef::new(bytes).expect("embedded font");
    let (table_offset, table, count, storage_offset) = name_table_layout(&font);
    for index in 0..count {
        let offset = NAME_HEADER_LENGTH + index * NAME_RECORD_LENGTH;
        let platform = u16::from_be_bytes([table[offset], table[offset + 1]]);
        let name_id = u16::from_be_bytes([
            table[offset + NAME_ID_HIGH_OFFSET],
            table[offset + NAME_ID_LOW_OFFSET],
        ]);
        if !matches!(name_id, FAMILY_NAME_ID | TYPOGRAPHIC_FAMILY_NAME_ID)
            || platform != PLATFORM_UNICODE
        {
            continue;
        }
        let length = u16::from_be_bytes([
            table[offset + LENGTH_HIGH_OFFSET],
            table[offset + LENGTH_LOW_OFFSET],
        ]) as usize;
        let string_offset = u16::from_be_bytes([
            table[offset + STRING_OFFSET_HIGH_OFFSET],
            table[offset + STRING_OFFSET_LOW_OFFSET],
        ]) as usize;
        assert!(
            length >= family_name.encode_utf16().count() * UTF16_CODE_UNIT_BYTES,
            "family name record is too short for the Unicode fixture"
        );
        assert!(length.is_multiple_of(UTF16_CODE_UNIT_BYTES));
        let mut encoded = family_name.encode_utf16().collect::<Vec<_>>();
        encoded.resize(length / UTF16_CODE_UNIT_BYTES, u16::from(b' '));
        let start = table_offset + storage_offset + string_offset;
        for (chunk, code_unit) in bytes[start..start + length]
            .chunks_exact_mut(UTF16_CODE_UNIT_BYTES)
            .zip(encoded)
        {
            chunk.copy_from_slice(&code_unit.to_be_bytes());
        }
    }
}
