use anyhow::{anyhow, Result};
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
        let output = Command::new("grim")
            .arg("-")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "grim falhou: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
    }

    fn capture_region() -> Result<Vec<u8>> {
        let slurp = Command::new("slurp")
            .output()?;

        if !slurp.status.success() {
            return Err(anyhow!("Selecao cancelada"));
        }

        let geometry = String::from_utf8_lossy(&slurp.stdout)
            .trim()
            .to_string();

        if geometry.is_empty() {
            return Err(anyhow!("Nenhuma regiao selecionada"));
        }

        let output = Command::new("grim")
            .arg("-g")
            .arg(&geometry)
            .arg("-")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "grim falhou: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
    }

    fn capture_window() -> Result<Vec<u8>> {
        let slurp = Command::new("slurp")
            .arg("-o")
            .output()?;

        if !slurp.status.success() {
            return Err(anyhow!("Selecao cancelada"));
        }

        let geometry = String::from_utf8_lossy(&slurp.stdout)
            .trim()
            .to_string();

        if geometry.is_empty() {
            return Err(anyhow!("Nenhuma janela selecionada"));
        }

        let output = Command::new("grim")
            .arg("-g")
            .arg(&geometry)
            .arg("-")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "grim falhou: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
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
