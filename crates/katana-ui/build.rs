// macOS ネイティブメニューバーのビルドスクリプト。
// Objective-C ファイル (macos_menu.m) をコンパイルしてリンクする。

fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/macos_menu.m")
            .flag("-fobjc-arc")
            .compile("macos_menu");

        println!("cargo:rustc-link-lib=framework=Cocoa");
    }
}
