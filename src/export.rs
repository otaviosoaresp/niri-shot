use anyhow::Result;
use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn save_png(data: &[u8]) -> Result<PathBuf> {
    let dir = screenshots_dir();
    fs::create_dir_all(&dir)?;

    let stem = format!("screenshot-{}", Local::now().format("%Y-%m-%d-%H%M%S"));
    let path = unique_path(&dir, &stem, "png");

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

fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let plain = dir.join(format!("{}.{}", stem, ext));
    if !plain.exists() {
        return plain;
    }

    let mut n = 1;
    loop {
        let candidate = dir.join(format!("{}-{}.{}", stem, n, ext));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

fn screenshots_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.picture_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Pictures")
        })
        .join("Screenshots")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("niri-shot-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn unique_path_uses_the_plain_name_when_free() {
        let dir = scratch_dir("free");
        assert_eq!(unique_path(&dir, "shot", "png"), dir.join("shot.png"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unique_path_appends_a_counter_on_collision() {
        let dir = scratch_dir("collision");
        fs::write(dir.join("shot.png"), b"x").unwrap();
        assert_eq!(unique_path(&dir, "shot", "png"), dir.join("shot-1.png"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unique_path_skips_taken_counters() {
        let dir = scratch_dir("taken");
        fs::write(dir.join("shot.png"), b"x").unwrap();
        fs::write(dir.join("shot-1.png"), b"x").unwrap();
        assert_eq!(unique_path(&dir, "shot", "png"), dir.join("shot-2.png"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
