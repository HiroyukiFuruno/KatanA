use base64::Engine;
use include_dir::{Dir, include_dir};
use serde::Serialize;
use std::collections::BTreeSet;

static DRAWIO_RESOURCE_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/src/markdown/drawio_renderer/js_runtime/resources");

pub(super) struct DrawioResourceCatalog;

impl DrawioResourceCatalog {
    pub(super) fn builtin(source: &str) -> Vec<DrawioResource> {
        let selector = DrawioResourceSelector::new(source);
        let mut resources = Vec::new();
        collect_resources(&DRAWIO_RESOURCE_DIR, &selector, &mut resources);
        resources
    }
}

#[derive(Serialize)]
pub(super) struct DrawioResource {
    path: String,
    mime_type: &'static str,
    content: String,
    encoding: DrawioResourceEncoding,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum DrawioResourceEncoding {
    Text,
    Base64,
}

struct DrawioResourceSelector<'a> {
    source: &'a str,
    groups: BTreeSet<String>,
    uses_drawio_shape: bool,
}

impl<'a> DrawioResourceSelector<'a> {
    fn new(source: &'a str) -> Self {
        let groups = extract_resource_groups(source);
        let uses_drawio_shape = source.contains("shape=mxgraph.");
        Self {
            source,
            groups,
            uses_drawio_shape,
        }
    }

    fn includes(&self, path: &str) -> bool {
        path == "stencils/basic.xml"
            || self.includes_stencil(path)
            || self.includes_shape_script(path)
            || self.includes_image(path)
    }

    fn includes_stencil(&self, path: &str) -> bool {
        let Some(relative_path) = path.strip_prefix("stencils/") else {
            return false;
        };
        if !relative_path.ends_with(".xml") {
            return false;
        }
        self.groups.iter().any(|group| {
            relative_path == format!("{group}.xml")
                || relative_path.starts_with(&format!("{group}/"))
        })
    }

    fn includes_shape_script(&self, path: &str) -> bool {
        path.starts_with("shapes/") && self.uses_drawio_shape
    }

    fn includes_image(&self, path: &str) -> bool {
        is_image_resource(path) && self.source_references(path)
    }

    fn source_references(&self, path: &str) -> bool {
        self.source.contains(path) || self.source.contains(&format!("/{path}"))
    }
}

fn collect_resources(
    dir: &Dir<'_>,
    selector: &DrawioResourceSelector<'_>,
    resources: &mut Vec<DrawioResource>,
) {
    for file in dir.files() {
        let path = file.path().to_string_lossy();
        if selector.includes(path.as_ref()) {
            resources.push(drawio_resource(path.into_owned(), file.contents()));
        }
    }
    for child in dir.dirs() {
        collect_resources(child, selector, resources);
    }
}

fn drawio_resource(path: String, contents: &[u8]) -> DrawioResource {
    let encoding = encoding_for_path(&path);
    let mime_type = mime_type_for_path(&path);
    let content = resource_content(contents, &encoding);
    DrawioResource {
        path,
        mime_type,
        content,
        encoding,
    }
}

fn resource_content(contents: &[u8], encoding: &DrawioResourceEncoding) -> String {
    match encoding {
        DrawioResourceEncoding::Text => String::from_utf8_lossy(contents).into_owned(),
        DrawioResourceEncoding::Base64 => {
            base64::engine::general_purpose::STANDARD.encode(contents)
        }
    }
}

fn encoding_for_path(path: &str) -> DrawioResourceEncoding {
    if path.ends_with(".xml") || path.ends_with(".js") {
        return DrawioResourceEncoding::Text;
    }
    DrawioResourceEncoding::Base64
}

fn mime_type_for_path(path: &str) -> &'static str {
    if path.ends_with(".xml") {
        return "text/xml";
    }
    if path.ends_with(".js") {
        return "application/javascript";
    }
    if path.ends_with(".svg") {
        return "image/svg+xml";
    }
    if path.ends_with(".png") {
        return "image/png";
    }
    if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        return "image/jpeg";
    }
    if path.ends_with(".gif") {
        return "image/gif";
    }
    "application/octet-stream"
}

fn is_image_resource(path: &str) -> bool {
    path.ends_with(".svg")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
}

fn extract_resource_groups(source: &str) -> BTreeSet<String> {
    let mut groups = BTreeSet::new();
    let mut remaining = source;
    while let Some(index) = remaining.find("shape=mxgraph.") {
        remaining = &remaining[index + "shape=mxgraph.".len()..];
        if let Some(prefix) = drawio_prefix(remaining) {
            groups.extend(resource_groups(prefix));
        }
    }
    groups
}

fn drawio_prefix(value: &str) -> Option<&str> {
    let prefix = value
        .split(['.', ';', '"', '\'', '&', ' '])
        .next()
        .unwrap_or_default();
    (!prefix.is_empty()).then_some(prefix)
}

fn resource_groups(prefix: &str) -> Vec<String> {
    match prefix.to_ascii_lowercase().as_str() {
        "arrows2" => vec!["arrows".to_string()],
        "ios" | "ios7" | "ios7ui" => vec!["ios7".to_string()],
        "pid2misc" | "pid2valves" => vec!["pid2".to_string()],
        "rackgeneral" => vec!["rack".to_string()],
        "veeam2" => vec!["veeam".to_string()],
        other => vec![other.to_string()],
    }
}
