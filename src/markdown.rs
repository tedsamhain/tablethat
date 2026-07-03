use crate::ThemeConfig;
use comrak::nodes::{NodeCodeBlock, NodeValue};
use comrak::{Arena, Options, parse_document};
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
    pub code_block: Style,
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
            .fg(cfg.h1_color)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        h2: Style::default()
            .fg(cfg.h2_color)
            .add_modifier(Modifier::UNDERLINED),
        h3: Style::default().fg(cfg.h3_color),
        bold: Style::default().add_modifier(bold_mod),
        dim: Style::default().add_modifier(emphasis_mod),
        code: Style::default().fg(cfg.code_color),
        code_block: Style::default().fg(cfg.code_block_color),
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

fn comrak_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.autolink = true;
    opts
}

fn collect_text(node: &comrak::Node<'_>) -> String {
    let mut buf = String::new();
    for child in node.children() {
        match &child.data.borrow().value {
            NodeValue::Text(t) => buf.push_str(t),
            NodeValue::Code(c) => buf.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => buf.push(' '),
            _ => buf.push_str(&collect_text(&child)),
        }
    }
    buf
}

fn render_inline(
    node: &comrak::Node<'_>,
    th: &MarkdownTheme,
    spans: &mut Vec<Span<'static>>,
    base_style: Style,
) {
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => {
                if !t.is_empty() {
                    spans.push(Span::styled(t.to_string(), base_style));
                }
            }
            NodeValue::Code(code) => {
                if !code.literal.is_empty() {
                    spans.push(Span::styled(code.literal.clone(), th.code));
                }
            }
            NodeValue::Strong => {
                let s = base_style.add_modifier(Modifier::BOLD);
                render_inline(&child, th, spans, s);
            }
            NodeValue::Emph => {
                let s = base_style.add_modifier(Modifier::DIM);
                render_inline(&child, th, spans, s);
            }
            NodeValue::Strikethrough => {
                render_inline(&child, th, spans, base_style);
            }
            NodeValue::Link(_link) => {
                let mut s = base_style;
                s = s.add_modifier(Modifier::UNDERLINED);
                render_inline(&child, th, spans, s);
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                spans.push(Span::styled(" ".to_string(), base_style));
            }
            NodeValue::HtmlInline(html) => {
                spans.push(Span::styled(html.clone(), base_style));
            }
            NodeValue::Image(link) => {
                let alt = collect_text(&child);
                let text = if alt.is_empty() {
                    link.url.clone()
                } else {
                    alt
                };
                spans.push(Span::styled(text, base_style));
            }
            _ => {
                render_inline(&child, th, spans, base_style);
            }
        }
    }
}

fn render_table(node: &comrak::Node<'_>, lines: &mut Vec<Line<'static>>) {
    let mut tbl: Vec<Vec<String>> = Vec::new();
    for row_node in node.children() {
        if let NodeValue::TableRow(_) = row_node.data.borrow().value {
            let mut cur_row: Vec<String> = Vec::new();
            for cell_node in row_node.children() {
                if let NodeValue::TableCell = cell_node.data.borrow().value {
                    cur_row.push(collect_text(&cell_node).trim().to_string());
                }
            }
            tbl.push(cur_row);
        }
    }

    if tbl.is_empty() {
        return;
    }
    let ncols = tbl.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return;
    }

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

    for row in &tbl {
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
    for row in &tbl {
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
}

fn render_node(
    node: &comrak::Node<'_>,
    th: &MarkdownTheme,
    lines: &mut Vec<Line<'static>>,
    wrap: usize,
) {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Document | NodeValue::FrontMatter(_) => {
            for child in node.children() {
                render_node(&child, th, lines, wrap);
            }
        }
        NodeValue::Heading(heading) => {
            let (prefix, style) = match heading.level {
                1 => ("# ", th.h1),
                2 => ("## ", th.h2),
                _ => ("### ", th.h3),
            };
            let mut spans = vec![Span::styled(prefix.to_string(), style)];
            render_inline(node, th, &mut spans, style);
            lines.push(Line::from(spans));
            lines.push(Line::from(""));
        }
        NodeValue::Paragraph => {
            let mut spans: Vec<Span<'static>> = Vec::new();
            render_inline(node, th, &mut spans, Style::default());
            if !spans.is_empty() {
                let wrapped = wrap_spans(&spans, wrap);
                for line_spans in wrapped {
                    lines.push(Line::from(line_spans));
                }
            }
            lines.push(Line::from(""));
        }
        NodeValue::BlockQuote => {
            for child in node.children() {
                render_node(&child, th, lines, wrap);
            }
        }
        NodeValue::List(_list) => {
            for child in node.children() {
                render_node(&child, th, lines, wrap);
            }
            lines.push(Line::from(""));
        }
        NodeValue::Item(_list) => {
            let mut item_text = String::new();

            for child in node.children() {
                item_text.push_str(&collect_text(&child));
            }

            let content = item_text.trim();
            if !content.is_empty() {
                let prefix = "  - ";
                let first = format!("{}{}", prefix, content);
                let lw = if wrap < 40 { 78 } else { wrap };
                for (i, seg) in wrap_lines(&first, lw).iter().enumerate() {
                    let s: String = if i == 0 {
                        seg.into()
                    } else {
                        format!("    {}", seg)
                    };
                    lines.push(Line::from(s));
                }
            }
        }
        NodeValue::TaskItem(task) => {
            let checked = task.symbol == Some('x') || task.symbol == Some('X');
            let mut item_text = String::new();

            for child in node.children() {
                item_text.push_str(&collect_text(&child));
            }

            let content = item_text.trim();
            let check_mark = if checked { "[x]" } else { "[ ]" };
            let prefix = format!("  - {} ", check_mark);
            let indent_sz = prefix.len();
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
        NodeValue::CodeBlock(code_block) => {
            let NodeCodeBlock { literal, .. } = code_block.as_ref();
            for line in literal.lines() {
                lines.push(Line::from(Span::styled(line.to_string(), th.code_block)));
            }
            lines.push(Line::from(""));
        }
        NodeValue::Table(_) => {
            render_table(node, lines);
            lines.push(Line::from(""));
        }
        NodeValue::ThematicBreak => {
            lines.push(Line::from("---".to_string()));
            lines.push(Line::from(""));
        }
        NodeValue::HtmlBlock(html_block) => {
            for line in html_block.literal.lines() {
                lines.push(Line::from(line.to_string()));
            }
        }
        _ => {
            for child in node.children() {
                render_node(&child, th, lines, wrap);
            }
        }
    }
}

pub fn render_markdown(th: &MarkdownTheme, text: &str, wrap: usize) -> Vec<Line<'static>> {
    let body = strip_frontmatter(text);
    let arena = Arena::new();
    let opts = comrak_options();
    let root = parse_document(&arena, body, &opts);

    let mut lines: Vec<Line<'static>> = Vec::new();
    render_node(&root, th, &mut lines, wrap);
    lines
}

pub fn format_commonmark(text: &str, width: usize) -> String {
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.autolink = true;
    opts.render.width = width;
    opts.render.prefer_fenced = true;
    let formatted = comrak::markdown_to_commonmark(text, &opts);
    let aligned = align_tables(&formatted);
    strip_trailing_whitespace(&aligned)
}

fn strip_trailing_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

fn align_tables(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Detect table: line starts with | and next line is separator (| --- |)
        if lines[i].starts_with('|')
            && i + 1 < lines.len()
            && lines[i + 1].starts_with('|')
            && lines[i + 1].contains("---")
        {
            // Collect all table rows
            let mut table_lines = Vec::new();
            while i < lines.len() && lines[i].starts_with('|') {
                table_lines.push(lines[i]);
                i += 1;
            }

            // Parse cells and compute column widths
            let parsed: Vec<Vec<&str>> = table_lines
                .iter()
                .map(|line| {
                    line.trim_start_matches('|')
                        .trim_end_matches('|')
                        .split('|')
                        .map(|s| s.trim())
                        .collect()
                })
                .collect();

            let num_cols = parsed.iter().map(|r| r.len()).max().unwrap_or(0);
            let mut col_widths = vec![0usize; num_cols];
            for row in &parsed {
                for (j, cell) in row.iter().enumerate() {
                    col_widths[j] = col_widths[j].max(cell.len());
                }
            }

            // Reformat table with aligned columns
            for (row_idx, row) in parsed.iter().enumerate() {
                let mut line = String::from("|");
                for (j, cell) in row.iter().enumerate() {
                    let w = col_widths.get(j).copied().unwrap_or(3);
                    if row_idx == 1 {
                        // Separator row: use alignment markers
                        let marker = if cell.starts_with(':') && cell.ends_with(':') {
                            format!(" :{:-<w$}: ", "", w = w.saturating_sub(2))
                        } else if cell.starts_with(':') {
                            format!(" :{:-<w$} ", "", w = w.saturating_sub(1))
                        } else if cell.ends_with(':') {
                            format!(" {:-<w$}: ", "", w = w.saturating_sub(1))
                        } else {
                            format!(" {:-<w$} ", "", w = w)
                        };
                        line.push_str(&marker);
                    } else {
                        line.push_str(&format!(" {:<w$} ", cell, w = w));
                    }
                    line.push('|');
                }
                result.push(line);
            }
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    result.join("\n")
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
