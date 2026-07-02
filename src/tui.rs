use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const FIELD_COUNT: usize = 4; // type, priority, area, slug

struct MarkdownTheme {
    name: &'static str,
    h1: Style,
    h2: Style,
    h3: Style,
    bold: Style,
    dim: Style,
    code: Style,
}

fn themes() -> Vec<MarkdownTheme> {
    vec![
        MarkdownTheme {
            name: "default",
            h1: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            h2: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
            h3: Style::default().fg(Color::Cyan),
            bold: Style::default().add_modifier(Modifier::BOLD),
            dim: Style::default().add_modifier(Modifier::UNDERLINED),
            code: Style::default().fg(Color::Yellow),
        },
        MarkdownTheme {
            name: "classic",
            h1: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            h2: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
            h3: Style::default().fg(Color::Cyan),
            bold: Style::default().add_modifier(Modifier::BOLD),
            dim: Style::default().add_modifier(Modifier::UNDERLINED),
            code: Style::default().fg(Color::Yellow),
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    None,
    Quit,
    OpenEditor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Browse,
    Preview,
}

struct App<'a> {
    #[allow(dead_code)]
    root: &'a Path,
    entries: Vec<PathBuf>,
    tasks: Vec<crate::tasks::LoadedTask>,
    task_paths: HashMap<String, PathBuf>,
    columns: Vec<Column>,
    selected_column: usize,
    selected_task: usize,
    selected_field: usize,
    quit: bool,
    filter_status: Option<String>,
    filter_type: Option<String>,
    filter_priority: Option<String>,
    filter_area: Option<String>,
    filter_search: Option<String>,
    mode: Mode,
    preview: Vec<Line<'static>>,
    preview_scroll: usize,
    preview_offset: usize,
    preview_theme: usize,
    preview_width: usize,
}

struct Column {
    status: String,
    task_indices: Vec<usize>,
}

impl<'a> App<'a> {
    fn new(root: &'a Path, tasks: Vec<crate::tasks::LoadedTask>, entries: Vec<PathBuf>) -> Self {
        let path_map: HashMap<String, PathBuf> = entries
            .iter()
            .filter_map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|slug| (slug.to_string(), p.clone()))
            })
            .collect();

        let mut app = Self {
            root,
            entries,
            tasks,
            task_paths: path_map,
            columns: Vec::new(),
            selected_column: 0,
            selected_task: 0,
            selected_field: 0,
            quit: false,
            filter_status: None,
            filter_type: None,
            filter_priority: None,
            filter_area: None,
            filter_search: None,
            mode: Mode::Browse,
            preview: Vec::new(),
            preview_scroll: 0,
            preview_offset: 0,
            preview_theme: 0,
            preview_width: 80,
        };

        app.sort_tasks();
        app.rebuild_columns();
        app
    }

    fn sort_tasks(&mut self) {
        self.tasks.sort_by(|a, b| crate::tasks::cmp_tasks(a, b, &[]));
    }

    fn reload_tasks(&mut self) {
        self.tasks = crate::tasks::load_and_filter_tasks(
            &self.entries,
            self.filter_status.as_deref(),
            self.filter_type.as_deref(),
            self.filter_priority.as_deref(),
            self.filter_area.as_deref(),
            self.filter_search.as_deref(),
        );
        let path_map: HashMap<String, PathBuf> = self
            .entries
            .iter()
            .filter_map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|slug| (slug.to_string(), p.clone()))
            })
            .collect();
        self.task_paths = path_map;
        self.selected_column = 0;
        self.selected_task = 0;
        self.sort_tasks();
        self.rebuild_columns();
    }

    fn has_active_filters(&self) -> bool {
        self.filter_status.is_some()
            || self.filter_type.is_some()
            || self.filter_priority.is_some()
            || self.filter_area.is_some()
            || self.filter_search.is_some()
    }

    fn rebuild_columns(&mut self) {
        let kanban_order = crate::tasks::KANBAN_ORDER;
        let mut columns = Vec::new();
        for &status in kanban_order {
            let indices: Vec<usize> = self
                .tasks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.task.status == status)
                .map(|(i, _)| i)
                .collect();
            columns.push(Column {
                status: status.to_string(),
                task_indices: indices,
            });
        }
        self.columns = columns;

        if self.selected_column >= self.columns.len() {
            self.selected_column = self.columns.len().saturating_sub(1);
        }
        if let Some(col) = self.columns.get(self.selected_column)
            && self.selected_task >= col.task_indices.len()
        {
            self.selected_task = col.task_indices.len().saturating_sub(1);
        }
    }
}

pub fn run_tui(
    root: &Path,
    status_filter: Option<&str>,
    type_filter: Option<&str>,
    priority_filter: Option<&str>,
    area_filter: Option<&str>,
    search_query: Option<&str>,
) {
    let tasks_dir = root.join(".plan").join("tasks");
    let entries = crate::tasks::read_task_files(&tasks_dir).unwrap_or_default();
    let tasks = crate::tasks::load_and_filter_tasks(
        &entries,
        status_filter,
        type_filter,
        priority_filter,
        area_filter,
        search_query,
    );

    if tasks.is_empty() {
        eprintln!("(no tasks match filters)");
        return;
    }

    let mut app = App::new(root, tasks, entries);
    let mut terminal = match ratatui::try_init() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("failed to init terminal: {e}");
            return;
        }
    };

    while !app.quit {
        if let Err(e) = terminal.draw(|frame| app.render(frame)) {
            if let Ok(t) = ratatui::try_init() {
                terminal = t;
            } else {
                eprintln!("render error: {e}");
                break;
            }
        }
        match app.handle_events() {
            Ok(action) => match action {
                Action::None => {}
                Action::Quit => break,
                Action::OpenEditor => {
                    if let Some((_slug, path)) = app.current_task_path() {
                        ratatui::restore();
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                        let status = std::process::Command::new(&editor).arg(path).status();
                        if let Ok(s) = status
                            && !s.success()
                        {
                            eprintln!("{editor} exited with code: {:?}", s.code());
                        }
                        if let Ok(t) = ratatui::try_init() {
                            terminal = t;
                        } else {
                            eprintln!("failed to re-init terminal after editor");
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("event error: {e}");
                break;
            }
        }
    }

    ratatui::restore();
}

impl App<'_> {
    fn render(&mut self, frame: &mut Frame) {
        match self.mode {
            Mode::Browse => self.render_browse(frame),
            Mode::Preview => self.render_preview(frame),
        }
    }

    fn render_browse(&mut self, frame: &mut Frame) {
        let slug_w = self
            .tasks
            .iter()
            .map(|t| t.slug.len())
            .max()
            .unwrap_or(4)
            .clamp(4, 28);
        let type_w: usize = 8;
        let prio_w: usize = 8;
        let slot = |w: usize| w + 1;

        let [head_area, task_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        self.render_header(frame, head_area, slug_w, type_w, prio_w, slot);
        self.render_task_area(frame, task_area, slug_w, type_w, prio_w, slot);
        self.render_footer(frame, footer_area);
    }

    fn render_preview(&mut self, frame: &mut Frame) {
        let [title_area, body_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());
        let ts = themes();
        let tn = ts[self.preview_theme.min(ts.len() - 1)].name;
        let title = Line::from(Span::styled(
            format!(
                " Preview [{}] \u{2014} q/Esc:close  c:theme  \u{2191}\u{2193}\u{2190}\u{2192}:pan",
                tn
            ),
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(title, title_area);
        let tw = ((body_area.width.saturating_sub(1) as f64 * 0.9) as u16).clamp(40, 120) as usize;
        if tw != self.preview_width
            && self.mode == Mode::Preview
            && let Some((_slug, path)) = self.current_task_path()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            self.preview_width = tw;
            self.preview = render_markdown(
                &themes()[self.preview_theme.min(themes().len() - 1)],
                &content,
                tw,
            );
            self.preview_scroll = 0;
            self.preview_offset = 0;
        }
        frame.render_widget(
            Paragraph::new(self.preview.clone())
                .scroll((self.preview_scroll as u16, self.preview_offset as u16)),
            Rect::new(
                body_area.x + 1,
                body_area.y,
                body_area.width.saturating_sub(1),
                body_area.height,
            ),
        );
    }

    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        slug_w: usize,
        type_w: usize,
        prio_w: usize,
        slot: impl Fn(usize) -> usize,
    ) {
        // Header labels in same order as task rows: slug, type, priority, area
        // Each padded with same slot widths so columns align
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{:<w$}", format!(" {}", "Issue"), w = slot(slug_w)),
            if self.selected_field == 3 {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ));
        spans.push(Span::styled(
            format!("{:<w$}", format!(" {}", "Type"), w = slot(type_w)),
            if self.selected_field == 0 {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ));
        spans.push(Span::styled(
            format!("{:<w$}", format!(" {}", "Priority"), w = slot(prio_w)),
            if self.selected_field == 1 {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ));
        spans.push(Span::styled(
            format!(" {}", "Area"),
            if self.selected_field == 2 {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ));
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_task_area(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        slug_w: usize,
        type_w: usize,
        prio_w: usize,
        slot: impl Fn(usize) -> usize,
    ) {
        if self.columns.is_empty() || area.width < 10 {
            return;
        }
        // Color scheme — BOLD only on active field, non-active selected row has plain bg
        let sel_base = Style::default().bg(Color::White).fg(Color::Black);
        let sel_act = Style::default()
            .bg(Color::White)
            .fg(Color::Rgb(0, 0, 0))
            .add_modifier(Modifier::BOLD);
        let sel_prio_base = |pc: Color| Style::default().bg(pc).fg(Color::Black);
        let sel_prio_act = |pc: Color| {
            Style::default()
                .bg(pc)
                .fg(Color::Rgb(0, 0, 0))
                .add_modifier(Modifier::BOLD)
        };
        let is_vivid = |c: Color| {
            matches!(
                c,
                Color::Red
                    | Color::Yellow
                    | Color::Blue
                    | Color::Magenta
                    | Color::Cyan
                    | Color::Green
            )
        };

        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut selected_line = 0usize;

        for (col_idx, column) in self.columns.iter().enumerate() {
            let is_active = col_idx == self.selected_column;
            let color = status_color(&column.status);

            let header = format!(
                "{} ({})",
                column.status.to_uppercase(),
                column.task_indices.len()
            );
            let dash = area.width.saturating_sub(header.len() as u16 + 2);
            lines.push(Line::from(Span::styled(
                format!("{} {}", header, "─".repeat(dash.max(1) as usize)),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));

            if column.task_indices.is_empty() {
                lines.push(Line::from(Span::raw(" (none)")));
            } else {
                for (row_idx, &task_idx) in column.task_indices.iter().enumerate() {
                    let task = &self.tasks[task_idx];
                    let is_sel = is_active && row_idx == self.selected_task;
                    if is_sel {
                        selected_line = lines.len();
                    }
                    let is_act = |f: usize| is_sel && self.selected_field == f;

                    let mut spans = Vec::new();

                    // Slug (field 3)
                    let slen = slot(slug_w);
                    let slug = if is_act(3) {
                        let n = slen.saturating_sub(2);
                        let s = if task.slug.len() > n {
                            format!("{}…", &task.slug[..n.saturating_sub(1)])
                        } else {
                            task.slug.clone()
                        };
                        format!("{:<w$}", format!("[{}]", s), w = slen)
                    } else {
                        let n = slen.saturating_sub(1);
                        let s = if task.slug.len() > n {
                            format!("{}…", &task.slug[..n.saturating_sub(1)])
                        } else {
                            task.slug.clone()
                        };
                        format!("{:<w$}", format!(" {}", s), w = slen)
                    };
                    let slug_st = if is_sel && is_act(3) {
                        sel_act
                    } else if is_sel {
                        sel_base
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(slug, slug_st));

                    // Type (field 0)
                    let content = if is_act(0) {
                        format!(
                            "{:<w$}",
                            format!("[{}]", task.task.task_type),
                            w = slot(type_w)
                        )
                    } else {
                        format!(
                            "{:<w$}",
                            format!(" {}", task.task.task_type),
                            w = slot(type_w)
                        )
                    };
                    let type_st = if is_sel && is_act(0) {
                        sel_act
                    } else if is_sel {
                        sel_base
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(content, type_st));

                    // Priority (field 1) — vivid colors get colored bg, low uses plain row highlight
                    let pc = priority_color(&task.task.priority);
                    let vivid = is_vivid(pc);
                    let content = if is_act(1) {
                        format!(
                            "{:<w$}",
                            format!("[{}]", task.task.priority),
                            w = slot(prio_w)
                        )
                    } else {
                        format!(
                            "{:<w$}",
                            format!(" {}", task.task.priority),
                            w = slot(prio_w)
                        )
                    };
                    let prio_st = if is_sel && vivid && is_act(1) {
                        sel_prio_act(pc)
                    } else if is_sel && vivid {
                        sel_prio_base(pc)
                    } else if is_sel && is_act(1) {
                        sel_act
                    } else if is_sel {
                        sel_base
                    } else {
                        Style::default().fg(pc)
                    };
                    spans.push(Span::styled(content, prio_st));

                    // Area (field 2)
                    let content = if is_act(2) {
                        format!("[{}]", task.task.area)
                    } else {
                        format!(" {}", task.task.area)
                    };
                    let area_st = if is_sel && is_act(2) {
                        sel_act
                    } else if is_sel {
                        sel_base
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(content, area_st));

                    lines.push(Line::from(spans));
                }
            }

            lines.push(Line::from(""));
        }

        lines.pop();
        let visible_lines = area.height as usize;
        let vert_scroll = selected_line.saturating_sub(visible_lines.saturating_sub(2));
        frame.render_widget(Paragraph::new(lines).scroll((vert_scroll as u16, 0)), area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            " Enter:preview  f:filter  e:edit  {}  Ctrl-c:quit ",
            if self.has_active_filters() {
                "q:clear-filters"
            } else {
                "q:quit"
            }
        );

        let filters: Vec<(&str, &str)> = [
            ("status", self.filter_status.as_deref()),
            ("type", self.filter_type.as_deref()),
            ("priority", self.filter_priority.as_deref()),
            ("area", self.filter_area.as_deref()),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .collect();

        let mut spans = vec![Span::styled(text, Style::default().fg(Color::DarkGray))];
        if !filters.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled("Filter:", Style::default().fg(Color::Yellow)));
            for (k, v) in &filters {
                spans.push(Span::styled(
                    format!(" {}={}", k, v),
                    Style::default().fg(Color::Yellow),
                ));
            }
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}

impl App<'_> {
    fn handle_events(&mut self) -> Result<Action, Box<dyn std::error::Error>> {
        if !event::poll(std::time::Duration::from_millis(100))? {
            return Ok(Action::None);
        }
        match event::read()? {
            Event::Resize(_, _) => {}
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return Ok(Action::None);
                }
                match self.mode {
                    Mode::Preview => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Browse,
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.preview_scroll = self.preview_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.preview_scroll = self.preview_scroll.saturating_add(1);
                        }
                        KeyCode::PageUp => {
                            self.preview_scroll = self.preview_scroll.saturating_sub(20);
                        }
                        KeyCode::PageDown => {
                            self.preview_scroll = self.preview_scroll.saturating_add(20);
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            self.preview_offset = self.preview_offset.saturating_sub(8);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            self.preview_offset = self.preview_offset.saturating_add(8);
                        }
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            return Ok(Action::Quit);
                        }
                        KeyCode::Char('c') => self.cycle_theme(),
                        _ => {}
                    },
                    Mode::Browse => {
                        return Ok(match key.code {
                            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                                Action::Quit
                            }
                            KeyCode::Char('q') | KeyCode::Esc => {
                                if self.has_active_filters() {
                                    self.filter_status = None;
                                    self.filter_type = None;
                                    self.filter_priority = None;
                                    self.filter_area = None;
                                    self.filter_search = None;
                                    self.reload_tasks();
                                    Action::None
                                } else {
                                    Action::Quit
                                }
                            }
                            KeyCode::Enter if !self.columns.is_empty() => {
                                self.open_preview();
                                Action::None
                            }
                            KeyCode::Char('f') if !self.columns.is_empty() => {
                                let col = &self.columns[self.selected_column];
                                if let Some(&task_idx) = col.task_indices.get(self.selected_task) {
                                    let task = &self.tasks[task_idx];
                                    match self.selected_field {
                                        0 => self.filter_type = Some(task.task.task_type.clone()),
                                        1 => {
                                            self.filter_priority = Some(task.task.priority.clone())
                                        }
                                        2 => self.filter_area = Some(task.task.area.clone()),
                                        3 => self.filter_search = Some(task.slug.clone()),
                                        _ => {}
                                    }
                                    self.reload_tasks();
                                }
                                Action::None
                            }
                            KeyCode::Char('e') if !self.columns.is_empty() => Action::OpenEditor,
                            _ => {
                                self.handle_task_key(key.code);
                                Action::None
                            }
                        });
                    }
                }
            }
            _ => {}
        }
        Ok(Action::None)
    }

    fn handle_task_key(&mut self, code: KeyCode) {
        if self.columns.is_empty() {
            return;
        }
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_task > 0 {
                    self.selected_task -= 1;
                } else if self.selected_column > 0 {
                    self.selected_column -= 1;
                    self.selected_task = self.columns[self.selected_column]
                        .task_indices
                        .len()
                        .saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let col = &self.columns[self.selected_column];
                if self.selected_task + 1 < col.task_indices.len() {
                    self.selected_task += 1;
                } else if self.selected_column + 1 < self.columns.len() {
                    self.selected_column += 1;
                    self.selected_task = 0;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.selected_field = if self.selected_field > 0 {
                    self.selected_field - 1
                } else {
                    FIELD_COUNT - 1
                };
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected_field = if self.selected_field + 1 < FIELD_COUNT {
                    self.selected_field + 1
                } else {
                    0
                };
            }
            _ => {}
        }
    }

    fn open_preview(&mut self) {
        if let Some((_slug, path)) = self.current_task_path()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            self.preview_theme = 0;
            self.render_content(&content);
            self.preview_scroll = 0;
            self.preview_offset = 0;
            self.mode = Mode::Preview;
        }
    }

    fn cycle_theme(&mut self) {
        if let Some((_slug, path)) = self.current_task_path()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            self.preview_theme = (self.preview_theme + 1) % themes().len();
            self.render_content(&content);
        }
    }

    fn render_content(&mut self, content: &str) {
        let ts = themes();
        let theme = &ts[self.preview_theme.min(ts.len() - 1)];
        self.preview = render_markdown(theme, content, self.preview_width);
    }

    fn current_task_path(&self) -> Option<(&str, &Path)> {
        let col = self.columns.get(self.selected_column)?;
        let task_idx = col.task_indices.get(self.selected_task)?;
        let task = self.tasks.get(*task_idx)?;
        let path = self.task_paths.get(&task.slug)?;
        Some((&task.slug, path))
    }
}

fn wrap_lines(text: &str, max: usize) -> Vec<String> {
    if text.len() <= max {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in text.split_inclusive(' ') {
        if cur.len() + word.trim_end().len() > max && !cur.is_empty() {
            out.push(cur.trim_end().to_string());
            cur = word.trim_start().to_string();
        } else {
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        out.push(cur.trim_end().to_string());
    }
    out
}

fn wrap_spans(spans: &[Span<'static>], max: usize) -> Vec<Vec<Span<'static>>> {
    let mut out: Vec<Vec<Span<'static>>> = Vec::new();
    let mut cur: Vec<(String, Style)> = Vec::new();
    let mut line_len = 0usize;

    for span in spans {
        let text = span.content.as_ref();
        let style = span.style;
        let mut word = String::new();

        for ch in text.chars() {
            if ch == ' ' {
                if !word.is_empty() {
                    let wlen = word.len();
                    if line_len + wlen > max && !cur.is_empty() {
                        out.push(cur.drain(..).map(|(s, st)| Span::styled(s, st)).collect());
                        line_len = 0;
                    }
                    cur.push((std::mem::take(&mut word), style));
                    line_len += wlen;
                }
                if line_len + 1 > max && !cur.is_empty() {
                    out.push(cur.drain(..).map(|(s, st)| Span::styled(s, st)).collect());
                    line_len = 0;
                } else {
                    cur.push((" ".to_string(), style));
                    line_len += 1;
                }
            } else {
                word.push(ch);
            }
        }

        if !word.is_empty() {
            let wlen = word.len();
            if line_len + wlen > max && !cur.is_empty() {
                out.push(cur.drain(..).map(|(s, st)| Span::styled(s, st)).collect());
                line_len = 0;
            }
            cur.push((std::mem::take(&mut word), style));
            line_len += wlen;
        }
    }

    if !cur.is_empty() {
        out.push(cur.drain(..).map(|(s, st)| Span::styled(s, st)).collect());
    }

    out
}

fn strip_frontmatter(text: &str) -> &str {
    if let Some(rest) = text.trim_start().strip_prefix("---")
        && let Some(end) = rest.find("---")
    {
        return rest[end + 3..].trim_start();
    }
    text
}

fn render_markdown(th: &MarkdownTheme, text: &str, wrap: usize) -> Vec<Line<'static>> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};
    use pulldown_cmark::{HeadingLevel, Options};

    let body = strip_frontmatter(text);
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(body, opts);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = Vec::new();
    let mut in_code_block = false;

    // Table state
    let mut tbl: Vec<Vec<String>> = Vec::new();
    let mut cur_row: Vec<String> = Vec::new();
    let mut cur_cell = String::new();
    let mut in_cell = false;
    // List item state
    let mut item_text = String::new();
    let mut in_item = false;
    let mut item_checked: Option<bool> = None;

    let push_span = |spans: &mut Vec<Span<'static>>, text: &str, style: &Style| {
        if !text.is_empty() {
            spans.push(Span::styled(text.to_string(), *style));
        }
    };

    let flush = |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>| {
        if !spans.is_empty() {
            lines.push(Line::from(std::mem::take(spans)));
        }
    };

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush(&mut lines, &mut spans);
                    let prefix = match level {
                        HeadingLevel::H1 => "# ",
                        HeadingLevel::H2 => "## ",
                        _ => "### ",
                    };
                    let s = match level {
                        HeadingLevel::H1 => th.h1,
                        HeadingLevel::H2 => th.h2,
                        _ => th.h3,
                    };
                    push_span(&mut spans, prefix, &s);
                    style_stack.push(s);
                }
                Tag::Paragraph => flush(&mut lines, &mut spans),
                Tag::List(_) => {}
                Tag::Item => {
                    flush(&mut lines, &mut spans);
                    in_item = true;
                    item_text.clear();
                    item_checked = None;
                }
                Tag::CodeBlock(_) => {
                    flush(&mut lines, &mut spans);
                    in_code_block = true;
                }
                Tag::Strong => {
                    let s = style_stack
                        .last()
                        .map_or(th.bold, |base| base.add_modifier(Modifier::BOLD));
                    style_stack.push(s);
                }
                Tag::Emphasis => {
                    let s = style_stack
                        .last()
                        .map_or(th.dim, |base| base.add_modifier(Modifier::DIM));
                    style_stack.push(s);
                }
                Tag::Table(_) => {
                    flush(&mut lines, &mut spans);
                    tbl.clear();
                    cur_row.clear();
                }
                Tag::TableHead => {}
                Tag::TableRow => {
                    cur_row.clear();
                }
                Tag::TableCell => {
                    cur_cell.clear();
                    in_cell = true;
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    flush(&mut lines, &mut spans);
                    style_stack.clear();
                    lines.push(Line::from(""));
                }
                TagEnd::Paragraph => {
                    // Wrap paragraph spans at textwidth, preserving styles
                    if !spans.is_empty() {
                        let wrapped = wrap_spans(&spans, wrap);
                        for line_spans in wrapped {
                            lines.push(Line::from(line_spans));
                        }
                        spans.clear();
                    }
                    style_stack.clear();
                    lines.push(Line::from(""));
                }
                TagEnd::List(_) => {
                    lines.push(Line::from(""));
                }
                TagEnd::Item => {
                    in_item = false;
                    flush(&mut lines, &mut spans);
                    if !item_text.is_empty() {
                        let content = item_text.trim();
                        let (prefix, indent_sz) = match item_checked {
                            Some(true) => ("  - [x] ".to_string(), 8),
                            Some(false) => ("  - [ ] ".to_string(), 8),
                            None => ("  - ".to_string(), 4),
                        };
                        let first = format!("{}{}", prefix, content);
                        let indent = " ".repeat(indent_sz);
                        let lw = if wrap < 40 { 78 } else { wrap };
                        for (i, seg) in wrap_lines(&first, lw).iter().enumerate() {
                            let s: String = if i == 0 {
                                seg.into()
                            } else {
                                format!("{}{}", indent, seg)
                            };
                            lines.push(Line::from(s));
                        }
                    }
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    flush(&mut lines, &mut spans);
                    lines.push(Line::from(""));
                }
                TagEnd::Strong | TagEnd::Emphasis => {
                    style_stack.pop();
                }
                TagEnd::TableCell => {
                    in_cell = false;
                    cur_row.push(std::mem::take(&mut cur_cell));
                }
                TagEnd::TableHead => {
                    if !cur_row.is_empty() {
                        tbl.push(std::mem::take(&mut cur_row));
                    }
                }
                TagEnd::TableRow => {
                    tbl.push(std::mem::take(&mut cur_row));
                }
                TagEnd::Table => {
                    if tbl.is_empty() {
                        continue;
                    }
                    let ncols = tbl.iter().map(|r| r.len()).max().unwrap_or(0);
                    if ncols == 0 {
                        continue;
                    }

                    let trimmed: Vec<Vec<String>> = tbl
                        .iter()
                        .map(|row| row.iter().map(|c| c.trim().to_string()).collect())
                        .collect();

                    // Compute widths per column
                    struct ColW {
                        min: usize,
                        p60: usize,
                        p80: usize,
                        p100: usize,
                    }
                    let mut cols: Vec<ColW> = Vec::new();
                    for _ in 0..ncols {
                        cols.push(ColW {
                            min: 0,
                            p60: 0,
                            p80: 0,
                            p100: 0,
                        });
                    }
                    let mut all_lens: Vec<Vec<usize>> = vec![Vec::new(); ncols];

                    for row in &trimmed {
                        for (i, cell) in row.iter().enumerate() {
                            let lw = cell
                                .split_whitespace()
                                .map(|w| w.len())
                                .max()
                                .unwrap_or(cell.len());
                            cols[i].min = cols[i].min.max(lw.min(40));
                            all_lens[i].push(cell.len());
                        }
                    }

                    for i in 0..ncols {
                        let mut s = all_lens[i].clone();
                        s.sort_unstable();
                        let min = cols[i].min;
                        let n = s.len();
                        let idx60 = ((n as f64 * 0.6).ceil() as usize).saturating_sub(1);
                        let idx80 = ((n as f64 * 0.8).ceil() as usize).saturating_sub(1);
                        let last = n.saturating_sub(1);
                        let cap = |v: usize| v.max(min).min(40);
                        cols[i].p60 = cap(s.get(idx60).copied().unwrap_or(4));
                        cols[i].p80 = cap(s.get(idx80).copied().unwrap_or(4)).max(cols[i].p60);
                        cols[i].p100 = cap(s.get(last).copied().unwrap_or(4)).max(cols[i].p80);
                    }

                    let border_w = ncols * 3 + 1;
                    // If p80 total fits within 160 (borders included), use wider target
                    let p80_total: usize = cols.iter().map(|c| c.p80).sum::<usize>() + border_w;
                    let base_target = 78usize;
                    let wide_target = 160usize;
                    let target = if p80_total <= wide_target {
                        // Pick smallest target that fits p80
                        base_target
                            .max(p80_total)
                            .min(wide_target)
                            .saturating_sub(border_w)
                    } else {
                        base_target.saturating_sub(border_w)
                    };

                    // Start at p60, try to promote to p80 then p100
                    let mut col_w: Vec<usize> = cols.iter().map(|c| c.p60).collect();
                    let used: usize = col_w.iter().sum();

                    // If total at p60 already exceeds target, scale down proportionally
                    if used > target {
                        let deficit = used - target;
                        let flex: usize = col_w
                            .iter()
                            .zip(&cols)
                            .map(|(&w, c)| w.saturating_sub(c.min))
                            .sum();
                        col_w = col_w
                            .iter()
                            .enumerate()
                            .map(|(i, &w)| {
                                let room = w.saturating_sub(cols[i].min);
                                w.saturating_sub(if flex > 0 {
                                    deficit * room / flex.max(1)
                                } else {
                                    0
                                })
                                .max(cols[i].min)
                            })
                            .collect();
                    }

                    // Promote columns: first to p80 then p100, prioritizing large jumps
                    for &level in &[1, 2] {
                        // 1 = p80, 2 = p100
                        if col_w.iter().sum::<usize>() >= target {
                            break;
                        }
                        let mut order: Vec<usize> = (0..ncols).collect();
                        let gain = |i: usize| -> usize {
                            let target_w = if level == 1 {
                                cols[i].p80
                            } else {
                                cols[i].p100
                            };
                            target_w.saturating_sub(col_w[i])
                        };
                        order.sort_by_key(|&b| std::cmp::Reverse(gain(b)));
                        for &i in &order {
                            let target_w = if level == 1 {
                                cols[i].p80
                            } else {
                                cols[i].p100
                            };
                            let add = target_w.saturating_sub(col_w[i]);
                            if add == 0 {
                                continue;
                            }
                            let room = target - col_w.iter().sum::<usize>();
                            let take = add.min(room);
                            col_w[i] += take;
                            if col_w.iter().sum::<usize>() >= target {
                                break;
                            }
                        }
                    }

                    // Distribute remaining slack to largest column
                    let slack = target.saturating_sub(col_w.iter().sum());
                    if slack > 0
                        && let Some(max_i) = (0..ncols).max_by_key(|&i| col_w[i])
                    {
                        col_w[max_i] += slack;
                    }

                    // Render rows — first separator uses =, rest use -
                    let mut sep_count = 0usize;
                    for row in &trimmed {
                        if row.is_empty() {
                            continue;
                        }
                        if sep_count > 0 {
                            let ch = if sep_count == 1 { '=' } else { '-' };
                            let mut sep = String::from("|");
                            for &w in &col_w {
                                let dashes: String = std::iter::repeat_n(ch, w + 1).collect();
                                sep.push_str(&format!("{}|", dashes));
                            }
                            lines.push(Line::from(sep));
                        }
                        let mut buf = String::from("|");
                        for (i, &w) in col_w.iter().enumerate() {
                            let raw = row.get(i).map(|s| s.as_str()).unwrap_or("");
                            let cell: String = raw.chars().take(w).collect();
                            buf.push_str(&format!(" {:<w$}|", cell, w = w));
                        }
                        lines.push(Line::from(buf));
                        sep_count += 1;
                    }
                    lines.push(Line::from(""));
                }
                _ => {}
            },
            Event::Text(t) => {
                if in_item {
                    item_text.push_str(&t);
                } else if in_cell {
                    cur_cell.push_str(&t);
                } else if in_code_block {
                    for (i, l) in t.lines().enumerate() {
                        if i > 0 {
                            flush(&mut lines, &mut spans);
                        }
                        push_span(&mut spans, l, &Style::default());
                    }
                } else {
                    let style = style_stack.last().copied().unwrap_or_default();
                    push_span(&mut spans, &t, &style);
                }
            }
            Event::Code(t) => {
                if in_item {
                    item_text.push_str(&t);
                } else if in_cell {
                    cur_cell.push_str(&t);
                } else {
                    push_span(&mut spans, &t, &th.code);
                }
            }
            Event::TaskListMarker(checked) => {
                item_checked = Some(checked);
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_item {
                    item_text.push(' ');
                } else if in_cell {
                    cur_cell.push(' ');
                } else {
                    push_span(&mut spans, " ", &Style::default());
                }
            }
            Event::Html(t) => push_span(&mut spans, &t, &Style::default()),
            _ => {}
        }
    }
    flush(&mut lines, &mut spans);
    lines
}
fn status_color(status: &str) -> Color {
    match status {
        "in-progress" => Color::Magenta,
        "open" => Color::Yellow,
        "blocked" => Color::Red,
        "backlog" => Color::Blue,
        "idea" => Color::Cyan,
        "done" => Color::Green,
        _ => Color::White,
    }
}

fn priority_color(p: &str) -> Color {
    match p {
        "high" => Color::Red,
        "medium" => Color::Yellow,
        "low" => Color::DarkGray,
        _ => Color::White,
    }
}

#[cfg(test)]
mod quick_table_test {
    use crate::tui::{render_markdown, themes};
    #[test]
    fn table_renders_header_and_body() {
        let md = "| **Name** | `Code` |\n|---|---|\n| foo | bar |\n";
        let th = &themes()[0];
        let lines = render_markdown(th, md, 80);
        // Should have header + separator + body + blank → at least 3 lines
        let total: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(
            total.contains("Name"),
            "Missing header 'Name' in: {:?}",
            lines
        );
        assert!(total.contains("foo"), "Missing body 'foo' in: {:?}", lines);
        assert!(total.contains("="), "Missing separator in: {:?}", lines);
    }
}
