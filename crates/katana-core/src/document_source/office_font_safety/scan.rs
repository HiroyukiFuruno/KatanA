use super::attributes::FontAttributes;
use super::{BinaryDocumentFormat, FontParts, OfficeFontSafetyError};
use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;

pub(super) struct FontScan<'a> {
    parts: &'a FontParts,
    in_font: bool,
    unsafe_name: bool,
    embedded: bool,
}

impl<'a> FontScan<'a> {
    pub(super) fn new(parts: &'a FontParts) -> Self {
        Self {
            parts,
            in_font: false,
            unsafe_name: false,
            embedded: false,
        }
    }

    pub(super) fn visit(&mut self, event: XmlEvent) -> Result<(), OfficeFontSafetyError> {
        match event {
            XmlEvent::StartElement {
                name, attributes, ..
            } => self.start(&name.local_name, &attributes)?,
            XmlEvent::EndElement { name } if name.local_name == self.wrapper() => self.finish()?,
            _ => {}
        }
        Ok(())
    }

    fn wrapper(&self) -> &'static str {
        if self.parts.format == BinaryDocumentFormat::Docx {
            "font"
        } else {
            "embeddedFont"
        }
    }

    fn start(
        &mut self,
        name: &str,
        attributes: &[OwnedAttribute],
    ) -> Result<(), OfficeFontSafetyError> {
        if name != self.wrapper() {
            if self.in_font {
                self.inspect_face(name, attributes);
            }
            return Ok(());
        }
        /* WHY: 入れ子wrapperのconsumer選択差で危険名がsafe値に上書きされるのを防ぐ。 */
        if self.in_font {
            return Err(OfficeFontSafetyError::metadata(
                self.parts.metadata,
                "nested embedded font wrapper",
            ));
        }
        self.in_font = true;
        self.embedded = false;
        self.unsafe_name = self.parts.format == BinaryDocumentFormat::Docx
            && FontAttributes::unsafe_value(attributes, "name");
        Ok(())
    }

    fn inspect_face(&mut self, name: &str, attributes: &[OwnedAttribute]) {
        if self.parts.format == BinaryDocumentFormat::Pptx && name == "font" {
            /* WHY: 同じface内の複数候補をORで保持し、safe値で先行候補を消さない。 */
            self.unsafe_name |= FontAttributes::unsafe_value(attributes, "typeface");
            return;
        }
        /* WHY: GUIDやrIdの解決失敗を安全の根拠にせず、consumerが読むface markerで拒否する。 */
        self.embedded |= match self.parts.format {
            BinaryDocumentFormat::Docx => matches!(
                name,
                "embedRegular" | "embedBold" | "embedItalic" | "embedBoldItalic"
            ),
            _ => matches!(name, "regular" | "bold" | "italic" | "boldItalic"),
        };
    }

    fn finish(&mut self) -> Result<(), OfficeFontSafetyError> {
        self.in_font = false;
        if self.unsafe_name && self.embedded {
            return Err(OfficeFontSafetyError::UnsafeFontName {
                part: self.parts.metadata,
            });
        }
        Ok(())
    }
}
