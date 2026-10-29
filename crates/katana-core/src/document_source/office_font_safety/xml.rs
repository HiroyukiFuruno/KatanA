use super::OfficeFontSafetyError;
use super::attributes::FontAttributes;
use std::io::{Read, Take};
use xml::reader::{EventReader, ParserConfig, XmlEvent};

pub(super) struct FontXml;

impl FontXml {
    pub(super) fn visit(
        source: impl Read,
        part: &'static str,
        limit: u64,
        mut visitor: impl FnMut(XmlEvent) -> Result<(), OfficeFontSafetyError>,
    ) -> Result<(), OfficeFontSafetyError> {
        /* WHY: DOMと独自字句解析を避け、既存parserで正規化とstreaming検証を共通化する。 */
        let config = ParserConfig::new()
            .coalesce_characters(false)
            .max_entity_expansion_length(0)
            .max_entity_expansion_depth(0)
            .max_attribute_length(limit as usize)
            .max_data_length(limit as usize);
        let mut reader = config.create_reader(source.take(limit + 1));
        loop {
            let event = Self::next(&mut reader, part)?;
            if matches!(event, XmlEvent::EndDocument) {
                return Ok(());
            }
            visitor(event)?;
        }
    }

    fn next<R: Read>(
        reader: &mut EventReader<Take<R>>,
        part: &'static str,
    ) -> Result<XmlEvent, OfficeFontSafetyError> {
        let event = reader.next();
        if reader.source().limit() == 0 {
            return Err(OfficeFontSafetyError::metadata(
                part,
                "XML byte limit exceeded",
            ));
        }
        let event = event.map_err(|error| OfficeFontSafetyError::metadata(part, error))?;
        /* WHY: 標準文字参照は許可し、文書定義entityや外部参照はconsumer差分を作る前に拒否する。 */
        if matches!(event, XmlEvent::Doctype { .. }) {
            return Err(OfficeFontSafetyError::metadata(
                part,
                "DTD is not allowed in Office font metadata",
            ));
        }
        if let XmlEvent::StartElement { attributes, .. } = &event {
            FontAttributes::validate(part, attributes)?;
        }
        Ok(event)
    }
}
