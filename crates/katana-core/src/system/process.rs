use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output};

/// Provides cross-platform process management, with automatic window suppression on Windows.
pub struct ProcessService;

impl ProcessService {
    /// Creates a new `Command` for the given program, configured to suppress the console window on Windows if `hidden` is true.
    pub fn create_command(program: &str, _hidden: bool) -> Command {
        #[allow(unused_mut)]
        let mut command = Command::new(program);
        #[cfg(windows)]
        if _hidden {
            use std::os::windows::process::CommandExt;
            /* WHY: CREATE_NO_WINDOW flag (0x08000000) prevents a console window from popping up. */
            /* This is essential for GUI applications calling CLI tools in the background. */
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

    /// Downloads a file from a URL to a destination path, using curl with a PowerShell fallback on Windows.
    pub fn download_file(url: &str, dest: &Path) -> Result<(), String> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        /* Attempt download with curl */
        let mut curl = Self::create_command("curl", true);
        curl.args(["-L", "-o", dest.to_str().unwrap_or(""), url]);

        match curl.status() {
            Ok(status) if status.success() => return Ok(()),
            _ => {
                #[cfg(windows)]
                {
                    /* Fallback to PowerShell's Invoke-WebRequest if curl fails on Windows */
                    let mut ps = Self::create_command("powershell", true);
                    ps.args([
                        "-Command",
                        &format!(
                            "Invoke-WebRequest -Uri \"{}\" -OutFile \"{}\"",
                            url,
                            dest.display()
                        ),
                    ]);
                    if ps.status().is_ok_and(|s| s.success()) {
                        return Ok(());
                    }
                }
            }
        }

        Err("Download failed: Both curl and fallback mechanisms failed.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_command() {
        let cmd = ProcessService::create_command("echo", true);
        assert!(cmd.get_program().to_string_lossy().contains("echo"));
    }

    #[test]
    fn test_output() {
        let mut cmd = ProcessService::create_command("echo", true);
        cmd.arg("hello");
        let output = ProcessService::output(cmd).unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[test]
    fn test_status() {
        let mut cmd = ProcessService::create_command("echo", true);
        cmd.arg("hello");
        let status = ProcessService::status(cmd).unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_spawn() {
        let mut cmd = ProcessService::create_command("sleep", true);
        cmd.arg("0.1");
        let mut child = ProcessService::spawn(cmd).unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_download_file_local() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("src.txt");
        let dest_path = dir.path().join("dest.txt");

        fs::write(&src_path, "test content").unwrap();

        /* WHY: Use file:// URL to test download_file logic with curl */
        let url = format!("file://{}", src_path.to_str().unwrap());
        let result = ProcessService::download_file(&url, &dest_path);

        assert!(result.is_ok(), "Local download failed");
        assert_eq!(fs::read_to_string(dest_path).unwrap(), "test content");
    }

    #[test]
    fn test_download_file_fail() {
        let dir = tempdir().unwrap();
        let dest_path = dir.path().join("nonexistent.txt");

        /* WHY: This should fail both curl and fallback */
        let result =
            ProcessService::download_file("https://invalid.domain.example/none", &dest_path);
        assert!(result.is_err());
    }
}
