use katana_core::markdown::mermaid_renderer;
use katana_core::markdown::{
    MarkdownRenderOps,
    diagram::{DiagramBlock, DiagramKind, DiagramResult},
};
use std::sync::Mutex;

static I18N_RENDER_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn renders_sample_mermaid_ja_without_diagram_errors() {
    let _guard = I18N_RENDER_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping i18n Mermaid regression");
        return;
    }

    let output = MarkdownRenderOps::render_with_katana_renderer(include_str!(
        "../../../assets/fixtures/sample_mermaid_ja.md"
    ))
    .unwrap();

    assert!(
        !output.html.contains("katana-diagram-error"),
        "localized Mermaid fixture must render without fallback errors"
    );
    for expected_text in [
        "\u{30af}\u{30ea}\u{30b9}\u{30de}\u{30b9}",
        "\u{8cb7}\u{3044}\u{7269}\u{306b}\u{884c}\u{304f}",
        "\u{79c1}\u{306e}\u{4ed5}\u{4e8b}\u{306e}\u{31}\u{65e5}",
        "\u{30dc}\u{30e9}\u{30f3}\u{30c6}\u{30a3}\u{30a2}\u{306b}\u{5f15}\u{304d}\u{53d6}\u{3089}\u{308c}\u{305f}\u{30da}\u{30c3}\u{30c8}",
        "\u{9867}\u{5ba2}",
        "\u{4e2d}\u{592e}\u{306e}\u{5e83}\u{3044}\u{30d6}\u{30ed}\u{30c3}\u{30af}",
        "\u{8fb2}\u{696d}\u{5ec3}\u{68c4}\u{7269}",
        "\u{706b}\u{529b}\u{767a}\u{96fb}",
        "\u{30c7}\u{30fc}\u{30bf}\u{30d9}\u{30fc}\u{30b9}",
        "\u{30c6}\u{30a3}\u{30fc}\u{30b7}\u{30e7}\u{30c3}\u{30d7}",
    ] {
        assert!(
            output.html.contains(expected_text),
            "localized Mermaid output should keep visible text: {expected_text}"
        );
    }
    for english_text in [
        "Christmas",
        "Go shopping",
        "My working day",
        "Pets adopted by volunteers",
        "A wide one in the middle",
        "Blurry Photo",
    ] {
        assert!(
            !output.html.contains(english_text),
            "localized Mermaid fixture should not keep English sample text: {english_text}"
        );
    }
}

#[test]
fn renders_non_ascii_diagram_syntax_as_visible_text() {
    let _guard = I18N_RENDER_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping i18n Mermaid regression");
        return;
    }

    for (name, source, expected_text) in localized_diagrams() {
        let svg = render_source(name, source);
        assert!(
            svg.contains(expected_text),
            "{name} should preserve visible localized text: {expected_text}"
        );
    }
}

#[test]
fn sankey_i18n_keeps_repeated_labels_as_single_nodes() {
    let _guard = I18N_RENDER_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping i18n Mermaid regression");
        return;
    }

    let svg = render_source(
        "localized sankey repeated labels",
        concat!(
            "sankey-beta\n",
            "\u{8fb2}\u{696d}\u{5ec3}\u{68c4}\u{7269},\u{751f}\u{7269}\u{5909}\u{63db},124.729\n",
            "\u{751f}\u{7269}\u{5909}\u{63db},\u{6db2}\u{4f53}\u{71c3}\u{6599},0.597\n",
            "\u{751f}\u{7269}\u{5909}\u{63db},\u{640d}\u{5931},26.862\n",
            "\u{751f}\u{7269}\u{5909}\u{63db},\u{56fa}\u{4f53}\u{71c3}\u{6599},280.322\n",
            "\u{751f}\u{7269}\u{5909}\u{63db},\u{30ac}\u{30b9},81.144\n",
            "\u{6d77}\u{85fb},\u{751f}\u{7269}\u{5909}\u{63db},4.375\n",
            "\u{305d}\u{306e}\u{4ed6}\u{5ec3}\u{68c4}\u{7269},\u{751f}\u{7269}\u{5909}\u{63db},77.81",
        ),
    );

    assert!(svg.contains("\u{8fb2}\u{696d}\u{5ec3}\u{68c4}\u{7269}"));
    assert_eq!(
        svg.matches(r#"<g class="node""#).count(),
        8,
        "Sankey labels must keep node identity after i18n normalization:\n{svg}"
    );
    assert!(
        !svg.contains(r#"height="0""#),
        "Sankey nodes collapsed after i18n normalization:\n{svg}"
    );
}

#[test]
fn wardley_i18n_accepts_compact_non_ascii_arrows() {
    let _guard = I18N_RENDER_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping i18n Mermaid regression");
        return;
    }

    let svg = render_source(
        "localized wardley compact arrows",
        concat!(
            "wardley-beta\n",
            "title \u{30ec}\u{30f3}\u{30c0}\u{30e9}\u{30fc}\u{5c0e}\u{5165}\n",
            "anchor \u{30e6}\u{30fc}\u{30b6}\u{30fc} [0.95, 0.62]\n",
            "component \u{30d7}\u{30ec}\u{30d3}\u{30e5}\u{30fc} [0.78, 0.55]\n",
            "component MermaidJS [0.62, 0.42]\n",
            "\u{30e6}\u{30fc}\u{30b6}\u{30fc}->\u{30d7}\u{30ec}\u{30d3}\u{30e5}\u{30fc}\n",
            "\u{30d7}\u{30ec}\u{30d3}\u{30e5}\u{30fc}->MermaidJS",
        ),
    );

    assert!(svg.contains("\u{30ec}\u{30f3}\u{30c0}\u{30e9}\u{30fc}\u{5c0e}\u{5165}"));
    assert!(svg.contains("\u{30e6}\u{30fc}\u{30b6}\u{30fc}"));
    assert!(svg.contains("\u{30d7}\u{30ec}\u{30d3}\u{30e5}\u{30fc}"));
}

fn localized_diagrams() -> [(&'static str, &'static str, &'static str); 7] {
    [
        (
            "er",
            concat!(
                "erDiagram\n",
                "\u{9867}\u{5ba2} ||--o{ \u{6ce8}\u{6587} : \u{6ce8}\u{6587}\u{3059}\u{308b}\n",
                "\u{6ce8}\u{6587} ||--|{ \u{6ce8}\u{6587}\u{660e}\u{7d30} : \u{542b}\u{3080}\n",
                "\u{9867}\u{5ba2} {\n",
                "string \u{6c0f}\u{540d}\n",
                "}",
            ),
            "\u{6ce8}\u{6587}\u{660e}\u{7d30}",
        ),
        (
            "requirement",
            concat!(
                "requirementDiagram\n",
                "requirement \u{30c6}\u{30b9}\u{30c8}\u{8981}\u{6c42} {\n",
                "id: 1\n",
                "text: \u{30c6}\u{30b9}\u{30c8}\u{7528}\u{306e}\u{8981}\u{6c42}\u{3002}\n",
                "risk: high\n",
                "verifymethod: test\n",
                "}\n",
                "element \u{30c6}\u{30b9}\u{30c8}\u{5bfe}\u{8c61} {\n",
                "type: \u{30b7}\u{30df}\u{30e5}\u{30ec}\u{30fc}\u{30b7}\u{30e7}\u{30f3}\n",
                "}\n",
                "\u{30c6}\u{30b9}\u{30c8}\u{5bfe}\u{8c61} - satisfies -> \u{30c6}\u{30b9}\u{30c8}\u{8981}\u{6c42}",
            ),
            "\u{30c6}\u{30b9}\u{30c8}\u{5bfe}\u{8c61}",
        ),
        (
            "quadrant",
            concat!(
                "quadrantChart\n",
                "title \u{30ad}\u{30e3}\u{30f3}\u{30da}\u{30fc}\u{30f3}\u{306e}\u{5230}\u{9054}\u{3068}\u{53cd}\u{5fdc}\n",
                "x-axis \u{4f4e}\u{5230}\u{9054} --> \u{9ad8}\u{5230}\u{9054}\n",
                "y-axis \u{4f4e}\u{53cd}\u{5fdc} --> \u{9ad8}\u{53cd}\u{5fdc}\n",
                "quadrant-1 \u{62e1}\u{5927}\u{3059}\u{3079}\u{304d}\n",
                "\u{30ad}\u{30e3}\u{30f3}\u{30da}\u{30fc}\u{30f3}A: [0.3, 0.6]",
            ),
            "\u{30ad}\u{30e3}\u{30f3}\u{30da}\u{30fc}\u{30f3}\u{306e}\u{5230}\u{9054}\u{3068}\u{53cd}\u{5fdc}",
        ),
        (
            "xychart",
            concat!(
                "xychart-beta\n",
                "title \"\u{58f2}\u{4e0a}\"\n",
                "x-axis [1\u{6708}, 2\u{6708}]\n",
                "y-axis \"\u{58f2}\u{4e0a}\u{ff08}\u{5186}\u{ff09}\" 4000 --> 11000\n",
                "bar [5000, 6000]",
            ),
            "\u{58f2}\u{4e0a}\u{ff08}\u{5186}\u{ff09}",
        ),
        (
            "sankey",
            concat!(
                "sankey-beta\n",
                "\u{8fb2}\u{696d}\u{5ec3}\u{68c4}\u{7269},\u{751f}\u{7269}\u{5909}\u{63db},124.729\n",
                "\u{751f}\u{7269}\u{5909}\u{63db},\u{6db2}\u{4f53}\u{71c3}\u{6599},0.597",
            ),
            "\u{8fb2}\u{696d}\u{5ec3}\u{68c4}\u{7269}",
        ),
        (
            "architecture",
            "architecture-beta\ngroup api(cloud)[API]\nservice db(database)[\u{30c7}\u{30fc}\u{30bf}\u{30d9}\u{30fc}\u{30b9}] in api",
            "\u{30c7}\u{30fc}\u{30bf}\u{30d9}\u{30fc}\u{30b9}",
        ),
        (
            "wardley",
            concat!(
                "wardley-beta\n",
                "title \u{7d05}\u{8336}\u{5e97}\n",
                "size [1100, 800]\n",
                "anchor \u{30d3}\u{30b8}\u{30cd}\u{30b9} [0.95, 0.63]\n",
                "component \u{7d05}\u{8336} [0.79, 0.61]\n",
                "\u{30d3}\u{30b8}\u{30cd}\u{30b9} -> \u{7d05}\u{8336}\n",
                "evolve \u{7d05}\u{8336} 0.62",
            ),
            "\u{7d05}\u{8336}\u{5e97}",
        ),
    ]
}

fn render_source(name: &str, source: &str) -> String {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: source.to_string(),
    };
    match mermaid_renderer::MermaidRenderOps::render_mermaid(&block) {
        DiagramResult::Ok(svg) => svg,
        other => panic!("{name} did not render: {other:?}"),
    }
}
