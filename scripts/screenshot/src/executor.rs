use crate::{capture, request::Step};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

pub struct Session {
    binary: PathBuf,
    home_dir: PathBuf,
    child: Option<Child>,
}

impl Session {
    pub fn new(binary: &Path, home_dir: &Path) -> Self {
        Self {
            binary: binary.to_owned(),
            home_dir: home_dir.to_owned(),
            child: None,
        }
    }

    pub fn is_running(&mut self) -> bool {
        self.child.as_mut().map_or(false, |c| c.try_wait().ok().flatten().is_none())
    }

    pub fn launch(&mut self, viewport: Option<(u32, u32)>, wait_secs: f64) -> Result<()> {
        let mut cmd = Command::new(&self.binary);
        cmd.env("HOME", &self.home_dir);

        let config_dir = if cfg!(target_os = "macos") {
            self.home_dir.join("Library").join("Application Support").join("KatanA")
        } else if cfg!(target_os = "windows") {
            self.home_dir.join("AppData").join("Roaming").join("KatanA")
        } else {
            self.home_dir.join(".config").join("KatanA")
        };
        cmd.env("KATANA_CONFIG_DIR", &config_dir);

        if cfg!(target_os = "linux") {
            cmd.env("XDG_CONFIG_HOME", self.home_dir.join(".config"));
        }
        if let Some((w, h)) = viewport {
            cmd.env("KATANA_SCREENSHOT_WIDTH", w.to_string());
            cmd.env("KATANA_SCREENSHOT_HEIGHT", h.to_string());
        }
        cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());

        let child = cmd.spawn()?;
        println!("  launched KatanA (pid={})", child.id());
        self.child = Some(child);

        thread::sleep(Duration::from_secs_f64(wait_secs));
        Ok(())
    }

    pub fn quit(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            println!("  KatanA quit");
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.quit();
    }
}

pub fn run(
    steps: &[Step],
    output_dir: &Path,
    binary: &Path,
    home_dir: &Path,
) -> Result<()> {
    let mut session = Session::new(binary, home_dir);

    for (i, step) in steps.iter().enumerate() {
        let label = step_label(step);
        println!("step {}/{}: {label}", i + 1, steps.len());

        match step {
            Step::Launch(s) => {
                let vp = s.viewport.map(|v| (v.width, v.height));
                session.launch(vp, s.wait_seconds)?;
            }
            Step::Wait(s) => {
                println!("  sleeping {:.1}s", s.seconds);
                thread::sleep(Duration::from_secs_f64(s.seconds));
            }
            Step::Screenshot(s) => {
                if !session.is_running() {
                    bail!("screenshot step requires a running KatanA (add a 'launch' step first)");
                }
                let out = output_dir.join(format!("{}.png", s.output_name));
                capture::screenshot("KatanA", &out)?;
                println!("  saved: {}", out.display());
            }
            Step::OpenFile(s) => {
                if !session.is_running() {
                    bail!("open_file step requires a running KatanA");
                }
                println!("  NOTE: place {:?} in fixture.workspace_files — KatanA auto-loads the workspace on launch", s.file_name);
                thread::sleep(Duration::from_secs_f64(s.wait_seconds));
            }
            Step::Quit => {
                session.quit();
            }
        }
    }

    Ok(())
}

fn step_label(step: &Step) -> &'static str {
    match step {
        Step::Launch(_) => "launch",
        Step::Wait(_) => "wait",
        Step::Screenshot(_) => "screenshot",
        Step::OpenFile(_) => "open_file",
        Step::Quit => "quit",
    }
}
