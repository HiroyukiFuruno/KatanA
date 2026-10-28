use super::create_build_command;
use std::io;
use std::path::Path;

pub(super) struct WindowsResources;

impl WindowsResources {
    pub(super) fn compile(resources: &winres::WindowsResource) -> io::Result<()> {
        if cfg!(windows) || std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
            return resources.compile();
        }
        Self::compile_cross_msvc(resources)
    }

    fn compile_cross_msvc(resources: &winres::WindowsResource) -> io::Result<()> {
        let directory = std::env::var("OUT_DIR").map_err(io::Error::other)?;
        let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?;
        let source = Path::new(&directory).join("resource.rc");
        resources.write_resource_file(&source)?;
        /* WHY: cross検査でもmanifestを省略せず、SDK実行ファイルではなくLLVMで同じresourceを生成する。 */
        let result = create_build_command("llvm-rc")
            .current_dir(&directory)
            .arg("/I")
            .arg(&manifest)
            .args(["/fo", "resource.lib", "resource.rc"])
            .output()?;
        if !result.status.success() {
            return Err(io::Error::other(format!(
                "llvm-rc failed: {}: {}",
                result.status,
                String::from_utf8_lossy(&result.stderr)
            )));
        }
        println!("cargo:rustc-link-search=native={directory}");
        println!("cargo:rustc-link-lib=dylib=resource");
        Ok(())
    }
}
