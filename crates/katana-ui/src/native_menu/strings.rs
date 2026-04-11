#[cfg(all(target_os = "macos", not(test)))]
use super::ffi;

#[cfg(all(target_os = "macos", not(test)))]
pub(super) fn update_from_i18n() {
    let msgs = crate::i18n::I18nOps::get();
    let preferences = format!("{}…", msgs.menu.settings);
    unsafe {
        update_menu_strings(
            &msgs.menu.file,
            &msgs.menu.open_workspace,
            &msgs.menu.save,
            &msgs.menu.settings,
            &preferences,
            &msgs.menu.language,
            &msgs.menu.about,
            &msgs.menu.quit,
            &msgs.menu.hide,
            &msgs.menu.hide_others,
            &msgs.menu.show_all,
            &msgs.menu.check_updates,
            &msgs.menu.help,
            &msgs.menu.release_notes,
            &msgs.menu.command_palette,
            &msgs.menu.view,
            &msgs.menu.demo,
            &msgs.menu.welcome_screen,
            &msgs.menu.user_guide,
        );
    }
}

#[cfg(all(target_os = "macos", not(test)))]
#[allow(clippy::too_many_arguments)]
unsafe fn update_menu_strings(
    file: &str,
    open_workspace: &str,
    save: &str,
    settings: &str,
    preferences: &str,
    language: &str,
    about: &str,
    quit: &str,
    hide: &str,
    hide_others: &str,
    show_all: &str,
    check_updates: &str,
    help: &str,
    release_notes: &str,
    command_palette: &str,
    view: &str,
    demo: &str,
    welcome_screen: &str,
    user_guide: &str,
) {
    let mk = |s: &str| std::ffi::CString::new(s).unwrap_or_default();
    let c_file = mk(file);
    let c_open_workspace = mk(open_workspace);
    let c_save = mk(save);
    let c_settings = mk(settings);
    let c_preferences = mk(preferences);
    let c_language = mk(language);
    let c_about = mk(about);
    let c_quit = mk(quit);
    let c_hide = mk(hide);
    let c_hide_others = mk(hide_others);
    let c_show_all = mk(show_all);
    let c_check_updates = mk(check_updates);
    let c_help = mk(help);
    let c_release_notes = mk(release_notes);
    let c_command_palette = mk(command_palette);
    let c_view = mk(view);
    let c_demo = mk(demo);
    let c_welcome = mk(welcome_screen);
    let c_guide = mk(user_guide);

    ffi::katana_update_menu_strings(
        c_file.as_ptr(),
        c_open_workspace.as_ptr(),
        c_save.as_ptr(),
        c_settings.as_ptr(),
        c_preferences.as_ptr(),
        c_language.as_ptr(),
        c_about.as_ptr(),
        c_quit.as_ptr(),
        c_hide.as_ptr(),
        c_hide_others.as_ptr(),
        c_show_all.as_ptr(),
        c_check_updates.as_ptr(),
        c_help.as_ptr(),
        c_release_notes.as_ptr(),
        c_command_palette.as_ptr(),
        c_view.as_ptr(),
        c_demo.as_ptr(),
        c_welcome.as_ptr(),
        c_guide.as_ptr(),
    );
}
