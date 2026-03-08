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
    #[arg(short, long, global = true, hide_short_help = true)]
    dir: Option<PathBuf>,

    /// Use the global crumbs store (~/.local/share/crumbs)
    #[arg(short, long, global = true, hide_short_help = true)]
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
        #[arg(short = 'm', long, allow_hyphen_values = true)]
        message: Option<String>,
        /// Comma-separated dependency IDs
        #[arg(long)]
        depends: Option<String>,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<NaiveDate>,
        /// Story points (Fibonacci: 1 2 3 5 8 13 21)
        #[arg(long)]
        points: Option<u8>,
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
        #[arg(short = 'm', long, allow_hyphen_values = true)]
        message: Option<String>,
        /// Comma-separated dependency IDs
        #[arg(long)]
        depends: Option<String>,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<NaiveDate>,
        /// Story points (Fibonacci: 1 2 3 5 8 13 21)
        #[arg(long)]
        points: Option<u8>,
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
        /// Show first two lines of body text beneath each item
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show one or more items
    Show {
        #[arg(num_args = 1..)]
        ids: Vec<String>,
    },
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
        #[arg(short = 'm', long, allow_hyphen_values = true)]
        message: Option<String>,
        /// Append text to the existing body with a [date] prefix
        #[arg(long, allow_hyphen_values = true)]
        append: Option<String>,
        /// Story points (Fibonacci: 1 2 3 5 8 13 21)
        #[arg(long)]
        points: Option<u8>,
        /// Remove the story points estimate
        #[arg(long)]
        clear_points: bool,
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
        /// Wake-up date: item resurfaces in `next` on or after this date (YYYY-MM-DD)
        #[arg(long)]
        until: Option<NaiveDate>,
    },
    /// Start a timer for an item (appends [start] entry, sets status to in_progress)
    Start {
        id: String,
        /// Optional comment to record with the start entry
        #[arg(short = 'm', long, allow_hyphen_values = true)]
        comment: Option<String>,
    },
    /// Stop the active timer for an item (appends [stop] entry with elapsed time)
    Stop {
        id: String,
        /// Optional comment to record with the stop entry
        #[arg(short = 'm', long, allow_hyphen_values = true)]
        comment: Option<String>,
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
    /// Permanently delete an item
    Delete {
        /// ID of the item to delete
        id: String,
    },
    /// Remove all closed items from the store
    Clean,
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
            points,
        }
        | Command::C {
            title,
            item_type,
            priority,
            tags,
            message,
            depends,
            due,
            points,
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
                points,
            )?;
        }
        Command::List {
            status,
            tag,
            priority,
            all,
            verbose,
        } => {
            commands::list::run(
                &dir,
                status.as_deref(),
                tag.as_deref(),
                priority,
                all,
                verbose,
            )?;
        }
        Command::Show { ids } => {
            commands::show::run(&dir, &ids)?;
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
            append,
            points,
            clear_points,
        } => {
            // --append 'text' sets append mode; --message 'text' sets replace mode.
            // If both are given, --append wins.
            let (final_message, final_append) = match (message, append) {
                (_, Some(a)) => (Some(a), true),
                (m, None) => (m, false),
            };
            commands::update::run(
                &dir,
                &id,
                commands::update::UpdateArgs {
                    status,
                    priority,
                    tags: tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
                    item_type,
                    dependencies: depends
                        .map(|d| d.split(',').map(|s| s.trim().to_string()).collect()),
                    due,
                    clear_due,
                    message: final_message,
                    append: final_append,
                    story_points: points,
                    clear_points,
                    title: None,
                },
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
        Command::Defer { id, reopen, until } => {
            commands::defer::run(&dir, &id, reopen, until)?;
        }
        Command::Start { id, comment } => {
            commands::start::run(&dir, &id, comment.as_deref())?;
        }
        Command::Stop { id, comment } => {
            commands::stop::run(&dir, &id, comment.as_deref())?;
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
        Command::Delete { id } => {
            commands::delete::run(&dir, &id)?;
        }
        Command::Clean => {
            commands::clean::run(&dir)?;
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
