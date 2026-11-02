use super::GroupHeader;
use crate::app::action::ActionOps;
use crate::app_state::AppAction;
use crate::app_state::AppState;
use crate::shell::KatanaApp;
use crate::state::document::TabGroup;
use eframe::egui;
use std::sync::Arc;

const TEST_VIEWPORT: egui::Vec2 = egui::vec2(640.0, 240.0);

pub(super) fn make_app() -> KatanaApp {
    let mut state = AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::new(Box::new(katana_platform::InMemoryRepository)),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
        katana_platform::workspace::InMemoryWorkspaceRepository::default(),
    ));
    KatanaApp::new(state)
}

fn raw_input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, TEST_VIEWPORT)),
        events,
        ..egui::RawInput::default()
    }
}

fn key_event(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

fn show_frame(
    context: &egui::Context,
    group: &TabGroup,
    inline_rename_group: Option<String>,
    events: Vec<egui::Event>,
) -> (Option<AppAction>, egui::Rect, bool) {
    let mut action = None;
    let mut other_group_rect = egui::Rect::NOTHING;
    let mut other_group_clicked = false;
    let mut output = context.run_ui(raw_input(events), |ui| {
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                GroupHeader {
                    g: group,
                    groups: std::slice::from_ref(group),
                    inline_rename_group: inline_rename_group.as_ref(),
                }
                .show(ui, &mut action);
                let other = ui.button(&crate::i18n::I18nOps::get().tab.group_name_placeholder);
                other_group_rect = other.rect;
                other_group_clicked = other.clicked();
            })
            .show(ui);
    });
    output.textures_delta.clear();
    (action, other_group_rect, other_group_clicked)
}

fn click_input(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        },
    ]
}

fn group() -> TabGroup {
    TabGroup {
        id: "group_created".to_owned(),
        name: "New group".to_owned(),
        color_hex: "#4A90D9".to_owned(),
        collapsed: false,
        members: vec!["one.md".to_owned(), "two.md".to_owned()],
    }
}

#[test]
fn inline_rename_keeps_focus_until_enter_and_then_closes() {
    let context = egui::Context::default();
    let group = group();
    let inline_id = Some(group.id.clone());
    let popup_id = egui::Id::new("group_popup").with(&group.id);

    let _ = show_frame(&context, &group, inline_id.clone(), vec![]);
    assert!(egui::Popup::is_id_open(&context, popup_id));

    let (action, _, _) = show_frame(
        &context,
        &group,
        inline_id,
        vec![
            egui::Event::Text(" confirmed".to_owned()),
            key_event(egui::Key::Enter),
        ],
    );
    assert_eq!(
        action.as_ref(),
        Some(&AppAction::FinishInlineRename {
            group_id: group.id.clone(),
            new_identity: Some(("New group confirmed".to_owned(), group.color_hex.clone())),
        })
    );
    assert!(egui::Popup::is_id_open(&context, popup_id));

    let mut app = make_app();
    app.state.document.tab_groups.push(group.clone());
    app.state.layout.inline_rename_group = Some(group.id.clone());
    app.process_action(
        &context,
        action.expect("Enter should emit the finish action"),
    );
    assert!(!egui::Popup::is_id_open(&context, popup_id));
    assert_eq!(app.state.document.tab_groups[0].name, "New group confirmed");
    assert_eq!(app.state.layout.inline_rename_group, None);

    app.state.layout.inline_rename_group = Some("other_group".to_owned());
    app.process_action(
        &context,
        AppAction::FinishInlineRename {
            group_id: group.id,
            new_identity: None,
        },
    );
    assert_eq!(
        app.state.layout.inline_rename_group.as_deref(),
        Some("other_group")
    );
}

#[test]
fn escape_cancels_draft_and_group_interaction_works_after_popup_closes() {
    let context = egui::Context::default();
    let group = group();
    let inline_id = Some(group.id.clone());
    let popup_id = egui::Id::new("group_popup").with(&group.id);

    let _ = show_frame(&context, &group, inline_id.clone(), vec![]);
    let (action, other_group_rect, _) = show_frame(
        &context,
        &group,
        inline_id,
        vec![
            egui::Event::Text(" discarded".to_owned()),
            key_event(egui::Key::Escape),
        ],
    );
    assert_eq!(
        action.clone(),
        Some(AppAction::FinishInlineRename {
            group_id: group.id.clone(),
            new_identity: None,
        })
    );
    assert!(!egui::Popup::is_id_open(&context, popup_id));
    let mut app = make_app();
    app.state.document.tab_groups.push(group.clone());
    app.state.layout.inline_rename_group = Some(group.id.clone());
    app.process_action(&context, action.expect("Escape emits cancellation"));
    assert_eq!(app.state.layout.inline_rename_group, None);
    assert_eq!(app.state.document.tab_groups[0].name, group.name);

    let _ = show_frame(
        &context,
        &group,
        None,
        click_input(other_group_rect.center(), true),
    );
    let (_, _, clicked) = show_frame(
        &context,
        &group,
        None,
        click_input(other_group_rect.center(), false),
    );
    assert!(
        clicked,
        "another group remains clickable after rename cancellation"
    );
}

#[test]
fn outside_click_closes_rename_and_next_group_click_is_delivered() {
    let context = egui::Context::default();
    let group = group();
    let inline_id = Some(group.id.clone());
    let popup_id = egui::Id::new("group_popup").with(&group.id);

    let _ = show_frame(&context, &group, inline_id.clone(), vec![]);
    assert!(egui::Popup::is_id_open(&context, popup_id));

    let outside = egui::pos2(600.0, 220.0);
    let _ = show_frame(
        &context,
        &group,
        inline_id.clone(),
        click_input(outside, true),
    );
    let _ = show_frame(
        &context,
        &group,
        inline_id.clone(),
        click_input(outside, false),
    );
    assert!(!egui::Popup::is_id_open(&context, popup_id));
    let (action, _, _) = show_frame(&context, &group, inline_id, vec![]);
    assert_eq!(
        action,
        Some(AppAction::FinishInlineRename {
            group_id: group.id.clone(),
            new_identity: None,
        })
    );

    let (_, target, _) = show_frame(&context, &group, None, vec![]);
    let _ = show_frame(&context, &group, None, click_input(target.center(), true));
    let (_, _, clicked) = show_frame(&context, &group, None, click_input(target.center(), false));
    assert!(clicked, "tab-group interaction resumes after outside-close");
}

#[test]
fn removing_the_final_group_member_clears_its_inline_rename() {
    let context = egui::Context::default();
    let mut app = make_app();
    let mut group = group();
    let member = std::path::PathBuf::from("notes.md");
    group.members = vec![member.to_string_lossy().into_owned()];
    app.state.layout.inline_rename_group = Some(group.id.clone());
    app.state.document.tab_groups.push(group);
    app.process_action(&context, AppAction::RemoveTabFromGroup(member));
    assert!(app.state.document.tab_groups.is_empty());
    assert_eq!(app.state.layout.inline_rename_group, None);
}
