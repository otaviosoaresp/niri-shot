use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{spawn_error, CaptureError};

pub(super) fn capture() -> Result<Vec<u8>, CaptureError> {
    let last_region = load_last_region();

    let mut slurp_cmd = Command::new("slurp");
    slurp_cmd.stdout(Stdio::piped());

    if last_region.is_some() {
        slurp_cmd.stdin(Stdio::piped()).arg("-B").arg("#3daee966");
    }

    let mut child = slurp_cmd.spawn().map_err(|e| spawn_error("slurp", e))?;

    if let Some(ref geometry) = last_region {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = writeln!(stdin, "{}", geometry);
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| CaptureError::Failed(format!("slurp failed: {}", e)))?;
    let geometry = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if !output.status.success() || geometry.is_empty() {
        return Err(CaptureError::Cancelled);
    }

    let _ = save_last_region(&geometry);

    capture_geometry(&geometry)
}

fn capture_geometry(geometry: &str) -> Result<Vec<u8>, CaptureError> {
    let output = Command::new("grim")
        .arg("-g")
        .arg(geometry)
        .arg("-")
        .output()
        .map_err(|e| spawn_error("grim", e))?;

    if !output.status.success() {
        return Err(CaptureError::Failed(format!(
            "grim failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(output.stdout)
}

fn cache_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "github", "niri-shot")
        .map(|dirs| dirs.cache_dir().to_path_buf())
}

fn save_last_region(geometry: &str) -> std::io::Result<()> {
    if let Some(cache_dir) = cache_dir() {
        fs::create_dir_all(&cache_dir)?;
        fs::write(cache_dir.join("last-region"), geometry)?;
    }
    Ok(())
}

fn load_last_region() -> Option<String> {
    cache_dir()
        .map(|dir| dir.join("last-region"))
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
