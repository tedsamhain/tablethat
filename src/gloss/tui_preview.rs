use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::path::{Path, PathBuf};
use tablethat_lib as lib;

struct App {
    files: Vec<PathBuf>,
    selected: usize,
    content: Vec<Line<'static>>,
    scroll: usize,
    offset: usize,
    width: usize,
    themes: Vec<lib::theme::ThemeFile>,
    current_theme: usize,
    quit: bool,
}

impl App {
    fn new(files: Vec<PathBuf>, themes: Vec<lib::theme::ThemeFile>) -> Self {
        let mut app = Self {
            files,
            selected: 0,
            content: Vec::new(),
            scroll: 0,
            offset: 0,
            width: 80,
            themes,
            current_theme: 0,
            quit: false,
        };
        app.load_selected();
        app
    }

    fn load_selected(&mut self) {
        if let Some(path) = self.files.get(self.selected)
            && let Ok(text) = std::fs::read_to_string(path)
        {
            let theme_cfg = self
                .themes
                .get(self.current_theme)
                .map(|t| &t.theme)
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<lib::ThemeConfig> =
                        std::sync::OnceLock::new();
                    DEFAULT.get_or_init(lib::ThemeConfig::default)
                });
            let th = lib::markdown::theme_from_cfg(theme_cfg);
            self.content = lib::markdown::render_markdown(&th, &text, self.width);
            self.scroll = 0;
            self.offset = 0;
        }
    }

    fn current_file(&self) -> Option<&Path> {
        self.files.get(self.selected).map(|p| p.as_path())
    }
}

/// Run single-file TUI viewer
pub fn run_file_viewer(
    path: &Path,
    cfg: &lib::Config,
    themes: &[lib::theme::ThemeFile],
    initial_theme: usize,
) {
    let files = vec![path.to_path_buf()];
    let mut app = App::new(files, themes.to_vec());
    app.current_theme = initial_theme;
    app.load_selected();
    run_tui(&mut app, cfg);
}

/// Run directory browser TUI
pub fn run_directory_browser(dir: &Path, cfg: &lib::Config, themes: &[lib::theme::ThemeFile]) {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort();

    if files.is_empty() {
        eprintln!("no .md files found in {}", dir.display());
        return;
    }

    let mut app = App::new(files, themes.to_vec());
    run_tui(&mut app, cfg);
}

fn run_tui(app: &mut App, cfg: &lib::Config) {
    let mut terminal = match ratatui::try_init() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("failed to init terminal: {e}");
            return;
        }
    };

    while !app.quit {
        if let Err(e) = terminal.draw(|frame| render(frame, app)) {
            if let Ok(t) = ratatui::try_init() {
                terminal = t;
            } else {
                eprintln!("render error: {e}");
                break;
            }
        }
        if let Err(e) = handle_events(app, cfg) {
            eprintln!("event error: {e}");
            break;
        }
    }

    ratatui::restore();
}

fn render(frame: &mut Frame, app: &App) {
    let [title_area, body_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    let file_name = app
        .current_file()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("(no file)");

    let theme_name = app
        .themes
        .get(app.current_theme)
        .map(|t| t.name.as_str())
        .unwrap_or("default");

    let title = Line::from(Span::styled(
        format!(
            " {} [{}] \u{2014} q:quit  c:theme  \u{2191}\u{2193}\u{2190}\u{2192}:pan",
            file_name, theme_name
        ),
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(title, title_area);

    let tw = ((body_area.width.saturating_sub(1) as f64 * 0.9) as u16).clamp(40, 120) as usize;
    if tw != app.width {
        // Width changed — would need to re-render, but we skip for simplicity in v1
    }

    frame.render_widget(
        Paragraph::new(app.content.clone()).scroll((app.scroll as u16, app.offset as u16)),
        Rect::new(
            body_area.x + 1,
            body_area.y,
            body_area.width.saturating_sub(1),
            body_area.height,
        ),
    );
}

fn handle_events(app: &mut App, _cfg: &lib::Config) -> Result<(), Box<dyn std::error::Error>> {
    if !event::poll(std::time::Duration::from_millis(100))? {
        return Ok(());
    }
    match event::read()? {
        Event::Resize(_, _) => {}
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.quit = true,
                KeyCode::Char('c') => {
                    if !app.themes.is_empty() {
                        app.current_theme = (app.current_theme + 1) % app.themes.len();
                        app.load_selected();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.scroll = app.scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.scroll = app.scroll.saturating_add(1);
                }
                KeyCode::PageUp => {
                    app.scroll = app.scroll.saturating_sub(20);
                }
                KeyCode::PageDown => {
                    app.scroll = app.scroll.saturating_add(20);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    app.offset = app.offset.saturating_sub(8);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app.offset = app.offset.saturating_add(8);
                }
                KeyCode::Enter if app.files.len() > 1 => {
                    app.load_selected();
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}
