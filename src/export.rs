use anyhow::Result;
use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn save_png(data: &[u8]) -> Result<PathBuf> {
    let dir = screenshots_dir();
    fs::create_dir_all(&dir)?;

    let filename = format!("screenshot-{}.png", Local::now().format("%Y-%m-%d-%H%M%S"));
    let path = dir.join(filename);

    fs::write(&path, data)?;
    println!("Saved to: {}", path.display());

    Ok(path)
}

pub fn copy_png(data: &[u8]) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(data)?;
    }

    child.wait()?;
    Ok(())
}

fn screenshots_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.picture_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Pictures")
        })
        .join("Screenshots")
}
