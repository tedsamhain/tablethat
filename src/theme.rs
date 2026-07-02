use crate::ThemeConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A theme loaded from a TOML file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeFile {
    pub name: String,
    pub theme: ThemeConfig,
}

/// Discover and load theme TOML files from a directory.
/// Searches in order:
/// 1. Explicit `themes_dir` path
/// 2. `./themes/` relative to cwd
/// 3. `~/.config/<app_name>/themes/`
pub fn load_themes(themes_dir: Option<&Path>, app_name: &str) -> Vec<ThemeFile> {
    let dirs: Vec<PathBuf> = if let Some(dir) = themes_dir {
        vec![dir.to_path_buf()]
    } else {
        let mut candidates = vec![PathBuf::from("themes")];
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", app_name) {
            candidates.push(proj_dirs.config_dir().join("themes"));
        }
        candidates
    };

    let mut themes = Vec::new();
    for dir in &dirs {
        if !dir.is_dir() {
            continue;
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        entries.sort();

        for path in entries {
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(theme) = toml::from_str::<ThemeFile>(&content)
            {
                themes.push(theme);
            }
        }
        if !themes.is_empty() {
            break;
        }
    }

    if themes.is_empty() {
        themes.push(ThemeFile {
            name: "default".into(),
            theme: ThemeConfig::default(),
        });
    }

    themes
}
