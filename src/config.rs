use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub root: Option<PathBuf>,
    pub editor: Option<String>,
    pub themes_dir: Option<PathBuf>,
    pub default_sort: Vec<String>,
    pub kanban_order: Vec<String>,
    pub theme: ThemeConfig,
    pub colors: ColorsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub h1_color: String,
    pub h2_color: String,
    pub h3_color: String,
    pub code_color: String,
    pub bold_style: String,
    pub emphasis_style: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ColorsConfig {
    pub status: StatusColors,
    pub priority: PriorityColors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusColors {
    pub in_progress: String,
    pub open: String,
    pub blocked: String,
    pub backlog: String,
    pub idea: String,
    pub done: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriorityColors {
    pub high: String,
    pub medium: String,
    pub low: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: None,
            editor: None,
            themes_dir: None,
            default_sort: vec!["priority".into(), "slug".into()],
            kanban_order: vec![
                "idea".into(),
                "backlog".into(),
                "open".into(),
                "in-progress".into(),
                "blocked".into(),
                "done".into(),
            ],
            theme: ThemeConfig::default(),
            colors: ColorsConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            h1_color: "cyan".into(),
            h2_color: "cyan".into(),
            h3_color: "cyan".into(),
            code_color: "yellow".into(),
            bold_style: "bold".into(),
            emphasis_style: "underlined".into(),
        }
    }
}

impl Default for StatusColors {
    fn default() -> Self {
        Self {
            in_progress: "magenta".into(),
            open: "yellow".into(),
            blocked: "red".into(),
            backlog: "blue".into(),
            idea: "cyan".into(),
            done: "green".into(),
        }
    }
}

impl Default for PriorityColors {
    fn default() -> Self {
        Self {
            high: "red".into(),
            medium: "yellow".into(),
            low: "darkgray".into(),
        }
    }
}

impl Config {
    /// Load configuration with layered precedence:
    /// defaults < config file < env vars < CLI overrides
    pub fn load(cli_config_path: Option<&Path>) -> Self {
        let mut figment = Figment::from(Serialized::defaults(Config::default()));

        // Layer 1: config file(s)
        // Explicit --config flag
        if let Some(path) = cli_config_path {
            if path.exists() {
                figment = figment.merge(Toml::file(path));
            }
        } else {
            // T2_CONFIG env var
            if let Ok(path) = std::env::var("T2_CONFIG") {
                let p = PathBuf::from(&path);
                if p.exists() {
                    figment = figment.merge(Toml::file(p));
                }
            }

            // Project-local
            let local = PathBuf::from("tablethat.toml");
            if local.exists() {
                figment = figment.merge(Toml::file(local));
            }

            // Platform config dir
            if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "tablethat") {
                let sys_config = proj_dirs.config_dir().join("config.toml");
                if sys_config.exists() {
                    figment = figment.merge(Toml::file(sys_config));
                }
            }
        }

        // Layer 2: environment variables (T2_ prefix)
        figment = figment.merge(Env::prefixed("T2_"));

        figment.extract().unwrap_or_default()
    }
}

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
/// 3. `~/.config/tablethat/themes/`
pub fn load_themes(themes_dir: Option<&Path>) -> Vec<ThemeFile> {
    let dirs: Vec<PathBuf> = if let Some(dir) = themes_dir {
        vec![dir.to_path_buf()]
    } else {
        let mut candidates = vec![PathBuf::from("themes")];
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "tablethat") {
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
            break; // use first directory that has themes
        }
    }

    // Fallback: always have at least the built-in default
    if themes.is_empty() {
        themes.push(ThemeFile {
            name: "default".into(),
            theme: ThemeConfig::default(),
        });
    }

    themes
}

/// Parse a color name string into a termcolor Color.
/// Supports named colors and basic ANSI256 decimal values.
pub fn parse_color(s: &str) -> termcolor::Color {
    match s.to_lowercase().as_str() {
        "black" => termcolor::Color::Black,
        "red" => termcolor::Color::Red,
        "green" => termcolor::Color::Green,
        "yellow" => termcolor::Color::Yellow,
        "blue" => termcolor::Color::Blue,
        "magenta" => termcolor::Color::Magenta,
        "cyan" => termcolor::Color::Cyan,
        "gray" | "grey" => termcolor::Color::White,
        "darkgray" | "darkgrey" => termcolor::Color::Ansi256(8),
        "white" => termcolor::Color::White,
        _ => {
            if let Ok(n) = s.parse::<u8>() {
                termcolor::Color::Ansi256(n)
            } else {
                termcolor::Color::White
            }
        }
    }
}

/// Parse a color name string into a ratatui Color.
pub fn parse_ratatui_color(s: &str) -> ratatui::style::Color {
    match s.to_lowercase().as_str() {
        "black" => ratatui::style::Color::Black,
        "red" => ratatui::style::Color::Red,
        "green" => ratatui::style::Color::Green,
        "yellow" => ratatui::style::Color::Yellow,
        "blue" => ratatui::style::Color::Blue,
        "magenta" => ratatui::style::Color::Magenta,
        "cyan" => ratatui::style::Color::Cyan,
        "gray" | "grey" => ratatui::style::Color::Gray,
        "darkgray" | "darkgrey" => ratatui::style::Color::DarkGray,
        "white" => ratatui::style::Color::White,
        _ => {
            if let Ok(n) = s.parse::<u8>() {
                ratatui::style::Color::Indexed(n)
            } else {
                ratatui::style::Color::White
            }
        }
    }
}
