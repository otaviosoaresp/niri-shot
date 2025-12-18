#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub save_directory: PathBuf,
    pub filename_template: String,
    pub default_format: ImageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpg,
}

impl Default for Config {
    fn default() -> Self {
        let pictures_dir = directories::UserDirs::new()
            .and_then(|dirs| dirs.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Pictures")
            });

        Self {
            save_directory: pictures_dir.join("Screenshots"),
            filename_template: "screenshot-%Y-%m-%d-%H%M%S".to_string(),
            default_format: ImageFormat::Png,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "github", "niri-shot")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    fn cache_dir() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "github", "niri-shot")
            .map(|dirs| dirs.cache_dir().to_path_buf())
    }

    pub fn save_last_region(geometry: &str) -> anyhow::Result<()> {
        if let Some(cache_dir) = Self::cache_dir() {
            fs::create_dir_all(&cache_dir)?;
            let path = cache_dir.join("last-region");
            fs::write(path, geometry)?;
        }
        Ok(())
    }

    pub fn load_last_region() -> Option<String> {
        Self::cache_dir()
            .map(|dir| dir.join("last-region"))
            .and_then(|path| fs::read_to_string(path).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}
