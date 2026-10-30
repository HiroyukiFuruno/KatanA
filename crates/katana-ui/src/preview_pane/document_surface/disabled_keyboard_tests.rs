use super::activation_keyboard_tests::{page_input, selected_pdf_app};
use super::{navigation_command, navigation_response};
use eframe::egui;

const PAGE_COUNT: usize = 3;

#[test]
fn disabled_modal_background_cannot_take_focus_or_page_on_arrow() {
    let (_directory, ctx, _app, owner_path) = selected_pdf_app();
    let page_id = egui::Id::new(("document-page-navigation", 0_u64));
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        navigation_response(ui, 0).request_focus();
    });
    output.textures_delta.clear();
    drop(output);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(page_id));

    crate::app::preview::keyboard_focus::PageNavigationFocusOps::arm_for_selection(
        &ctx,
        &owner_path,
    );
    let mut command = None;
    let mut took_focus = false;
    let input = page_input(vec![egui::Event::Key {
        key: egui::Key::ArrowRight,
        physical_key: Some(egui::Key::ArrowRight),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    }]);
    let mut output = ctx.run_ui(input, |ui| {
        ui.add_enabled_ui(false, |ui| {
            if super::prepare_navigation(ui) {
                let response = navigation_response(ui, 0);
                took_focus =
                    crate::app::preview::keyboard_focus::PageNavigationFocusOps::focus_if_activated(
                        ui.ctx(),
                        &response,
                        &owner_path,
                    ) == Some(true);
                command = navigation_command(
                    ui,
                    &response,
                    &super::keyboard_tests::frame(
                        katana_document_viewer::ViewerDocumentFormat::Pdf,
                        0,
                        PAGE_COUNT,
                    ),
                );
            }
        });
        egui::Area::new(egui::Id::new("blocking-modal-layer"))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                crate::widgets::InteractionFacade::consume_rect(
                    ui,
                    "blocking-modal-input",
                    ui.max_rect(),
                );
                ui.label(&crate::i18n::I18nOps::get().search.modal_title);
            });
    });
    output.textures_delta.clear();
    drop(output);

    assert!(!took_focus);
    assert_eq!(command, None);
    /* WHY: 無効化されたwidgetのfocusはeguiが解放するため、前景から奪い返さないことを確認する。 */
    assert_ne!(ctx.memory(|memory| memory.focused()), Some(page_id));

    let mut activation = None;
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        let response = navigation_response(ui, 0);
        activation =
            crate::app::preview::keyboard_focus::PageNavigationFocusOps::focus_if_activated(
                ui.ctx(),
                &response,
                &owner_path,
            );
    });
    output.textures_delta.clear();
    drop(output);
    assert_eq!(activation, None);
}
