use crate::ThemeConfig;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

pub struct MarkdownTheme {
    pub h1: Style,
    pub h2: Style,
    pub h3: Style,
    pub bold: Style,
    pub dim: Style,
    pub code: Style,
}

pub fn theme_from_cfg(cfg: &ThemeConfig) -> MarkdownTheme {
    let bold_mod = match cfg.bold_style.as_str() {
        "bold" => Modifier::BOLD,
        "dim" => Modifier::DIM,
        "italic" => Modifier::ITALIC,
        "underlined" => Modifier::UNDERLINED,
        _ => Modifier::BOLD,
    };
    let emphasis_mod = match cfg.emphasis_style.as_str() {
        "bold" => Modifier::BOLD,
        "dim" => Modifier::DIM,
        "italic" => Modifier::ITALIC,
        "underlined" => Modifier::UNDERLINED,
        _ => Modifier::UNDERLINED,
    };

    MarkdownTheme {
        h1: Style::default()
            .fg(crate::parse_ratatui_color(&cfg.h1_color))
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        h2: Style::default()
            .fg(crate::parse_ratatui_color(&cfg.h2_color))
            .add_modifier(Modifier::UNDERLINED),
        h3: Style::default().fg(crate::parse_ratatui_color(&cfg.h3_color)),
        bold: Style::default().add_modifier(bold_mod),
        dim: Style::default().add_modifier(emphasis_mod),
        code: Style::default().fg(crate::parse_ratatui_color(&cfg.code_color)),
    }
}

pub fn strip_frontmatter(text: &str) -> &str {
    if let Some(rest) = text.trim_start().strip_prefix("---")
        && let Some(end) = rest.find("---")
    {
        return rest[end + 3..].trim_start();
    }
    text
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

pub fn render_markdown(th: &MarkdownTheme, text: &str, wrap: usize) -> Vec<Line<'static>> {
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

    let mut tbl: Vec<Vec<String>> = Vec::new();
    let mut cur_row: Vec<String> = Vec::new();
    let mut cur_cell = String::new();
    let mut in_cell = false;
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
                    let p80_total: usize = cols.iter().map(|c| c.p80).sum::<usize>() + border_w;
                    let base_target = 78usize;
                    let wide_target = 160usize;
                    let target = if p80_total <= wide_target {
                        base_target
                            .max(p80_total)
                            .min(wide_target)
                            .saturating_sub(border_w)
                    } else {
                        base_target.saturating_sub(border_w)
                    };

                    let mut col_w: Vec<usize> = cols.iter().map(|c| c.p60).collect();
                    let used: usize = col_w.iter().sum();

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

                    for &level in &[1, 2] {
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

                    let slack = target.saturating_sub(col_w.iter().sum());
                    if slack > 0
                        && let Some(max_i) = (0..ncols).max_by_key(|&i| col_w[i])
                    {
                        col_w[max_i] += slack;
                    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_renders_header_and_body() {
        let md = "| **Name** | `Code` |\n|---|---|\n| foo | bar |\n";
        let th = theme_from_cfg(&crate::ThemeConfig::default());
        let lines = render_markdown(&th, md, 80);
        let total: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(total.contains("Name"), "Missing header 'Name'");
        assert!(total.contains("foo"), "Missing body 'foo'");
        assert!(total.contains("="), "Missing separator");
    }
}
