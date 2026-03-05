use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use crumbs::{commands, config, item::ItemType};

#[derive(Parser)]
#[command(name = "crumbs", about = "Flat-folder Markdown task tracker", version)]
struct Cli {
    /// Explicit directory to use
    #[arg(short, long, global = true)]
    dir: Option<PathBuf>,

    /// Use the global crumbs store (~/.local/share/crumbs)
    #[arg(short, long, global = true)]
    global: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a .crumbs store in the current directory
    Init {
        /// ID prefix to use (skips interactive prompt)
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Create a new item
    Create {
        title: String,
        #[arg(short = 't', long, default_value = "task")]
        item_type: String,
        #[arg(short, long, default_value = "2")]
        priority: u8,
        #[arg(long)]
        tags: Option<String>,
        #[arg(short = 'm', long)]
        message: Option<String>,
        /// Comma-separated dependency IDs
        #[arg(long)]
        depends: Option<String>,
    },
    /// Shorthand for create
    #[command(name = "c")]
    C {
        title: String,
        #[arg(short = 't', long, default_value = "task")]
        item_type: String,
        #[arg(short, long, default_value = "2")]
        priority: u8,
        #[arg(long)]
        tags: Option<String>,
        #[arg(short = 'm', long)]
        message: Option<String>,
        /// Comma-separated dependency IDs
        #[arg(long)]
        depends: Option<String>,
    },
    /// List items
    List {
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
        /// Show all items including closed
        #[arg(short, long)]
        all: bool,
    },
    /// Show a single item
    Show { id: String },
    /// Update an item
    Update {
        id: String,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        priority: Option<u8>,
        #[arg(long)]
        tags: Option<String>,
        #[arg(short = 't', long = "type")]
        item_type: Option<String>,
        /// Comma-separated dependency IDs (replaces existing)
        #[arg(long)]
        depends: Option<String>,
    },
    /// Close an item
    Close {
        id: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Permanently delete an item (or all closed items with --closed)
    Delete {
        /// ID of the item to delete
        id: Option<String>,
        /// Delete all closed items
        #[arg(long)]
        closed: bool,
    },
    /// Rebuild the CSV index from .md files
    Reindex,
    /// Search item content
    Search { query: String },
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Completions require no store.
    if let Command::Completions { shell } = cli.command {
        generate(shell, &mut Cli::command(), "crumbs", &mut std::io::stdout());
        return Ok(());
    }

    // For Init: --global initializes the global store; otherwise use cwd/.crumbs.
    if let Command::Init { prefix } = &cli.command {
        let target = if cli.global {
            config::global_dir()
        } else {
            std::env::current_dir()?.join(".crumbs")
        };
        return commands::init::run(&target, prefix.clone());
    }

    let dir = config::resolve_dir(cli.dir.clone(), cli.global);
    if !dir.is_dir() {
        let hint = if cli.dir.is_some() {
            format!("directory not found: {}", dir.display())
        } else if cli.global {
            format!(
                "global store not initialized — run: crumbs init --global\n  (expected: {})",
                dir.display()
            )
        } else {
            "no crumbs store found — run: crumbs init".to_string()
        };
        anyhow::bail!(hint);
    }

    match cli.command {
        Command::Init { .. } => unreachable!(),
        Command::Create {
            title,
            item_type,
            priority,
            tags,
            message,
            depends,
        }
        | Command::C {
            title,
            item_type,
            priority,
            tags,
            message,
            depends,
        } => {
            let item_type: ItemType = item_type.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let tags = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            let dependencies = depends
                .map(|d| d.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            commands::create::run(
                &dir,
                title,
                item_type,
                priority,
                tags,
                message.unwrap_or_default(),
                dependencies,
            )?;
        }
        Command::List { status, tag, all } => {
            commands::list::run(&dir, status.as_deref(), tag.as_deref(), all)?;
        }
        Command::Show { id } => {
            commands::show::run(&dir, &id)?;
        }
        Command::Update {
            id,
            status,
            priority,
            tags,
            item_type,
            depends,
        } => {
            let tags = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
            let dependencies =
                depends.map(|d| d.split(',').map(|s| s.trim().to_string()).collect());
            commands::update::run(&dir, &id, status, priority, tags, item_type, dependencies)?;
        }
        Command::Close { id, reason } => {
            commands::close::run(&dir, &id, reason)?;
        }
        Command::Delete { id, closed } => {
            if closed {
                commands::delete::run_closed(&dir)?;
            } else if let Some(id) = id {
                commands::delete::run(&dir, &id)?;
            } else {
                anyhow::bail!("provide an item ID or use --closed to delete all closed items");
            }
        }
        Command::Reindex => {
            commands::reindex::run(&dir)?;
        }
        Command::Search { query } => {
            commands::search::run(&dir, &query)?;
        }
        Command::Completions { .. } => unreachable!(),
    }

    Ok(())
}
