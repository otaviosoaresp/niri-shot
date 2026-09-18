use anyhow::{anyhow, Result};
use std::process::Command;

pub(super) fn capture_fullscreen() -> Result<Vec<u8>> {
    let output = Command::new("grim").arg("-").output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "grim failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output.stdout)
}

pub(super) fn capture_window() -> Result<Vec<u8>> {
    let slurp = Command::new("slurp").arg("-o").output()?;

    if !slurp.status.success() {
        return Err(anyhow!("Selection cancelled"));
    }

    let geometry = String::from_utf8_lossy(&slurp.stdout).trim().to_string();

    if geometry.is_empty() {
        return Err(anyhow!("No window selected"));
    }

    super::region::capture_geometry(&geometry)
}
