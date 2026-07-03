use std::io::{self, Read, Write};
use tablethat_lib::markdown;
use tablethat_lib::theme::ThemeFile;

/// Run in filter mode: read markdown from stdin, render styled output to stdout.
pub fn run_filter(theme: &ThemeFile, no_color: bool) {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read stdin");

    let md_theme = markdown::theme_from_cfg(&theme.theme);
    let lines = markdown::render_markdown(&md_theme, &input, 120);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let is_tty = atty_check();
    let _use_color = is_tty && !no_color;

    for line in &lines {
        for span in &line.spans {
            let _ = out.write_all(span.content.as_ref().as_bytes());
        }
        let _ = out.write_all(b"\n");
    }
}

fn atty_check() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}
