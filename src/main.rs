use std::path::PathBuf;

use anyhow::Result;
use chrono::NaiveDate;
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
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<NaiveDate>,
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
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<NaiveDate>,
    },
    /// List items
    List {
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
        /// Filter by priority (0-4)
        #[arg(short, long)]
        priority: Option<u8>,
        /// Show all items including closed
        #[arg(short, long)]
        all: bool,
    },
    /// Show a single item
    Show { id: String },
    /// Open an item in $EDITOR
    Edit { id: String },
    /// Show summary statistics
    Stats,
    /// Show the highest-priority open item
    Next,
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
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<NaiveDate>,
        /// Remove the due date
        #[arg(long)]
        clear_due: bool,
        /// Replace the item description
        #[arg(short = 'm', long)]
        message: Option<String>,
    },
    /// Mark an item as blocking others (links + sets blocked status on targets)
    Block {
        /// The item that is doing the blocking
        id: String,
        /// Comma-separated IDs of items being blocked (omit to mark <id> itself as blocked)
        targets: Option<String>,
        /// Remove the blocking relationship (and reopen targets if unblocked)
        #[arg(long)]
        remove: bool,
    },
    /// Defer an item (or reopen a deferred item)
    Defer {
        id: String,
        /// Reopen a deferred item (set status back to open)
        #[arg(long)]
        reopen: bool,
    },
    /// Move an item to a different store (assigns a new ID)
    Move {
        /// ID of the item to move
        id: String,
        /// Destination store path, or "global" for the global store
        #[arg(long)]
        to: String,
    },
    /// Import an item from another store into the current store
    Import {
        /// Full ID of the item to import (e.g. glob-x7q)
        id: String,
        /// Source store path, or "global" for the global store
        #[arg(long)]
        from: String,
    },
    /// Add or remove blocks/blocked-by relationships between items
    Link {
        id: String,
        /// Relationship direction: "blocks" or "blocked-by"
        relation: String,
        /// Comma-separated target IDs
        targets: String,
        /// Remove the link instead of adding it
        #[arg(long)]
        remove: bool,
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
    /// Export items to CSV, JSON, or TOON
    Export {
        /// Output format: csv, json, or toon
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Write output to a file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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
            due,
        }
        | Command::C {
            title,
            item_type,
            priority,
            tags,
            message,
            depends,
            due,
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
                due,
            )?;
        }
        Command::List {
            status,
            tag,
            priority,
            all,
        } => {
            commands::list::run(&dir, status.as_deref(), tag.as_deref(), priority, all)?;
        }
        Command::Show { id } => {
            commands::show::run(&dir, &id)?;
        }
        Command::Edit { id } => {
            commands::edit::run(&dir, &id)?;
        }
        Command::Stats => {
            commands::stats::run(&dir)?;
        }
        Command::Next => {
            commands::next::run(&dir)?;
        }
        Command::Update {
            id,
            status,
            priority,
            tags,
            item_type,
            depends,
            due,
            clear_due,
            message,
        } => {
            let tags = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
            let dependencies =
                depends.map(|d| d.split(',').map(|s| s.trim().to_string()).collect());
            commands::update::run(
                &dir,
                &id,
                status,
                priority,
                tags,
                item_type,
                dependencies,
                due,
                clear_due,
                message,
            )?;
        }
        Command::Block {
            id,
            targets,
            remove,
        } => {
            if let Some(targets) = targets {
                let target_ids: Vec<String> =
                    targets.split(',').map(|s| s.trim().to_string()).collect();
                commands::block::run(&dir, &id, &target_ids, remove)?;
            } else {
                commands::block::run_set(&dir, &id)?;
            }
        }
        Command::Defer { id, reopen } => {
            commands::defer::run(&dir, &id, reopen)?;
        }
        Command::Move { id, to } => {
            let dst = if to == "global" {
                config::global_dir()
            } else {
                PathBuf::from(&to)
            };
            commands::move_::run(&dir, &id, &dst)?;
        }
        Command::Import { id, from } => {
            let src = if from == "global" {
                config::global_dir()
            } else {
                PathBuf::from(&from)
            };
            commands::move_::run(&dir, &id, &src)?;
        }
        Command::Link {
            id,
            relation,
            targets,
            remove,
        } => {
            let target_ids: Vec<String> =
                targets.split(',').map(|s| s.trim().to_string()).collect();
            commands::link::run(&dir, &id, &relation, &target_ids, remove)?;
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
        Command::Export { format, output } => {
            commands::export::run(&dir, &format, output.as_deref())?;
        }
        Command::Completions { .. } => unreachable!(),
    }

    Ok(())
}
