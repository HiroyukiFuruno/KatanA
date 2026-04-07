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
) {
    let mk = |s: &str| std::ffi::CString::new(s).unwrap_or_default();
    ffi::katana_update_menu_strings(
        mk(file).as_ptr(),
        mk(open_workspace).as_ptr(),
        mk(save).as_ptr(),
        mk(settings).as_ptr(),
        mk(preferences).as_ptr(),
        mk(language).as_ptr(),
        mk(about).as_ptr(),
        mk(quit).as_ptr(),
        mk(hide).as_ptr(),
        mk(hide_others).as_ptr(),
        mk(show_all).as_ptr(),
        mk(check_updates).as_ptr(),
        mk(help).as_ptr(),
        mk(release_notes).as_ptr(),
        mk(command_palette).as_ptr(),
        mk(view).as_ptr(),
        mk(demo).as_ptr(),
    );
}
