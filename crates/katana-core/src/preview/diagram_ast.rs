use crate::markdown::DiagramKind;
use crate::preview::DiagramSectionOps;
use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};
use katana_markdown_model::{
    CodeBlockRole as KmmCodeBlockRole, DiagramKind as KmmDiagramKind, KatanaMarkdownModel,
    KmmNodeKind, MarkdownInput,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramAstBlock {
    pub kind: DiagramKind,
    pub source: String,
}

pub struct DiagramAstBlockExtractor;

impl DiagramAstBlockExtractor {
    pub fn extract(source: &str) -> Vec<DiagramAstBlock> {
        Self::extract_with_kmm(source).unwrap_or_else(|| Self::extract_with_comrak(source))
    }

    fn extract_with_kmm(source: &str) -> Option<Vec<DiagramAstBlock>> {
        let document = KatanaMarkdownModel::parse(MarkdownInput::from_content("", source)).ok()?;
        Some(
            document
                .nodes
                .iter()
                .filter_map(Self::extract_kmm_node)
                .collect(),
        )
    }

    fn extract_kmm_node(node: &katana_markdown_model::KmmNode) -> Option<DiagramAstBlock> {
        let KmmNodeKind::CodeBlock(KmmCodeBlockRole::Diagram { kind }) = &node.kind else {
            return None;
        };
        let expected = Self::map_kmm_diagram_kind(kind);
        let (actual, source, _) =
            DiagramSectionOps::try_parse_diagram_fence(&node.source.raw.text)?;
        (actual == expected).then_some(DiagramAstBlock {
            kind: actual,
            source,
        })
    }

    fn map_kmm_diagram_kind(kind: &KmmDiagramKind) -> DiagramKind {
        match kind {
            KmmDiagramKind::Mermaid => DiagramKind::Mermaid,
            KmmDiagramKind::DrawIo => DiagramKind::DrawIo,
            KmmDiagramKind::PlantUml => DiagramKind::PlantUml,
        }
    }

    fn extract_with_comrak(source: &str) -> Vec<DiagramAstBlock> {
        let arena = Arena::new();
        let root = parse_document(&arena, source, &Options::default());
        root.descendants().filter_map(Self::extract_node).collect()
    }

    fn extract_node(node: &comrak::nodes::AstNode<'_>) -> Option<DiagramAstBlock> {
        let data = node.data.borrow();
        let NodeValue::CodeBlock(block) = &data.value else {
            return None;
        };
        if !block.fenced {
            return None;
        }
        let kind = DiagramKind::from_info(&block.info)?;
        if kind.should_preserve_fenced_source(&block.literal) {
            return None;
        }
        Some(DiagramAstBlock {
            kind,
            source: block.literal.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::nodes::NodeValue;
    use comrak::{Arena, Options, parse_document};

    #[test]
    fn extractor_returns_only_diagram_code_blocks() {
        let source = concat!(
            "```mermaid\n",
            "graph TD;A-->B\n",
            "```\n\n",
            "```rust\n",
            "fn main() {}\n",
            "```\n\n",
            "```drawio\n",
            "<mxGraphModel></mxGraphModel>\n",
            "```\n"
        );

        let blocks = DiagramAstBlockExtractor::extract(source);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].kind, DiagramKind::Mermaid);
        assert_eq!(blocks[0].source, "graph TD;A-->B");
        assert_eq!(blocks[1].kind, DiagramKind::DrawIo);
    }

    #[test]
    fn extractor_ignores_empty_mermaid_block() {
        let blocks = DiagramAstBlockExtractor::extract("```mermaid\n\n```\n");

        assert!(blocks.is_empty());
    }

    #[test]
    fn comrak_extractor_ignores_indented_code_blocks() {
        let arena = Arena::new();
        let root = parse_document(&arena, "    graph TD;A-->B\n", &Options::default());
        let code_block = root
            .descendants()
            .find(|node| matches!(node.data.borrow().value, NodeValue::CodeBlock(_)))
            .expect("code block exists");

        assert_eq!(DiagramAstBlockExtractor::extract_node(code_block), None);
    }

    #[test]
    fn comrak_extractor_ignores_empty_mermaid_fences() {
        let arena = Arena::new();
        let root = parse_document(&arena, "```mermaid\n\n```\n", &Options::default());
        let code_block = root
            .descendants()
            .find(|node| matches!(node.data.borrow().value, NodeValue::CodeBlock(_)))
            .expect("code block exists");

        assert_eq!(DiagramAstBlockExtractor::extract_node(code_block), None);
    }

    #[test]
    fn comrak_extractor_returns_diagram_block() {
        let arena = Arena::new();
        let root = parse_document(
            &arena,
            "```plantuml\n@startuml\nA -> B\n@enduml\n```\n",
            &Options::default(),
        );
        let code_block = root
            .descendants()
            .find(|node| matches!(node.data.borrow().value, NodeValue::CodeBlock(_)))
            .expect("code block exists");

        assert_eq!(
            DiagramAstBlockExtractor::extract_node(code_block),
            Some(DiagramAstBlock {
                kind: DiagramKind::PlantUml,
                source: "@startuml\nA -> B\n@enduml\n".to_string(),
            })
        );
    }
}
