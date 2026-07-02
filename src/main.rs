mod config;
mod tasks;
mod tui;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tablethat",
    version,
    max_term_width = 80,
    about = "Markdown-native task tracker with kanban TUI — table that thought and come back to it later"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to config file (default: auto-detect)
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Repo root containing .plan (default: auto-detect)
    #[arg(short, long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

    /// Filter by status (open, in-progress, blocked, backlog, idea, done)
    #[arg(short, long, global = true, value_name = "STATUS")]
    status: Option<String>,

    /// Filter by type (bug, feature, chore, decision, perf)
    #[arg(short = 't', long = "type", global = true, value_name = "TYPE")]
    type_: Option<String>,

    /// Filter by priority (high, medium, low)
    #[arg(short, long, global = true, value_name = "PRIORITY")]
    priority: Option<String>,

    /// Filter by area label
    #[arg(short, long, global = true, value_name = "AREA")]
    area: Option<String>,

    /// Search slug and body text
    #[arg(short = 'q', long, global = true, value_name = "QUERY")]
    search: Option<String>,

    /// Group by status (vertical sections)
    #[arg(short, long)]
    kanban: bool,

    /// Sort key(s), repeatable for compound sort (status, type, priority, area, slug)
    #[arg(short = 'S', long = "sort", value_name = "FIELD")]
    sort: Vec<String>,

    /// Validate frontmatter only, then exit
    #[arg(long)]
    lint: bool,

    /// Normalize frontmatter + body, then exit
    #[arg(long)]
    format: bool,
}

#[derive(Parser)]
enum Commands {
    /// Interactive kanban browser (arrow keys, Enter to filter, e to edit)
    Tui {
        /// Filter by status
        #[arg(short, long, global = true, value_name = "STATUS")]
        status: Option<String>,

        /// Filter by type
        #[arg(short = 't', long = "type", global = true, value_name = "TYPE")]
        type_: Option<String>,

        /// Filter by priority
        #[arg(short, long, global = true, value_name = "PRIORITY")]
        priority: Option<String>,

        /// Filter by area
        #[arg(short, long, global = true, value_name = "AREA")]
        area: Option<String>,

        /// Search slug and body text
        #[arg(short = 'q', long, global = true, value_name = "QUERY")]
        search: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Load layered config: defaults < file < env < CLI
    let mut cfg = config::Config::load(cli.config.as_deref());

    // CLI --root overrides config root
    if cli.root.is_some() {
        cfg.root = cli.root;
    }

    // Resolve root: config/CLI > auto-detect
    let root = cfg.root.clone().unwrap_or_else(tasks::workspace_root);

    // Resolve filter values — prefer subcommand flags, fall back to top-level
    let (status_filter, type_filter, priority_filter, area_filter, search_query) =
        match &cli.command {
            Some(Commands::Tui {
                status,
                type_,
                priority,
                area,
                search,
            }) => (
                status.as_deref(),
                type_.as_deref(),
                priority.as_deref(),
                area.as_deref(),
                search.as_deref(),
            ),
            None => (
                cli.status.as_deref(),
                cli.type_.as_deref(),
                cli.priority.as_deref(),
                cli.area.as_deref(),
                cli.search.as_deref(),
            ),
        };

    match cli.command {
        Some(Commands::Tui { .. }) => {
            tui::run_tui(
                &root,
                &cfg,
                status_filter,
                type_filter,
                priority_filter,
                area_filter,
                search_query,
            );
        }
        None => {
            if cli.lint {
                let ok = tasks::validate_all(&root);
                std::process::exit(if ok { 0 } else { 1 });
            }

            if cli.format {
                tasks::normalize_all(&root);
                return;
            }

            tasks::list_tasks(
                &root,
                &cfg,
                status_filter,
                type_filter,
                priority_filter,
                area_filter,
                search_query,
                cli.kanban,
                &cli.sort,
            );
        }
    }
}
