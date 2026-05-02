use katana_core::markdown::mermaid_renderer;
use katana_core::markdown::{
    MarkdownRenderOps,
    diagram::{DiagramBlock, DiagramKind, DiagramResult},
};
use std::sync::Mutex;

static I18N_RENDER_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn renders_sample_mermaid_ja_without_diagram_errors() {
    let _guard = I18N_RENDER_LOCK.lock().unwrap();
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
        "クリスマス",
        "買い物へ行く",
        "私の作業日",
        "ボランティアが引き取ったペット",
        "顧客",
        "注文明細",
        "中央の幅広いブロック",
        "テスト用の要求。",
        "テスト対象",
        "キャンペーンの到達と反応",
        "売上（円）",
        "構文解析",
        "プレビュー",
        "農業廃棄物",
        "火力発電",
        "データベース",
        "写真のぼやけ",
        "紅茶店",
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
        "CUSTOMER",
    ] {
        assert!(
            !output.html.contains(english_text),
            "localized Mermaid fixture should not keep English sample text: {english_text}"
        );
    }
}

#[test]
fn renders_non_ascii_diagram_syntax_as_visible_text() {
    let _guard = I18N_RENDER_LOCK.lock().unwrap();
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
    let _guard = I18N_RENDER_LOCK.lock().unwrap();
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping i18n Mermaid regression");
        return;
    }

    let svg = render_source(
        "localized sankey repeated labels",
        "sankey-beta\n農業廃棄物,生物変換,124.729\n生物変換,液体燃料,0.597\n生物変換,損失,26.862\n生物変換,固体燃料,280.322\n生物変換,ガス,81.144\n海藻,生物変換,4.375\nその他廃棄物,生物変換,77.81",
    );

    assert!(svg.contains("農業廃棄物"));
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

fn localized_diagrams() -> [(&'static str, &'static str, &'static str); 7] {
    [
        (
            "er",
            "erDiagram\n顧客 ||--o{ 注文 : 注文する\n注文 ||--|{ 注文明細 : 含む\n顧客 {\nstring 氏名\n}",
            "注文明細",
        ),
        (
            "requirement",
            "requirementDiagram\nrequirement テスト要求 {\nid: 1\ntext: テスト用の要求。\nrisk: high\nverifymethod: test\n}\nelement テスト対象 {\ntype: シミュレーション\n}\nテスト対象 - satisfies -> テスト要求",
            "テスト対象",
        ),
        (
            "quadrant",
            "quadrantChart\ntitle キャンペーンの到達と反応\nx-axis 低到達 --> 高到達\ny-axis 低反応 --> 高反応\nquadrant-1 拡大すべき\nキャンペーンA: [0.3, 0.6]",
            "キャンペーンの到達と反応",
        ),
        (
            "xychart",
            "xychart-beta\ntitle \"売上\"\nx-axis [1月, 2月]\ny-axis \"売上（円）\" 4000 --> 11000\nbar [5000, 6000]",
            "売上（円）",
        ),
        (
            "sankey",
            "sankey-beta\n農業廃棄物,生物変換,124.729\n生物変換,液体燃料,0.597",
            "農業廃棄物",
        ),
        (
            "architecture",
            "architecture-beta\ngroup api(cloud)[API]\nservice db(database)[データベース] in api",
            "データベース",
        ),
        (
            "wardley",
            "wardley-beta\ntitle 紅茶店\nsize [1100, 800]\nanchor ビジネス [0.95, 0.63]\ncomponent 紅茶 [0.79, 0.61]\nビジネス -> 紅茶\nevolve 紅茶 0.62",
            "紅茶店",
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
