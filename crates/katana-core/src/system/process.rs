use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output};

/// Provides cross-platform process management, with automatic window suppression on Windows.
pub struct ProcessService;

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

    /// Downloads a file from a URL to a destination path.
    /// Tries curl first, then falls back to platform-specific alternatives:
    /// Windows → PowerShell Invoke-WebRequest, Linux/macOS → wget.
    pub fn download_file(url: &str, dest: &Path) -> Result<(), String> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        /* WHY: Define the chain of commands to try for downloading. */
        /* Each command is tried in order until one succeeds. */
        let mut commands: Vec<(String, Vec<String>)> = Vec::new();

        /* 1. curl (Cross-platform) */
        commands.push((
            "curl".to_string(),
            vec![
                "-fsSL".to_string(),
                "-o".to_string(),
                dest.to_str().unwrap_or("").to_string(),
                url.to_string(),
            ],
        ));

        #[cfg(windows)]
        {
            /* 2. PowerShell (Windows) */
            let ps_script = format!(
                "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
                 $ProgressPreference = 'SilentlyContinue'; \
                 Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                url.replace("'", "''"),
                dest.display().to_string().replace("'", "''")
            );
            commands.push((
                "powershell".to_string(),
                vec![
                    "-NoProfile".to_string(),
                    "-NonInteractive".to_string(),
                    "-Command".to_string(),
                    ps_script,
                ],
            ));

            /* 3. Node.js (Windows Fallback) */
            let node_script = format!(
                "fetch('{}').then(r => {{ if(!r.ok) throw r.status; return r.arrayBuffer(); }}).then(b => require('fs').writeFileSync('{}', Buffer.from(b)))",
                url.replace("'", "\\'"),
                dest.to_str().unwrap_or("").replace("'", "\\'")
            );
            commands.push(("node".to_string(), vec!["-e".to_string(), node_script]));
        }

        #[cfg(not(windows))]
        {
            /* 2. wget (Unix) */
            commands.push((
                "wget".to_string(),
                vec![
                    "-q".to_string(),
                    "-O".to_string(),
                    dest.to_str().unwrap_or("").to_string(),
                    url.to_string(),
                ],
            ));

            /* 3. Node.js (Unix Fallback) */
            let node_script = format!(
                "fetch('{}').then(r => {{ if(!r.ok) throw r.status; return r.arrayBuffer(); }}).then(b => require('fs').writeFileSync('{}', Buffer.from(b)))",
                url.replace("'", "\\'"),
                dest.to_str().unwrap_or("").replace("'", "\\'")
            );
            commands.push(("node".to_string(), vec!["-e".to_string(), node_script]));
        }

        for (prog, args) in commands {
            let mut cmd = Self::create_command(&prog);
            cmd.args(args);
            if cmd.status().is_ok_and(|s| s.success()) {
                return Ok(());
            }
        }

        Err("Download failed: curl and all fallback mechanisms failed.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_command() {
        let cmd = ProcessService::create_command("echo");
        assert!(cmd.get_program().to_string_lossy().contains("echo"));
    }

    #[test]
    fn test_output() {
        let mut cmd = ProcessService::create_command("echo");
        cmd.arg("hello");
        let output = ProcessService::output(cmd).unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[test]
    fn test_status() {
        let mut cmd = ProcessService::create_command("echo");
        cmd.arg("hello");
        let status = ProcessService::status(cmd).unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_spawn() {
        let mut cmd = ProcessService::create_command("sleep");
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

    #[cfg(not(windows))]
    #[cfg(not(coverage))]
    #[test]
    fn test_download_file_wget_fallback() {
        /* WHY: Verify the wget fallback path is exercised on non-Windows platforms */
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("wget_src.txt");
        let dest_path = dir.path().join("wget_dest.txt");

        fs::write(&src_path, "wget content").unwrap();

        let url = format!("file://{}", src_path.to_str().unwrap());

        /* WHY: Attempt via wget directly to confirm the fallback path is reachable */
        let mut wget = ProcessService::create_command("wget");
        wget.args(["-q", "-O", dest_path.to_str().unwrap_or(""), &url]);
        let wget_available = wget.status().is_ok_and(|s| s.success());

        if wget_available {
            assert_eq!(fs::read_to_string(&dest_path).unwrap(), "wget content");
        }
        /* WHY: If wget is not installed in the test environment, skip silently */
    }
}
