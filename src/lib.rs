pub mod markdown;
pub mod theme;

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const PLAN_DIR: &str = ".plan";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub root: Option<PathBuf>,
    pub editor: Option<String>,
    pub themes_dir: Option<PathBuf>,
    pub default_view: String,
    pub default_sort: Vec<String>,
    pub kanban_order: Vec<String>,
    pub tui_width: usize,
    pub pager_width: usize,
    pub width: usize,
    pub colors: ColorsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub h1_color: Color,
    pub h2_color: Color,
    pub h3_color: Color,
    pub code_color: Color,
    #[serde(default = "default_code_block_color")]
    pub code_block_color: Color,
    pub bold_style: String,
    pub emphasis_style: String,
}

fn default_code_block_color() -> Color {
    Color::Yellow
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ColorsConfig {
    pub status: StatusColors,
    pub priority: PriorityColors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusColors {
    pub in_progress: Color,
    pub open: Color,
    pub blocked: Color,
    pub backlog: Color,
    pub idea: Color,
    pub done: Color,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriorityColors {
    pub high: Color,
    pub medium: Color,
    pub low: Color,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: None,
            editor: None,
            themes_dir: None,
            default_view: "list".into(),
            default_sort: vec!["priority".into(), "slug".into()],
            kanban_order: vec![
                "idea".into(),
                "backlog".into(),
                "open".into(),
                "in-progress".into(),
                "blocked".into(),
                "done".into(),
            ],
            tui_width: 80,
            pager_width: 120,
            width: 80,
            colors: ColorsConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            h1_color: Color::Cyan,
            h2_color: Color::Cyan,
            h3_color: Color::Cyan,
            code_color: Color::Yellow,
            code_block_color: Color::Yellow,
            bold_style: "bold".into(),
            emphasis_style: "underlined".into(),
        }
    }
}

impl Default for StatusColors {
    fn default() -> Self {
        Self {
            in_progress: Color::Magenta,
            open: Color::Yellow,
            blocked: Color::Red,
            backlog: Color::Blue,
            idea: Color::Cyan,
            done: Color::Green,
        }
    }
}

impl Default for PriorityColors {
    fn default() -> Self {
        Self {
            high: Color::Red,
            medium: Color::Yellow,
            low: Color::DarkGray,
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
        "gray" | "grey" => termcolor::Color::Ansi256(7),
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

/// Convert a ratatui Color to a termcolor Color.
pub fn ratatui_to_termcolor(c: Color) -> termcolor::Color {
    match c {
        Color::Black => termcolor::Color::Black,
        Color::Red => termcolor::Color::Red,
        Color::Green => termcolor::Color::Green,
        Color::Yellow => termcolor::Color::Yellow,
        Color::Blue => termcolor::Color::Blue,
        Color::Magenta => termcolor::Color::Magenta,
        Color::Cyan => termcolor::Color::Cyan,
        Color::Gray => termcolor::Color::Ansi256(7),
        Color::DarkGray => termcolor::Color::Ansi256(8),
        Color::White => termcolor::Color::White,
        Color::LightRed => termcolor::Color::Ansi256(9),
        Color::LightGreen => termcolor::Color::Ansi256(10),
        Color::LightYellow => termcolor::Color::Ansi256(11),
        Color::LightBlue => termcolor::Color::Ansi256(12),
        Color::LightMagenta => termcolor::Color::Ansi256(13),
        Color::LightCyan => termcolor::Color::Ansi256(14),
        Color::Indexed(n) => termcolor::Color::Ansi256(n),
        Color::Rgb(r, g, b) => termcolor::Color::Ansi256(rgb_to_ansi256(r, g, b)),
        _ => termcolor::Color::White,
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let ri = (r as u16 * 5 / 255) as u8;
    let gi = (g as u16 * 5 / 255) as u8;
    let bi = (b as u16 * 5 / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}

pub fn workspace_root() -> PathBuf {
    let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut candidate = Some(dir.as_path());
    while let Some(path) = candidate {
        if path.join(PLAN_DIR).is_dir() {
            return path.to_path_buf();
        }
        candidate = path.parent();
    }
    dir
}
