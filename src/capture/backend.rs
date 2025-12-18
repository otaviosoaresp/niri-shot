use crate::config::Config;
use anyhow::{anyhow, Result};
use std::io::{self, IsTerminal, Write};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum CaptureMode {
    Fullscreen,
    Region,
    Window,
}

pub struct CaptureBackend;

impl CaptureBackend {
    pub fn capture(mode: CaptureMode) -> Result<Vec<u8>> {
        match mode {
            CaptureMode::Fullscreen => Self::capture_fullscreen(),
            CaptureMode::Region => Self::capture_region(),
            CaptureMode::Window => Self::capture_window(),
        }
    }

    fn capture_fullscreen() -> Result<Vec<u8>> {
        let output = Command::new("grim").arg("-").output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "grim failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
    }

    fn capture_region() -> Result<Vec<u8>> {
        if io::stdin().is_terminal() {
            if let Some(last) = Config::load_last_region() {
                print!("Previous region: {}. Enter to use, any key for new: ", last);
                io::stdout().flush().ok();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() && input.trim().is_empty() {
                    return Self::capture_geometry(&last);
                }
            }
        }

        let slurp = Command::new("slurp").output()?;

        if !slurp.status.success() {
            return Err(anyhow!("Selection cancelled"));
        }

        let geometry = String::from_utf8_lossy(&slurp.stdout).trim().to_string();

        if geometry.is_empty() {
            return Err(anyhow!("No region selected"));
        }

        let _ = Config::save_last_region(&geometry);

        Self::capture_geometry(&geometry)
    }

    fn capture_geometry(geometry: &str) -> Result<Vec<u8>> {
        let output = Command::new("grim")
            .arg("-g")
            .arg(geometry)
            .arg("-")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "grim failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
    }

    fn capture_window() -> Result<Vec<u8>> {
        let slurp = Command::new("slurp").arg("-o").output()?;

        if !slurp.status.success() {
            return Err(anyhow!("Selection cancelled"));
        }

        let geometry = String::from_utf8_lossy(&slurp.stdout).trim().to_string();

        if geometry.is_empty() {
            return Err(anyhow!("No window selected"));
        }

        Self::capture_geometry(&geometry)
    }

    #[allow(dead_code)]
    pub fn is_available() -> bool {
        Command::new("which")
            .arg("grim")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            && Command::new("which")
                .arg("slurp")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }
}
