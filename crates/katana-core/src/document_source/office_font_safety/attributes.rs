use super::OfficeFontSafetyError;
use std::collections::HashSet;
use xml::attribute::OwnedAttribute;

pub(super) struct FontAttributes;

impl FontAttributes {
    pub(super) fn unsafe_value(attributes: &[OwnedAttribute], local_name: &str) -> bool {
        attributes.iter().any(|attribute| {
            attribute.name.local_name == local_name
                && attribute.value.contains(['/', '\\', ':', '\0'])
        })
    }

    pub(super) fn validate(
        part: &'static str,
        attributes: &[OwnedAttribute],
    ) -> Result<(), OfficeFontSafetyError> {
        let mut names = HashSet::with_capacity(attributes.len());
        for attribute in attributes {
            if !names.insert((&attribute.name.prefix, &attribute.name.local_name)) {
                return Err(OfficeFontSafetyError::metadata(
                    part,
                    "duplicate XML attributes",
                ));
            }
        }
        Ok(())
    }
}
