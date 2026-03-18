#import <Cocoa/Cocoa.h>

/// Returns the name of the NSApplication effective appearance as a null-terminated C string.
///
/// Returns NULL if NSApp is not yet initialized.
const char *katana_macos_appearance_name(void) {
    @autoreleasepool {
        if (NSApp == nil) {
            return NULL;
        }
        NSAppearance *appearance = [NSApp effectiveAppearance];
        if (appearance == nil) {
            return NULL;
        }
        // NSAppearanceName is an NSString typedef; -UTF8String returns a transient C string.
        // The caller must not cache this pointer beyond the autorelease pool lifetime.
        return [[appearance name] UTF8String];
    }
}
