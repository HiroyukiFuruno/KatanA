use xml::attribute::OwnedAttribute;

pub(super) struct FontAttributes;

impl FontAttributes {
    pub(super) fn unsafe_value(attributes: &[OwnedAttribute], local_name: &str) -> bool {
        attributes.iter().any(|attribute| {
            attribute.name.local_name == local_name
                && attribute.value.contains(['/', '\\', ':', '\0'])
        })
    }
}
