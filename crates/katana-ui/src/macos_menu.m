#import <Cocoa/Cocoa.h>

/* WHY: Tag constants for identifying menu actions. */
/* WHY: Should match the MenuAction enum in Rust. */
enum {
    TAG_OPEN_WORKSPACE = 1,
    TAG_SAVE           = 2,
    TAG_LANG_EN        = 3,
    TAG_LANG_JA        = 4,
    TAG_ABOUT          = 5,
    TAG_SETTINGS       = 6,
    TAG_LANG_ZH_CN     = 7,
    TAG_LANG_ZH_TW     = 8,
    TAG_LANG_KO        = 9,
    TAG_LANG_PT        = 10,
    TAG_LANG_FR        = 11,
    TAG_LANG_DE        = 12,
    TAG_LANG_ES        = 13,
    TAG_LANG_IT        = 14,
    TAG_CHECK_UPDATES  = 15,
    TAG_RELEASE_NOTES  = 16,
    TAG_COMMAND_PALETTE = 17,
    TAG_DEMO           = 18,
    TAG_WELCOME_SCREEN = 19,
    TAG_USER_GUIDE      = 20,
    TAG_SEARCH_DOCUMENT = 21,
    TAG_SEARCH_WORKSPACE = 22,
    TAG_CLOSE_WORKSPACE = 23,
    TAG_EXPLORER       = 24,
    TAG_REFRESH_EXPLORER = 25,
    TAG_CLOSE_ALL      = 26,
    TAG_GITHUB         = 27,
};

/* WHY: Global: Tag of the last selected menu action. */
/* WHY: Read by polling from Rust. */
static volatile int g_last_action = 0;

@interface KatanaMenuTarget : NSObject
- (void)menuAction:(id)sender;
@end

@implementation KatanaMenuTarget
- (void)menuAction:(id)sender {
    NSMenuItem *item = (NSMenuItem *)sender;
    g_last_action = (int)[item tag];
}
@end

static KatanaMenuTarget *g_target = nil;

static NSMenu *g_file_menu = nil;
static NSMenuItem *g_open_workspace_item = nil;
static NSMenuItem *g_close_workspace_item = nil;
static NSMenuItem *g_save_item = nil;
static NSMenu *g_view_menu = nil;
static NSMenuItem *g_command_palette_item = nil;
static NSMenuItem *g_explorer_item = nil;
static NSMenuItem *g_refresh_explorer_item = nil;
static NSMenuItem *g_close_all_item = nil;
static NSMenu *g_settings_menu = nil;
static NSMenuItem *g_preferences_item = nil;
static NSMenu *g_language_menu = nil;
static NSMenuItem *g_about_item = nil;
static NSMenuItem *g_check_updates_item = nil;
static NSMenuItem *g_quit_item = nil;
static NSMenu *g_help_menu = nil;
static NSMenuItem *g_release_notes_item = nil;
static NSMenuItem *g_welcome_item = nil;
static NSMenuItem *g_guide_item = nil;
static NSMenuItem *g_demo_item = nil;
static NSMenuItem *g_github_item = nil;

/// Called from Rust at the very start of main(), before eframe creates the window.
/// Must be called before the window server registers the process to ensure
/// the Dock label shows "KatanA" instead of the binary name "katana".
void katana_set_process_name(void) {
    [[NSProcessInfo processInfo] setProcessName:@"KatanA"];
}

/// Called from Rust: Builds the native menu bar.
void katana_setup_native_menu(void) {
    g_target = [[KatanaMenuTarget alloc] init];
    SEL action = @selector(menuAction:);

    /* WHY: --- Application Menu --- */
    NSMenu *appMenu = [[NSMenu alloc] initWithTitle:@"KatanA"];

    NSMenuItem *aboutItem = [[NSMenuItem alloc]
        initWithTitle:@"About KatanA"
        action:action
        keyEquivalent:@""];
    [aboutItem setTarget:g_target];
    [aboutItem setTag:TAG_ABOUT];
    [appMenu addItem:aboutItem];
    g_about_item = aboutItem;

    NSMenuItem *checkUpdatesItem = [[NSMenuItem alloc]
        initWithTitle:@"Check for Updates…"
        action:action
        keyEquivalent:@""];
    [checkUpdatesItem setTarget:g_target];
    [checkUpdatesItem setTag:TAG_CHECK_UPDATES];
    [appMenu addItem:checkUpdatesItem];
    g_check_updates_item = checkUpdatesItem;

    [appMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *quitItem = [[NSMenuItem alloc]
        initWithTitle:@"Quit KatanA"
        action:@selector(terminate:)
        keyEquivalent:@"q"];
    [appMenu addItem:quitItem];
    g_quit_item = quitItem;

    NSMenuItem *appMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [appMenuItem setSubmenu:appMenu];

    /* WHY: --- File Menu --- */
    NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
    g_file_menu = fileMenu;

    NSMenuItem *openItem = [[NSMenuItem alloc]
        initWithTitle:@"Open Workspace…"
        action:action
        keyEquivalent:@"o"];
    [openItem setTarget:g_target];
    [openItem setTag:TAG_OPEN_WORKSPACE];
    [fileMenu addItem:openItem];
    g_open_workspace_item = openItem;

    NSMenuItem *closeWsItem = [[NSMenuItem alloc]
        initWithTitle:@"Close Workspace"
        action:action
        keyEquivalent:@""];
    [closeWsItem setTarget:g_target];
    [closeWsItem setTag:TAG_CLOSE_WORKSPACE];
    [fileMenu addItem:closeWsItem];
    g_close_workspace_item = closeWsItem;

    [fileMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *saveItem = [[NSMenuItem alloc]
        initWithTitle:@"Save"
        action:action
        keyEquivalent:@"s"];
    [saveItem setTarget:g_target];
    [saveItem setTag:TAG_SAVE];
    [fileMenu addItem:saveItem];
    g_save_item = saveItem;

    NSMenuItem *fileMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [fileMenuItem setSubmenu:fileMenu];

    /* WHY: --- View Menu --- */
    NSMenu *viewMenu = [[NSMenu alloc] initWithTitle:@"View"];
    g_view_menu = viewMenu;

    NSMenuItem *paletteItem = [[NSMenuItem alloc]
        initWithTitle:@"Command Palette…"
        action:action
        keyEquivalent:@"k"];
    [paletteItem setTarget:g_target];
    [paletteItem setTag:TAG_COMMAND_PALETTE];
    [viewMenu addItem:paletteItem];

    [viewMenu addItem:[NSMenuItem separatorItem]];
    
    NSMenuItem *explorerItem = [[NSMenuItem alloc]
        initWithTitle:@"Explorer"
        action:action
        keyEquivalent:@"e"];
    [explorerItem setTarget:g_target];
    [explorerItem setTag:TAG_EXPLORER];
    [viewMenu addItem:explorerItem];
    g_explorer_item = explorerItem;

    NSMenuItem *refreshItem = [[NSMenuItem alloc]
        initWithTitle:@"Refresh Explorer"
        action:action
        keyEquivalent:@"r"];
    [refreshItem setTarget:g_target];
    [refreshItem setTag:TAG_REFRESH_EXPLORER];
    [viewMenu addItem:refreshItem];
    g_refresh_explorer_item = refreshItem;

    [viewMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *closeAllItem = [[NSMenuItem alloc]
        initWithTitle:@"Close All Documents"
        action:action
        keyEquivalent:@"w"];
    [closeAllItem setKeyEquivalentModifierMask:NSEventModifierFlagCommand | NSEventModifierFlagOption];
    [closeAllItem setTarget:g_target];
    [closeAllItem setTag:TAG_CLOSE_ALL];
    [viewMenu addItem:closeAllItem];
    g_close_all_item = closeAllItem;

    /* WHY: Also alias Cmd+P to command palette for parity with VS Code */
    /* WHY: We use the same TAG and a different key equivalent. */
    /* WHY: To avoid duplication in the menu while supporting multiple shortcuts, */
    /* WHY: we can add it as a hidden item or just rely on the shell_ui.rs handling for Cmd+P. */
    /* WHY: However, to show it in the menu without a separate visible entry, */
    /* WHY: we use a hidden menu item or just skip it if we want it clean. */
    /* WHY: Let's add it as an 'alternate' to the same menu item if possible, */
    /* WHY: but Cocoa NSMenuItem doesn't easily support two shortcuts for one item. */
    /* WHY: A common trick is to add another item with the same action/tag but make it hidden. */
    NSMenuItem *paletteItem2 = [[NSMenuItem alloc]
        initWithTitle:@"Command Palette…"
        action:action
        keyEquivalent:@"p"];
    [paletteItem2 setTarget:g_target];
    [paletteItem2 setTag:TAG_COMMAND_PALETTE];
    [paletteItem2 setHidden:YES]; /* WHY: Hide from view but keep shortcut active */
    [viewMenu addItem:paletteItem2];
    
    g_command_palette_item = paletteItem;

    NSMenuItem *viewMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [viewMenuItem setSubmenu:viewMenu];

    /* WHY: --- Settings > Language --- */
    NSMenu *langMenu = [[NSMenu alloc] initWithTitle:@"Language"];
    g_language_menu = langMenu;

    NSMenuItem *enItem = [[NSMenuItem alloc]
        initWithTitle:@"English"
        action:action
        keyEquivalent:@""];
    [enItem setTarget:g_target];
    [enItem setTag:TAG_LANG_EN];
    [langMenu addItem:enItem];

    NSMenuItem *jaItem = [[NSMenuItem alloc]
        initWithTitle:@"日本語"
        action:action
        keyEquivalent:@""];
    [jaItem setTarget:g_target];
    [jaItem setTag:TAG_LANG_JA];
    [langMenu addItem:jaItem];

    NSMenuItem *zhCNItem = [[NSMenuItem alloc] initWithTitle:@"简体中文" action:action keyEquivalent:@""];
    [zhCNItem setTarget:g_target];
    [zhCNItem setTag:TAG_LANG_ZH_CN];
    [langMenu addItem:zhCNItem];

    NSMenuItem *zhTWItem = [[NSMenuItem alloc] initWithTitle:@"繁體中文" action:action keyEquivalent:@""];
    [zhTWItem setTarget:g_target];
    [zhTWItem setTag:TAG_LANG_ZH_TW];
    [langMenu addItem:zhTWItem];

    NSMenuItem *koItem = [[NSMenuItem alloc] initWithTitle:@"한국어" action:action keyEquivalent:@""];
    [koItem setTarget:g_target];
    [koItem setTag:TAG_LANG_KO];
    [langMenu addItem:koItem];

    NSMenuItem *ptItem = [[NSMenuItem alloc] initWithTitle:@"Português" action:action keyEquivalent:@""];
    [ptItem setTarget:g_target];
    [ptItem setTag:TAG_LANG_PT];
    [langMenu addItem:ptItem];

    NSMenuItem *frItem = [[NSMenuItem alloc] initWithTitle:@"Français" action:action keyEquivalent:@""];
    [frItem setTarget:g_target];
    [frItem setTag:TAG_LANG_FR];
    [langMenu addItem:frItem];

    NSMenuItem *deItem = [[NSMenuItem alloc] initWithTitle:@"Deutsch" action:action keyEquivalent:@""];
    [deItem setTarget:g_target];
    [deItem setTag:TAG_LANG_DE];
    [langMenu addItem:deItem];

    NSMenuItem *esItem = [[NSMenuItem alloc] initWithTitle:@"Español" action:action keyEquivalent:@""];
    [esItem setTarget:g_target];
    [esItem setTag:TAG_LANG_ES];
    [langMenu addItem:esItem];

    NSMenuItem *itItem = [[NSMenuItem alloc] initWithTitle:@"Italiano" action:action keyEquivalent:@""];
    [itItem setTarget:g_target];
    [itItem setTag:TAG_LANG_IT];
    [langMenu addItem:itItem];

    NSMenuItem *langMenuItem = [[NSMenuItem alloc] initWithTitle:@"Language" action:nil keyEquivalent:@""];
    [langMenuItem setSubmenu:langMenu];

    NSMenu *settingsMenu = [[NSMenu alloc] initWithTitle:@"Settings"];
    g_settings_menu = settingsMenu;

    NSMenuItem *prefsItem = [[NSMenuItem alloc]
        initWithTitle:@"Preferences…"
        action:action
        keyEquivalent:@","];
    [prefsItem setTarget:g_target];
    [prefsItem setTag:TAG_SETTINGS];
    [settingsMenu addItem:prefsItem];
    g_preferences_item = prefsItem;

    [settingsMenu addItem:[NSMenuItem separatorItem]];
    [settingsMenu addItem:langMenuItem];

    NSMenuItem *settingsMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [settingsMenuItem setSubmenu:settingsMenu];

    /* WHY: --- Help Menu --- */
    NSMenu *helpMenu = [[NSMenu alloc] initWithTitle:@"Help"];
    g_help_menu = helpMenu;
    
    NSMenuItem *releaseNotesItem = [[NSMenuItem alloc] 
        initWithTitle:@"Release Notes" 
        action:action 
        keyEquivalent:@""];
    [releaseNotesItem setTarget:g_target];
    [releaseNotesItem setTag:TAG_RELEASE_NOTES];
    [helpMenu addItem:releaseNotesItem];
    g_release_notes_item = releaseNotesItem;

    [helpMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *welcomeItem = [[NSMenuItem alloc]
        initWithTitle:@"Welcome Screen"
        action:action
        keyEquivalent:@""];
    [welcomeItem setTarget:g_target];
    [welcomeItem setTag:TAG_WELCOME_SCREEN];
    [helpMenu addItem:welcomeItem];
    g_welcome_item = welcomeItem;

    NSMenuItem *guideItem = [[NSMenuItem alloc]
        initWithTitle:@"User Guide"
        action:action
        keyEquivalent:@""];
    [guideItem setTarget:g_target];
    [guideItem setTag:TAG_USER_GUIDE];
    [helpMenu addItem:guideItem];
    g_guide_item = guideItem;

    NSMenuItem *demoItem = [[NSMenuItem alloc]
        initWithTitle:@"Demo"
        action:action
        keyEquivalent:@"d"];
    [demoItem setKeyEquivalentModifierMask:NSEventModifierFlagCommand | NSEventModifierFlagOption];
    [demoItem setTarget:g_target];
    [demoItem setTag:TAG_DEMO];
    [helpMenu addItem:demoItem];
    g_demo_item = demoItem;

    [helpMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *githubItem = [[NSMenuItem alloc] 
        initWithTitle:@"GitHub Repository" 
        action:action 
        keyEquivalent:@""];
    [githubItem setTarget:g_target];
    [githubItem setTag:TAG_GITHUB];
    [helpMenu addItem:githubItem];
    g_github_item = githubItem;

    NSMenuItem *helpMenuItem = [[NSMenuItem alloc] initWithTitle:@"Help" action:nil keyEquivalent:@""];
    [helpMenuItem setSubmenu:helpMenu];

    /* WHY: --- Build Main Menu --- */
    NSMenu *mainMenu = [[NSMenu alloc] initWithTitle:@""];
    [NSApp setMainMenu:mainMenu];
    [mainMenu addItem:appMenuItem];
    [mainMenu addItem:fileMenuItem];
    [mainMenu addItem:viewMenuItem];
    [mainMenu addItem:settingsMenuItem];
    [mainMenu addItem:helpMenuItem];

    /* WHY: Prevent macOS from auto-injecting the Spotlight Search box into our custom "Help" menu */
    /* WHY: by explicitly giving it a dummy, detached Help Menu. */
    NSMenu *dummyHelpMenu = [[NSMenu alloc] initWithTitle:@"DummyHelp"];
    [NSApp setHelpMenu:dummyHelpMenu];
}

/// Called from Rust: Gets and resets the last menu action.
/// Return value: 0 = No action, otherwise = TAG_* constant.
int katana_poll_menu_action(void) {
    int action = g_last_action;
    g_last_action = 0;
    return action;
}

/// Called from Rust to dynamically update menu strings for i18n
void katana_update_menu_strings(
    const char* file, 
    const char* open_workspace, 
    const char* save, 
    const char* settings, 
    const char* preferences, 
    const char* language,
    const char* about,
    const char* quit,
    const char* check_updates,
    const char* help,
    const char* release_notes,
    const char* command_palette,
    const char* view,
    const char* demo,
    const char* welcome_screen,
    const char* user_guide,
    const char* close_workspace,
    const char* explorer,
    const char* refresh_explorer,
    const char* close_all,
    const char* github
) {
    @autoreleasepool {
        if (g_file_menu && file) {
            [g_file_menu setTitle:[NSString stringWithUTF8String:file]];
        }
        if (g_open_workspace_item && open_workspace) {
            [g_open_workspace_item setTitle:[NSString stringWithUTF8String:open_workspace]];
        }
        if (g_close_workspace_item && close_workspace) {
            [g_close_workspace_item setTitle:[NSString stringWithUTF8String:close_workspace]];
        }
        if (g_save_item && save) {
            [g_save_item setTitle:[NSString stringWithUTF8String:save]];
        }
        if (g_view_menu && view) {
            [g_view_menu setTitle:[NSString stringWithUTF8String:view]];
        }
        if (g_explorer_item && explorer) {
            [g_explorer_item setTitle:[NSString stringWithUTF8String:explorer]];
        }
        if (g_refresh_explorer_item && refresh_explorer) {
            [g_refresh_explorer_item setTitle:[NSString stringWithUTF8String:refresh_explorer]];
        }
        if (g_close_all_item && close_all) {
            [g_close_all_item setTitle:[NSString stringWithUTF8String:close_all]];
        }
        if (g_command_palette_item && command_palette) {
            [g_command_palette_item setTitle:[NSString stringWithUTF8String:command_palette]];
            /* WHY: Also update the hidden parallel shortcut item if needed */
            for (NSMenuItem *item in [g_view_menu itemArray]) {
                if ([item tag] == TAG_COMMAND_PALETTE) {
                    [item setTitle:[NSString stringWithUTF8String:command_palette]];
                }
            }
        }
        if (g_settings_menu && settings) {
            [g_settings_menu setTitle:[NSString stringWithUTF8String:settings]];
        }
        if (g_preferences_item && preferences) {
            [g_preferences_item setTitle:[NSString stringWithUTF8String:preferences]];
        }
        if (g_language_menu && language) {
            [g_language_menu setTitle:[NSString stringWithUTF8String:language]];
        }
        if (g_about_item && about) {
            [g_about_item setTitle:[NSString stringWithUTF8String:about]];
        }
        if (g_check_updates_item && check_updates) {
            [g_check_updates_item setTitle:[NSString stringWithUTF8String:check_updates]];
        }
        if (g_quit_item && quit) {
            [g_quit_item setTitle:[NSString stringWithUTF8String:quit]];
        }
        if (g_help_menu && help) {
            [g_help_menu setTitle:[NSString stringWithUTF8String:help]];
        }
        if (g_release_notes_item && release_notes) {
            [g_release_notes_item setTitle:[NSString stringWithUTF8String:release_notes]];
        }
        if (g_welcome_item && welcome_screen) {
            [g_welcome_item setTitle:[NSString stringWithUTF8String:welcome_screen]];
        }
        if (g_guide_item && user_guide) {
            [g_guide_item setTitle:[NSString stringWithUTF8String:user_guide]];
        }
        if (g_github_item && github) {
            [g_github_item setTitle:[NSString stringWithUTF8String:github]];
        }
        if (g_demo_item && demo) {
            [g_demo_item setTitle:[NSString stringWithUTF8String:demo]];
        }
    }
}

void katana_update_menu_state(bool save_enabled, bool close_workspace_enabled, bool refresh_explorer_enabled, bool close_all_enabled) {
    @autoreleasepool {
        if (g_save_item) {
            [g_save_item setEnabled:(save_enabled ? YES : NO)];
        }
        if (g_close_workspace_item) {
            [g_close_workspace_item setEnabled:(close_workspace_enabled ? YES : NO)];
        }
        if (g_refresh_explorer_item) {
            [g_refresh_explorer_item setEnabled:(refresh_explorer_enabled ? YES : NO)];
        }
        if (g_close_all_item) {
            [g_close_all_item setEnabled:(close_all_enabled ? YES : NO)];
        }
    }
}

static NSImage *g_app_icon = nil;

/// Called from Rust: Sets the application and dock icon from PNG bytes.
void katana_set_app_icon_png(const unsigned char *png_data, unsigned long png_len) {
    @autoreleasepool {
        NSData *data = [NSData dataWithBytes:png_data length:png_len];
        NSImage *image = [[NSImage alloc] initWithData:data];
        if (image) {
            g_app_icon = image;
            [NSApp setApplicationIconImage:image];
        }
    }
}

/// Called from Rust: Gets the current user locale from macOS.
void katana_get_mac_locale(char *buf, size_t max_len) {
    if (buf == NULL || max_len == 0) return;
    buf[0] = '\0';
    @autoreleasepool {
        NSArray *languages = [NSLocale preferredLanguages];
        if (languages != nil && languages.count > 0) {
            NSString *preferred = languages[0];
            const char *utf8 = [preferred UTF8String];
            if (utf8) {
                strncpy(buf, utf8, max_len - 1);
                buf[max_len - 1] = '\0';
            }
        }
    }
}
