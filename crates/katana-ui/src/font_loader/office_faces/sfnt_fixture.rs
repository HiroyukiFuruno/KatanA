use egui::FontDefinitions;
use skrifa::{FontRef, MetadataProvider, string::StringId};

const NAME_RECORD_LENGTH: usize = 12;
const NAME_HEADER_LENGTH: usize = 6;
const NAME_COUNT_HIGH_OFFSET: usize = 2;
const NAME_COUNT_LOW_OFFSET: usize = 3;
const STORAGE_OFFSET_HIGH_OFFSET: usize = 4;
const STORAGE_OFFSET_LOW_OFFSET: usize = 5;
const FAMILY_NAME_ID: u16 = 1;
const TYPOGRAPHIC_FAMILY_NAME_ID: u16 = 16;
const SUBFAMILY_NAME_ID: u16 = 17;
const PLATFORM_UNICODE: u16 = 3;
const LANGUAGE_HIGH_OFFSET: usize = 4;
const LANGUAGE_LOW_OFFSET: usize = 5;
const NAME_ID_HIGH_OFFSET: usize = 6;
const NAME_ID_LOW_OFFSET: usize = 7;
const LENGTH_HIGH_OFFSET: usize = 8;
const LENGTH_LOW_OFFSET: usize = 9;
const STRING_OFFSET_HIGH_OFFSET: usize = 10;
const STRING_OFFSET_LOW_OFFSET: usize = 11;
const LANGUAGE_HIGH_BYTE: u8 = 0x04;
const ENGLISH_LANGUAGE_LOW_BYTE: u8 = 0x09;
const JAPANESE_LANGUAGE_LOW_BYTE: u8 = 0x11;
const UTF16_HIGH_BYTE: u8 = 0;
const JAPANESE_DAY_HIGH_BYTE: u8 = 0x65;
const JAPANESE_DAY_LOW_BYTE: u8 = 0xe5;
const ASCII_SPACE: u8 = b' ';
const UTF16_CODE_UNIT_BYTES: usize = 2;

pub(super) fn embedded_font() -> Vec<u8> {
    FontDefinitions::default().font_data["Ubuntu-Light"]
        .font
        .to_vec()
}

pub(super) fn preferred_family_name(font: &FontRef<'_>) -> Option<String> {
    [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
        .into_iter()
        .find_map(|id| {
            font.localized_strings(id)
                .english_or_first()
                .map(|record| record.to_string().trim().to_owned())
        })
}

pub(super) fn family_names_for_id(font: &FontRef<'_>, id: StringId) -> Vec<String> {
    font.localized_strings(id)
        .map(|record| record.to_string().trim().to_owned())
        .filter(|name| !name.is_empty() && !name.chars().any(char::is_control))
        .collect()
}

pub(super) fn with_distinct_family_aliases(mut bytes: Vec<u8>) -> Vec<u8> {
    let font = FontRef::new(&bytes).expect("embedded font");
    let (table_offset, table, count, storage_offset) = name_table_layout(&font);
    let mut localized_family_added = false;
    let mut english_family_added = false;
    for index in 0..count {
        let offset = NAME_HEADER_LENGTH + index * NAME_RECORD_LENGTH;
        let platform = u16::from_be_bytes([table[offset], table[offset + 1]]);
        let name_id = name_id(&table, offset);
        if !matches!(
            name_id,
            FAMILY_NAME_ID | TYPOGRAPHIC_FAMILY_NAME_ID | SUBFAMILY_NAME_ID
        ) {
            continue;
        }
        let length = record_length(&table, offset);
        let string_offset = string_offset(&table, offset);
        let start = table_offset + storage_offset + string_offset;
        if length == 0 || start + length > bytes.len() {
            continue;
        }
        if name_id == SUBFAMILY_NAME_ID
            && platform == PLATFORM_UNICODE
            && !english_family_added
            && length >= UTF16_CODE_UNIT_BYTES
        {
            bytes[table_offset + offset + LANGUAGE_HIGH_OFFSET] = LANGUAGE_HIGH_BYTE;
            bytes[table_offset + offset + LANGUAGE_LOW_OFFSET] = ENGLISH_LANGUAGE_LOW_BYTE;
            bytes[table_offset + offset + NAME_ID_HIGH_OFFSET] = UTF16_HIGH_BYTE;
            bytes[table_offset + offset + NAME_ID_LOW_OFFSET] = FAMILY_NAME_ID as u8;
            bytes[start] = UTF16_HIGH_BYTE;
            bytes[start + 1] = b'L';
            for chunk in bytes[start + UTF16_CODE_UNIT_BYTES..start + length]
                .chunks_exact_mut(UTF16_CODE_UNIT_BYTES)
            {
                chunk.copy_from_slice(&[UTF16_HIGH_BYTE, ASCII_SPACE]);
            }
            english_family_added = true;
            continue;
        }
        if name_id == FAMILY_NAME_ID
            && platform == PLATFORM_UNICODE
            && !localized_family_added
            && length >= UTF16_CODE_UNIT_BYTES
        {
            bytes[table_offset + offset + LANGUAGE_HIGH_OFFSET] = LANGUAGE_HIGH_BYTE;
            bytes[table_offset + offset + LANGUAGE_LOW_OFFSET] = JAPANESE_LANGUAGE_LOW_BYTE;
            bytes[start] = JAPANESE_DAY_HIGH_BYTE;
            bytes[start + 1] = JAPANESE_DAY_LOW_BYTE;
            for chunk in bytes[start + UTF16_CODE_UNIT_BYTES..start + length]
                .chunks_exact_mut(UTF16_CODE_UNIT_BYTES)
            {
                chunk.copy_from_slice(&[UTF16_HIGH_BYTE, ASCII_SPACE]);
            }
            localized_family_added = true;
            continue;
        }
        let marker = if name_id == FAMILY_NAME_ID {
            b'L'
        } else {
            b'T'
        };
        if platform == PLATFORM_UNICODE && length >= UTF16_CODE_UNIT_BYTES {
            bytes[start] = UTF16_HIGH_BYTE;
            bytes[start + 1] = marker;
        } else if platform == 1 {
            bytes[start] = marker;
        }
    }
    bytes
}

pub(super) fn name_table_layout(font: &FontRef<'_>) -> (usize, Vec<u8>, usize, usize) {
    let record = font
        .table_directory()
        .table_records()
        .iter()
        .find(|record| record.tag() == skrifa::Tag::new(b"name"))
        .expect("name table");
    let table = font
        .table_data(skrifa::Tag::new(b"name"))
        .expect("name table data")
        .as_bytes()
        .to_owned();
    let count =
        u16::from_be_bytes([table[NAME_COUNT_HIGH_OFFSET], table[NAME_COUNT_LOW_OFFSET]]) as usize;
    let storage_offset = u16::from_be_bytes([
        table[STORAGE_OFFSET_HIGH_OFFSET],
        table[STORAGE_OFFSET_LOW_OFFSET],
    ]) as usize;
    (record.offset() as usize, table, count, storage_offset)
}

fn name_id(table: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([
        table[offset + NAME_ID_HIGH_OFFSET],
        table[offset + NAME_ID_LOW_OFFSET],
    ])
}

fn record_length(table: &[u8], offset: usize) -> usize {
    u16::from_be_bytes([
        table[offset + LENGTH_HIGH_OFFSET],
        table[offset + LENGTH_LOW_OFFSET],
    ]) as usize
}

fn string_offset(table: &[u8], offset: usize) -> usize {
    u16::from_be_bytes([
        table[offset + STRING_OFFSET_HIGH_OFFSET],
        table[offset + STRING_OFFSET_LOW_OFFSET],
    ]) as usize
}
