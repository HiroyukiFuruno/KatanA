use super::types::*;
use egui_commonmark::TaskListAction;
use std::ops::Range;

impl MarkdownHooksOps {
    pub(super) fn push_context_menu(
        response: &egui::Response,
        span: &Range<usize>,
        events: &mut Vec<TaskListAction>,
    ) {
        response.context_menu(|ui| {
            let i18n = crate::i18n::I18nOps::get();
            if ui.button(i18n.markdown.task_todo.clone()).clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: ' ',
                });
                ui.close();
            }
            if ui.button(i18n.markdown.task_in_progress.clone()).clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: '/',
                });
                ui.close();
            }
            if ui.button(i18n.markdown.task_done.clone()).clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: 'x',
                });
                ui.close();
            }
        });
    }
    pub fn katana_task_context_menu(
        response: &egui::Response,
        state: char,
        span: Range<usize>,
        _mutable: bool,
        events: &mut Vec<TaskListAction>,
    ) {
        if response.clicked() {
            let new_state = match state {
                ' ' => 'x',
                '/' | '-' | '~' => 'x',
                'x' | 'X' => ' ',
                _ => ' ',
            };
            events.push(TaskListAction {
                span: span.clone(),
                new_state,
            });
        }
        response.context_menu(|ui| {
            let i18n = crate::i18n::I18nOps::get();
            if ui.button(i18n.markdown.task_todo.clone()).clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: ' ',
                });
                ui.close();
            }
            if ui.button(i18n.markdown.task_in_progress.clone()).clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: '/',
                });
                ui.close();
            }
            if ui.button(i18n.markdown.task_done.clone()).clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: 'x',
                });
                ui.close();
            }
        });
    }

    pub fn katana_list_item_highlight(
        active_char_range: Option<Range<usize>>,
        active_bg_color: egui::Color32,
        hover_bg_color: egui::Color32,
    ) -> impl Fn(&mut egui::Ui, egui::Rect, &Range<usize>) -> (bool, bool) {
        move |ui: &mut egui::Ui, rect: egui::Rect, span: &Range<usize>| {
            let mut highlighted = false;
            let mut hovered = false;
            /* WHY: Active highlight (editor cursor line) */
            if let Some(ref active) = active_char_range
                && active.start <= span.end
                && active.end >= span.start
            {
                highlighted = true;
                ui.painter()
                    .rect_filled(rect, HIGHLIGHT_ROUNDING, active_bg_color);
            }
            /* WHY: Hover highlight (mouse pointer) */
            if let Some(pos) = ui.ctx().pointer_hover_pos()
                && rect.contains(pos)
            {
                hovered = true;
                ui.painter()
                    .rect_filled(rect, HIGHLIGHT_ROUNDING, hover_bg_color);
            }
            (highlighted, hovered)
        }
    }
}
