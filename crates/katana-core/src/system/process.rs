use std::process::{Child, Command, ExitStatus, Output};

use super::types::ProcessService;

impl ProcessService {
    pub fn create_command(program: &str) -> Command {
        #[allow(unused_mut)]
        let mut command = Command::new(program);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            /* WHY: CREATE_NO_WINDOW flag (0x08000000) prevents a console window from popping up. */
            /* This is legally enforced across the entire KatanA codebase for background processes. */
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        command
    }

    /// Spawns the command as a child process.
    pub fn spawn(mut command: Command) -> std::io::Result<Child> {
        command.spawn()
    }

    /// Runs the command and waits for it to complete.
    pub fn status(mut command: Command) -> std::io::Result<ExitStatus> {
        command.status()
    }

    /// Returns the output of the command.
    pub fn output(mut command: Command) -> std::io::Result<Output> {
        command.output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_command() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let cmd = ProcessService::create_command("echo");
        assert!(cmd.get_program().to_string_lossy().contains("echo"));
    }

    #[test]
    fn test_output() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let mut cmd = ProcessService::create_command("echo");
        cmd.arg("hello");
        let output = ProcessService::output(cmd).unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[test]
    fn test_status() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let mut cmd = ProcessService::create_command("echo");
        cmd.arg("hello");
        let status = ProcessService::status(cmd).unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_spawn() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let mut cmd = ProcessService::create_command("sleep");
        cmd.arg("0.1");
        let mut child = ProcessService::spawn(cmd).unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }

    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
}
