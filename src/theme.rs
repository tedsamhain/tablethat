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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_themes_from_dir() {
        let themes = load_themes(Some(Path::new("themes")), "plan");
        assert!(
            !themes.is_empty(),
            "should find themes in themes/ directory"
        );
        // Should find at least the built-in themes
        assert!(
            themes.len() >= 10,
            "expected at least 10 themes, got {}",
            themes.len()
        );
    }

    #[test]
    fn load_themes_includes_default() {
        let themes = load_themes(Some(Path::new("themes")), "plan");
        let has_default = themes.iter().any(|t| t.name == "default");
        assert!(has_default, "should include default theme");
    }

    #[test]
    fn load_themes_fallback_to_default() {
        let themes = load_themes(Some(Path::new("/nonexistent/path")), "plan");
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "default");
    }

    #[test]
    fn theme_file_has_required_fields() {
        let themes = load_themes(Some(Path::new("themes")), "plan");
        for theme in &themes {
            assert!(!theme.name.is_empty(), "theme name should not be empty");
            // All themes should have valid colors (they deserialize via ratatui Color)
            let _ = theme.theme.h1_color;
            let _ = theme.theme.h2_color;
            let _ = theme.theme.h3_color;
            let _ = theme.theme.code_color;
            let _ = theme.theme.code_block_color;
        }
    }
}
