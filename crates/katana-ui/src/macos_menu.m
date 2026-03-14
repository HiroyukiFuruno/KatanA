#import <Cocoa/Cocoa.h>

// メニューアクションの識別用タグ定数。
// Rust 側の MenuAction enum と一致させる。
enum {
    TAG_OPEN_WORKSPACE = 1,
    TAG_SAVE           = 2,
    TAG_LANG_EN        = 3,
    TAG_LANG_JA        = 4,
};

// グローバル：最後に選択されたメニューアクションのタグ。
// Rust 側から polling で読み取る。
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

/// Rust から呼ばれる: ネイティブメニューバーを構築する。
void katana_setup_native_menu(void) {
    g_target = [[KatanaMenuTarget alloc] init];
    SEL action = @selector(menuAction:);

    // --- File メニュー ---
    NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];

    NSMenuItem *openItem = [[NSMenuItem alloc]
        initWithTitle:@"Open Workspace…"
        action:action
        keyEquivalent:@"o"];
    [openItem setTarget:g_target];
    [openItem setTag:TAG_OPEN_WORKSPACE];
    [fileMenu addItem:openItem];

    [fileMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *saveItem = [[NSMenuItem alloc]
        initWithTitle:@"Save"
        action:action
        keyEquivalent:@"s"];
    [saveItem setTarget:g_target];
    [saveItem setTag:TAG_SAVE];
    [fileMenu addItem:saveItem];

    NSMenuItem *fileMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [fileMenuItem setSubmenu:fileMenu];

    // --- Settings > Language メニュー ---
    NSMenu *langMenu = [[NSMenu alloc] initWithTitle:@"Language"];

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

    NSMenuItem *langMenuItem = [[NSMenuItem alloc] initWithTitle:@"Language" action:nil keyEquivalent:@""];
    [langMenuItem setSubmenu:langMenu];

    NSMenu *settingsMenu = [[NSMenu alloc] initWithTitle:@"Settings"];
    [settingsMenu addItem:langMenuItem];

    NSMenuItem *settingsMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [settingsMenuItem setSubmenu:settingsMenu];

    // --- メインメニューに追加 ---
    NSMenu *mainMenu = [NSApp mainMenu];
    if (!mainMenu) {
        mainMenu = [[NSMenu alloc] initWithTitle:@""];
        [NSApp setMainMenu:mainMenu];
    }
    [mainMenu addItem:fileMenuItem];
    [mainMenu addItem:settingsMenuItem];
}

/// Rust から呼ばれる: 最後のメニューアクションを取得してリセットする。
/// 戻り値: 0 = アクションなし, それ以外 = TAG_* 定数。
int katana_poll_menu_action(void) {
    int action = g_last_action;
    g_last_action = 0;
    return action;
}
