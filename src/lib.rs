pub mod markdown;
pub mod theme;

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
    ///
    /// `app_name` is used for config file search paths (e.g. "plan" → plan.toml)
    /// `env_prefix` is used for environment variable filtering (e.g. "PLAN_")
    pub fn load(app_name: &str, env_prefix: &str, cli_config_path: Option<&Path>) -> Self {
        let mut figment = Figment::from(Serialized::defaults(Config::default()));

        // Layer 1: config file(s)
        if let Some(path) = cli_config_path {
            if path.exists() {
                figment = figment.merge(Toml::file(path));
            }
        } else {
            // Env var for config path
            let config_env_key = format!("{env_prefix}CONFIG");
            if let Ok(path) = std::env::var(&config_env_key) {
                let p = PathBuf::from(&path);
                if p.exists() {
                    figment = figment.merge(Toml::file(p));
                }
            }

            // Project-local
            let local = PathBuf::from(format!("{app_name}.toml"));
            if local.exists() {
                figment = figment.merge(Toml::file(local));
            }

            // Platform config dir
            if let Some(proj_dirs) = directories::ProjectDirs::from("", "", app_name) {
                let sys_config = proj_dirs.config_dir().join("config.toml");
                if sys_config.exists() {
                    figment = figment.merge(Toml::file(sys_config));
                }
            }
        }

        // Layer 2: environment variables
        figment = figment.merge(Env::prefixed(env_prefix));

        figment.extract().unwrap_or_default()
    }
}

/// Resolve a file by checking project-local, then config dir, then data dir.
/// Returns the first path that exists, or None.
pub fn resolve_file(
    root: &Path,
    plan_dir: &str,
    local_name: &str,
    global_name: &str,
    app_name: &str,
) -> Option<PathBuf> {
    // Project-local (dot-prefixed)
    let local = root.join(plan_dir).join(local_name);
    if local.exists() {
        return Some(local);
    }

    // Config dir (user)
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", app_name) {
        let config_file = proj_dirs.config_dir().join(global_name);
        if config_file.exists() {
            return Some(config_file);
        }
    }

    // Data dir (system)
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", app_name) {
        let data_file = proj_dirs.data_dir().join(global_name);
        if data_file.exists() {
            return Some(data_file);
        }
    }

    None
}

/// Parse a color name string into a termcolor Color.
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

pub fn workspace_root() -> PathBuf {
    let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut candidate = Some(dir.as_path());
    while let Some(path) = candidate {
        if path.join(".plan").is_dir() {
            return path.to_path_buf();
        }
        candidate = path.parent();
    }
    dir
}
