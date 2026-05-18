/* WHY: Build-script-only helper for spawning external processes without a Windows console window.
 * This file is included via `include!()` from build.rs files so they do not need to depend on
 * `katana-core` (which would create a build-time circular dependency). It mirrors
 * `katana_core::system::ProcessService::create_command` semantics but is intentionally
 * minimal (no facade type, no extra methods) because build scripts only need command construction.
 *
 * The AST lint rule `no-direct-process-command` allows direct `Command::new` only inside this
 * file and inside `crates/katana-core/src/system/process.rs`. */

#[allow(dead_code)]
fn create_build_command(program: &str) -> std::process::Command {
    #[allow(unused_mut)]
    let mut command = std::process::Command::new(program);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}
