//! プレビューペイン — egui_commonmark によるネイティブ Markdown レンダリング
//! + ダイアグラムブロックのラスタライズ画像表示。
//!
//! 設計方針（MVP）:
//! - テキスト変更のたびに Markdown 部分は即座に更新する（egui_commonmark）。
//! - ダイアグラムはサブプロセスを伴うため、「🔄 Refresh」ボタン or
//!   ドキュメント選択時にのみ再レンダリングする。

use eframe::egui::{self, ScrollArea, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::diagram::DiagramKind;
use katana_core::{
    markdown::{
        diagram::{DiagramBlock, DiagramResult},
        drawio_renderer, mermaid_renderer, plantuml_renderer,
        svg_rasterize::{rasterize_svg, RasterizedSvg},
    },
    preview::{split_into_sections, PreviewSection},
};

/// UI 層で保持するレンダリング済みセクション。
#[derive(Debug, Clone)]
pub enum RenderedSection {
    /// egui_commonmark で描画する Markdown テキスト。
    Markdown(String),
    /// ラスタライズ済みダイアグラム画像。
    Image {
        svg_data: RasterizedSvg,
        alt: String,
    },
    /// レンダリングエラー（ソースとメッセージを保持）。
    Error {
        kind: String,
        _source: String,
        message: String,
    },
    /// コマンドラインツールが見つからない（パスの問題など）。
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
    },
    /// 必要なツールが未インストール— UI からダウンロードできる。
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
    },
    /// バックグラウンドレンダリング中のプレースホルダー。
    Pending { kind: String },
}

/// プレビューペインから返されるダウンロードリクエスト。
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}

#[derive(Default)]
pub struct PreviewPane {
    commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
    /// バックグラウンドレンダリング完了通知チャネル。
    render_rx: Option<std::sync::mpsc::Receiver<(usize, RenderedSection)>>,
}

impl PreviewPane {
    /// Markdown ソースからテキストセクションのみ即時更新する（ダイアグラムは保持）。
    pub fn update_markdown_sections(&mut self, source: &str) {
        let raw = split_into_sections(source);
        let mut new_sections = Vec::with_capacity(raw.len());
        let mut diagram_iter = self
            .sections
            .iter()
            .filter(|s| !matches!(s, RenderedSection::Markdown(_)));
        for section in &raw {
            match section {
                PreviewSection::Markdown(md) => {
                    new_sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram { kind, source } => {
                    // 既存のレンダリング済み画像があれば再利用する。
                    let reused =
                        diagram_iter
                            .next()
                            .cloned()
                            .unwrap_or_else(|| RenderedSection::Error {
                                kind: format!("{kind:?}"),
                                _source: source.clone(),
                                message: "🔄 プレビューを更新してください".to_string(),
                            });
                    new_sections.push(reused);
                }
            }
        }
        self.sections = new_sections;
    }

    /// 全セクション（ダイアグラム含む）を完全に再レンダリングする。
    ///
    /// Markdown セクションは即座に返す。ダイアグラムは `Pending` にせて
    /// バックグラウンドスレッドでレンダリングする。
    pub fn full_render(&mut self, source: &str) {
        let raw = split_into_sections(source);
        // 前回レンダリングをキャンセル。
        self.render_rx = None;

        let mut sections = Vec::with_capacity(raw.len());
        let mut jobs: Vec<(usize, DiagramKind, String)> = Vec::new();

        for (i, section) in raw.iter().enumerate() {
            match section {
                PreviewSection::Markdown(md) => {
                    sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram { kind, source: src } => {
                    sections.push(RenderedSection::Pending {
                        kind: format!("{kind:?}"),
                    });
                    jobs.push((i, kind.clone(), src.clone()));
                }
            }
        }
        self.sections = sections;

        if jobs.is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.render_rx = Some(rx);
        std::thread::spawn(move || {
            for (index, kind, src) in jobs {
                let section = render_diagram(&kind, &src);
                if tx.send((index, section)).is_err() {
                    break; // レシーバがドロップされた。
                }
            }
        });
    }

    /// プレビューペインの内容を描画する（ScrollArea 込み）。
    /// PreviewOnly モードなどスクロール同期が不要な場面で使う。
    /// ダウンロードボタンが押された場合は `Some(DownloadRequest)` を返す。
    #[allow(dead_code)]
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        // バックグラウンドレンダリング完了をポーリング。
        self.poll_renders(ui.ctx());

        let mut request: Option<DownloadRequest> = None;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                request = self.render_sections(ui);
            });
        request
    }

    /// ScrollArea なしでプレビューコンテンツだけを描画する。
    /// 外側で ScrollArea を制御したい場合（スクロール同期など）に使う。
    pub fn show_content(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        self.poll_renders(ui.ctx());
        self.render_sections(ui)
    }

    /// セクションを順に描画する内部メソッド。
    fn render_sections(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let mut request: Option<DownloadRequest> = None;
        for (i, section) in self.sections.iter().enumerate() {
            // セクションごとに ID スコープを分離し、同一ドキュメント内に
            // 複数のテーブルがあっても egui の Grid ID が衝突しないようにする。
            ui.push_id(format!("section_{i}"), |ui| {
                if let Some(req) = show_section(ui, &mut self.commonmark_cache, section, i) {
                    request = Some(req);
                }
                ui.separator();
            });
        }
        if self.sections.is_empty() {
            ui.label(egui::RichText::new("（プレビューなし）").weak());
        }
        request
    }

    /// バックグラウンドレンダリング完了をポーリングし、届いた結果でセクションを更新する。
    fn poll_renders(&mut self, ctx: &egui::Context) {
        let still_pending = if let Some(rx) = &self.render_rx {
            let mut updated = false;
            while let Ok((idx, section)) = rx.try_recv() {
                if idx < self.sections.len() {
                    self.sections[idx] = section;
                    updated = true;
                }
            }
            if updated {
                ctx.request_repaint();
            }
            self.sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
        } else {
            false
        };
        if still_pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            self.render_rx = None;
        }
    }

    /// テスト用: Pending がなくなるまでバックグラウンドスレッドをブロック待機する。
    #[cfg(test)]
    pub fn wait_for_renders(&mut self) {
        loop {
            if let Some(rx) = &self.render_rx {
                while let Ok((idx, section)) = rx.try_recv() {
                    if idx < self.sections.len() {
                        self.sections[idx] = section;
                    }
                }
                if self
                    .sections
                    .iter()
                    .any(|s| matches!(s, RenderedSection::Pending { .. }))
                {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                } else {
                    self.render_rx = None;
                    break;
                }
            } else {
                break;
            }
        }
    }
}

/// 単一セクションを描画する。
/// ダウンロードボタンが押された場合は `Some(DownloadRequest)` を返す。
fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
) -> Option<DownloadRequest> {
    match section {
        RenderedSection::Markdown(md) => {
            CommonMarkViewer::new().show(ui, cache, md);
            None
        }
        RenderedSection::Image { svg_data, alt } => {
            show_rasterized(ui, svg_data, alt, id);
            None
        }
        RenderedSection::Error {
            kind,
            _source: _,
            message,
        } => {
            ui.label(
                egui::RichText::new(format!("⚠ [{kind}] {message}"))
                    .color(egui::Color32::YELLOW)
                    .small(),
            );
            None
        }
        RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: _,
        } => {
            let msg = crate::i18n::t("missing_dependency")
                .replace("{tool_name}", tool_name)
                .replace("{install_hint}", install_hint);
            ui.label(
                egui::RichText::new(msg)
                    .color(egui::Color32::YELLOW)
                    .small(),
            );
            None
        }
        RenderedSection::NotInstalled {
            kind,
            download_url,
            install_path,
        } => show_not_installed(ui, kind, download_url, install_path),
        RenderedSection::Pending { kind } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new(format!("{} {}", kind, crate::i18n::t("rendering"))).weak(),
                );
            });
            None
        }
    }
}

/// 未インストールツールのダウンロードボタン UI。
fn show_not_installed(
    ui: &mut egui::Ui,
    kind: &str,
    download_url: &str,
    install_path: &std::path::Path,
) -> Option<DownloadRequest> {
    let mut request = None;
    ui.group(|ui| {
        ui.label(
            egui::RichText::new(format!("⚠ {kind} がインストールされていません"))
                .color(egui::Color32::from_rgb(255, 165, 0)),
        );
        ui.label(
            egui::RichText::new(format!("インストール先: {}", install_path.display()))
                .small()
                .weak(),
        );
        if ui.button(format!("⬇ {} をダウンロード", kind)).clicked() {
            request = Some(DownloadRequest {
                url: download_url.to_string(),
                dest: install_path.to_path_buf(),
            });
        }
    });
    request
}

/// ラスタライズ済み SVG を egui テクスチャとして表示する。
fn show_rasterized(ui: &mut egui::Ui, img: &RasterizedSvg, _alt: &str, id: usize) {
    let color_img = egui::ColorImage::from_rgba_unmultiplied(
        [img.width as usize, img.height as usize],
        &img.rgba,
    );
    let texture = ui.ctx().load_texture(
        format!("diagram_{id}"),
        color_img,
        egui::TextureOptions::LINEAR,
    );
    let max_w = ui.available_width();
    let scale = (max_w / img.width as f32).min(1.0);
    let size = Vec2::new(img.width as f32 * scale, img.height as f32 * scale);
    ui.add(egui::Image::new((texture.id(), size)));
}

/// `PreviewSection` をレンダリングして `RenderedSection` に変換する（非使用になったことでの削除答候用コメント）。
/// ダイアグラムブロックをレンダラー経由で変換し、SVG ラスタライズを試みる。
fn render_diagram(kind: &DiagramKind, source: &str) -> RenderedSection {
    let block = DiagramBlock {
        kind: kind.clone(),
        source: source.to_string(),
    };
    let result = dispatch_renderer(&block);
    match result {
        DiagramResult::Ok(html) => try_rasterize(kind, source, &html),
        DiagramResult::OkPng(bytes) => decode_png_to_section(kind, source, bytes),
        DiagramResult::Err { source, error } => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
        },
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: source,
        },
        DiagramResult::NotInstalled {
            kind: k,
            download_url,
            install_path,
        } => RenderedSection::NotInstalled {
            kind: k,
            download_url,
            install_path,
        },
    }
}

/// ダイアグラム種別ごとのレンダラーに委譲する。
fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    match block.kind {
        DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
        DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
        DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
    }
}

/// HTML フラグメントから SVG を抽出してラスタライズする。
fn try_rasterize(kind: &DiagramKind, source: &str, html: &str) -> RenderedSection {
    let Some(svg) = extract_svg(html) else {
        return RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: "SVG の抽出に失敗しました".to_string(),
        };
    };
    match rasterize_svg(svg, 1.5) {
        Ok(img) => RenderedSection::Image {
            svg_data: img,
            alt: format!("{kind:?} diagram"),
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: e.to_string(),
        },
    }
}

/// HTML フラグメントから `<svg...>...</svg>` を抽出する。
fn extract_svg(html: &str) -> Option<&str> {
    let start = html.find("<svg")?;
    let end = html.rfind("</svg>")? + "</svg>".len();
    Some(&html[start..end])
}

/// PNG バイト列を `RenderedSection::Image` に変換する。
///
/// mmdc の PNG 出力を `image` クレートでデコードし、RGBA ピクセルバッファを取得する。
/// これにより resvg の `<foreignObject>` 非対応を完全に回避できる。
fn decode_png_to_section(kind: &DiagramKind, source: &str, bytes: Vec<u8>) -> RenderedSection {
    match decode_png_rgba(&bytes) {
        Ok(rasterized) => RenderedSection::Image {
            svg_data: rasterized,
            alt: format!("{kind:?} diagram"),
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: format!("PNG デコード失敗: {e}"),
        },
    }
}

/// PNG バイト列を RGBA ピクセルに変換する。
fn decode_png_rgba(bytes: &[u8]) -> Result<RasterizedSvg, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RasterizedSvg {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ヘルパー: RenderedSection から Markdown テキストを抽出する。
    fn markdown_texts(sections: &[RenderedSection]) -> Vec<&str> {
        sections
            .iter()
            .filter_map(|s| match s {
                RenderedSection::Markdown(md) => Some(md.as_str()),
                _ => None,
            })
            .collect()
    }

    // ── 3.2 プレビュー同期: 未保存バッファからの即時プレビュー更新 ──

    #[test]
    fn 未保存バッファの変更がプレビューに反映される() {
        let mut pane = PreviewPane::default();

        // 初期コンテンツでプレビューを構築
        pane.update_markdown_sections("# Hello");
        assert_eq!(pane.sections.len(), 1);
        let texts = markdown_texts(&pane.sections);
        assert!(texts[0].contains("# Hello"));

        // ファイル保存なしでバッファを更新 → プレビューに反映される
        pane.update_markdown_sections("# Hello World\n\nNew paragraph");
        let texts = markdown_texts(&pane.sections);
        assert!(texts[0].contains("# Hello World"));
        assert!(texts[0].contains("New paragraph"));
    }

    #[test]
    fn 連続編集がプレビューに即時反映される() {
        let mut pane = PreviewPane::default();

        // 複数回の連続編集がすべて反映される
        let edits = vec![
            "# Draft 1",
            "# Draft 2\n\n- item A",
            "# Draft 3\n\n- item A\n- item B\n- item C",
        ];

        for edit in &edits {
            pane.update_markdown_sections(edit);
            let texts = markdown_texts(&pane.sections);
            assert!(
                texts[0].contains(edit),
                "Edit not reflected in preview: {edit}"
            );
        }
    }

    #[test]
    fn 空バッファでもプレビューがクラッシュしない() {
        let mut pane = PreviewPane::default();

        // コンテンツを入力
        pane.update_markdown_sections("# Hello");
        assert_eq!(pane.sections.len(), 1);

        // 空に戻す → セクション数は 0 になる（空文字列は flush されない）
        pane.update_markdown_sections("");
        assert_eq!(pane.sections.len(), 0);
    }

    #[test]
    fn ダイアグラム含みのバッファでmarkdown部分のみ即時更新される() {
        let mut pane = PreviewPane::default();

        // ダイアグラムを含む初期コンテンツ
        let source = "# Title\n```mermaid\ngraph TD; A-->B\n```\n## Footer";
        pane.full_render(source);

        // ダイアグラムは Pending になっている
        assert!(pane.sections.len() >= 3);
        assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));

        // Markdown 部分のみ更新する（ダイアグラムは保持）
        let modified = "# Updated Title\n```mermaid\ngraph TD; A-->B\n```\n## Updated Footer";
        pane.update_markdown_sections(modified);

        // Markdown テキストが更新されていることを確認
        let texts = markdown_texts(&pane.sections);
        assert!(texts.iter().any(|t| t.contains("Updated Title")));
        assert!(texts.iter().any(|t| t.contains("Updated Footer")));
    }

    #[test]
    fn full_renderでセクションが正しく分割される() {
        let mut pane = PreviewPane::default();

        let source = "Before\n```mermaid\ngraph TD; A-->B\n```\nAfter";
        pane.full_render(source);

        // 3 セクション: Markdown, Diagram(Pending), Markdown
        assert_eq!(pane.sections.len(), 3);
        assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
        assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));
        assert!(matches!(pane.sections[2], RenderedSection::Markdown(_)));
    }

    #[test]
    fn ダイアグラムなしのバッファではpendingセクションが生成されない() {
        let mut pane = PreviewPane::default();

        pane.full_render("# Pure Markdown\n\nNo diagrams here.");

        assert!(pane
            .sections
            .iter()
            .all(|s| matches!(s, RenderedSection::Markdown(_))));
        assert!(!pane
            .sections
            .iter()
            .any(|s| matches!(s, RenderedSection::Pending { .. })));
    }

    #[test]
    fn プレビュー更新がファイル保存に依存しないことの検証() {
        // Document + PreviewPane の統合テスト:
        // ドキュメントのバッファを更新（is_dirty = true）し、save を呼ばずに
        // プレビューが最新バッファを反映していることを確認する。
        use katana_core::document::Document;

        let mut doc = Document::new("/workspace/spec.md", "# Original");
        let mut pane = PreviewPane::default();

        // 初期プレビュー
        pane.update_markdown_sections(&doc.buffer);
        let texts = markdown_texts(&pane.sections);
        assert!(texts[0].contains("# Original"));

        // ドキュメントを編集（未保存状態）
        doc.update_buffer("# Modified by user\n\nThis is not saved yet.");
        assert!(doc.is_dirty, "ドキュメントは dirty でなければならない");

        // 未保存バッファでプレビューを更新
        pane.update_markdown_sections(&doc.buffer);
        let texts = markdown_texts(&pane.sections);
        assert!(
            texts[0].contains("Modified by user"),
            "未保存の編集がプレビューに反映されていない"
        );
        assert!(
            texts[0].contains("This is not saved yet"),
            "未保存の編集がプレビューに反映されていない"
        );

        // ドキュメントはまだ dirty のまま（保存していない）
        assert!(doc.is_dirty, "ドキュメントは保存されていないはず");
    }
}
