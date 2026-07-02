use clap::Parser;
use std::path::PathBuf;
use tablethat_lib as lib;

#[path = "../../plan/tasks.rs"]
mod tasks;
#[path = "../../plan/tui_kanban.rs"]
mod tui_kanban;

#[derive(Parser)]
#[command(
    name = "plan",
    version,
    max_term_width = 80,
    about = "Task management with kanban TUI — plan your work, table that thought"
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

    /// Initialize a .plan/ directory with default schema and template
    #[arg(long)]
    init: bool,
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

    let mut cfg = lib::Config::load("plan", "PLAN_", cli.config.as_deref());

    if cli.root.is_some() {
        cfg.root = cli.root;
    }

    let root = cfg.root.clone().unwrap_or_else(lib::workspace_root);

    // --init: scaffold .plan/ directory
    if cli.init {
        tasks::init_plan(&root);
        return;
    }

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
            tui_kanban::run_tui(
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
