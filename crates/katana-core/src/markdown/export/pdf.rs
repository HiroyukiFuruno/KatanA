use super::native_document::{NativeDocumentImage, NativeHtmlDocument};
use super::types::PdfExporter;
use crate::markdown::MarkdownError;

const IMAGE_SCALE_RATIO: f32 = 0.75;
const LETTER_PAGE_HEIGHT_RATIO: f32 = 11.0 / 8.5;

impl PdfExporter {
    pub fn is_available() -> bool {
        true
    }

    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        let document = NativeHtmlDocument::parse(html)?;
        let image = document.render_image()?;
        let pdf = NativePdfDocument::new(&image)?.to_bytes();
        std::fs::write(output, pdf).map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }
}

struct NativePdfDocument {
    page_width: f32,
    page_height: f32,
    image_display_height: f32,
    jpeg: Vec<u8>,
    image_width: u32,
    image_height: u32,
}

impl NativePdfDocument {
    fn new(image: &NativeDocumentImage) -> Result<Self, MarkdownError> {
        let page_width = image.width as f32 * IMAGE_SCALE_RATIO;
        Ok(Self {
            page_width,
            page_height: page_width * LETTER_PAGE_HEIGHT_RATIO,
            image_display_height: image.height as f32 * IMAGE_SCALE_RATIO,
            jpeg: image.jpeg_bytes()?,
            image_width: image.width,
            image_height: image.height,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let objects = self.objects();
        let mut output = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::with_capacity(objects.len());
        for (index, object) in objects.iter().enumerate() {
            offsets.push(output.len());
            output.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
            output.extend_from_slice(object);
            output.extend_from_slice(b"\nendobj\n");
        }
        let xref_offset = output.len();
        output.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        output.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets {
            output.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        output.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        output
    }

    fn objects(&self) -> Vec<Vec<u8>> {
        let page_count = self.page_count();
        let image_object_id = Self::image_object_id(page_count);
        let mut objects = vec![
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            self.pages_object(page_count),
        ];
        for page_index in 0..page_count {
            objects.push(self.page_object(page_count, page_index, image_object_id));
        }
        objects.push(self.image_object());
        for page_index in 0..page_count {
            objects.push(self.content_object(page_index));
        }
        objects
    }

    fn page_count(&self) -> usize {
        (self.image_display_height / self.page_height).ceil().max(1.0) as usize
    }

    fn pages_object(&self, page_count: usize) -> Vec<u8> {
        let kids = (0..page_count)
            .map(|page_index| format!("{} 0 R", Self::page_object_id(page_index)))
            .collect::<Vec<_>>()
            .join(" ");
        format!("<< /Type /Pages /Kids [{kids}] /Count {page_count} >>").into_bytes()
    }

    fn page_object(
        &self,
        page_count: usize,
        page_index: usize,
        image_object_id: usize,
    ) -> Vec<u8> {
        let content_object_id = Self::content_object_id(page_count, page_index);
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /XObject << /Im1 {} 0 R >> >> /Contents {} 0 R >>",
            self.page_width, self.page_height, image_object_id, content_object_id
        )
        .into_bytes()
    }

    fn image_object(&self) -> Vec<u8> {
        let mut object = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
            self.image_width,
            self.image_height,
            self.jpeg.len()
        )
        .into_bytes();
        object.extend_from_slice(&self.jpeg);
        object.extend_from_slice(b"\nendstream");
        object
    }

    fn content_object(&self, page_index: usize) -> Vec<u8> {
        let image_y = self.page_height - self.image_display_height
            + self.page_height * page_index as f32;
        let commands = format!(
            "q\n0 0 {:.2} {:.2} re\nW\nn\n{:.2} 0 0 {:.2} 0 {:.2} cm\n/Im1 Do\nQ\n",
            self.page_width, self.page_height, self.page_width, self.image_display_height, image_y
        );
        let mut object = format!("<< /Length {} >>\nstream\n", commands.len()).into_bytes();
        object.extend_from_slice(commands.as_bytes());
        object.extend_from_slice(b"endstream");
        object
    }

    fn page_object_id(page_index: usize) -> usize {
        page_index + 3
    }

    fn image_object_id(page_count: usize) -> usize {
        page_count + 3
    }

    fn content_object_id(page_count: usize, page_index: usize) -> usize {
        Self::image_object_id(page_count) + page_index + 1
    }
}
