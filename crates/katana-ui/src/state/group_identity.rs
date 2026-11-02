use super::document::TabGroup;

#[cfg(test)]
#[path = "group_identity_tests.rs"]
mod tests;

pub(crate) struct GroupIdentity;

impl GroupIdentity {
    pub(crate) fn conflicts(
        groups: &[TabGroup],
        exclude: Option<&str>,
        name: &str,
        color: &str,
    ) -> bool {
        groups.iter().any(|group| {
            Some(group.id.as_str()) != exclude
                && Self::same_identity(&group.name, &group.color_hex, name, color)
        })
    }

    pub(crate) fn same_identity(
        name: &str,
        color: &str,
        other_name: &str,
        other_color: &str,
    ) -> bool {
        name_key(name) == name_key(other_name) && colors_equal(color, other_color)
    }
}

fn name_key(name: &str) -> &str {
    if name.trim().is_empty() { "" } else { name }
}

fn colors_equal(left: &str, right: &str) -> bool {
    match (rgba(left), rgba(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => left == right,
    }
}

const RGBA_CHANNELS: usize = 4;
const HEX_RADIX: u32 = 16;
const SHORT_RGB: usize = 3;
const SHORT_RGBA: usize = 4;
const LONG_RGB: usize = 6;
const LONG_RGBA: usize = 8;
const NIBBLE_MASK: u32 = 15;
const NIBBLE_SCALE: u8 = 17;
const NIBBLE_BITS: u32 = 4;
const BYTE_BITS: u32 = 8;
const SHORT_RED_SHIFT: u32 = 12;
const LONG_RED_SHIFT: u32 = 16;

fn rgba(value: &str) -> Result<[u8; RGBA_CHANNELS], ()> {
    let digits = value.strip_prefix('#').ok_or(())?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(());
    }
    let number = u32::from_str_radix(digits, HEX_RADIX).map_err(|_| ())?;
    match digits.len() {
        SHORT_RGB => Ok([
            ((number >> BYTE_BITS) as u8) * NIBBLE_SCALE,
            (((number >> NIBBLE_BITS) & NIBBLE_MASK) as u8) * NIBBLE_SCALE,
            ((number & NIBBLE_MASK) as u8) * NIBBLE_SCALE,
            u8::MAX,
        ]),
        SHORT_RGBA => Ok([
            ((number >> SHORT_RED_SHIFT) as u8) * NIBBLE_SCALE,
            (((number >> BYTE_BITS) & NIBBLE_MASK) as u8) * NIBBLE_SCALE,
            (((number >> NIBBLE_BITS) & NIBBLE_MASK) as u8) * NIBBLE_SCALE,
            ((number & NIBBLE_MASK) as u8) * NIBBLE_SCALE,
        ]),
        LONG_RGB => Ok([
            (number >> LONG_RED_SHIFT) as u8,
            (number >> BYTE_BITS) as u8,
            number as u8,
            u8::MAX,
        ]),
        LONG_RGBA => Ok(number.to_be_bytes()),
        _ => Err(()),
    }
}
