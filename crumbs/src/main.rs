use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::Result;
use chrono::NaiveDate;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use crumbs::{
    commands,
    commands::create::CreateArgs,
    commands::list::{ListArgs, SortKey},
    config,
    item::ItemType,
};

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
    // ── Browsing ──────────────────────────────────────────────────────────────
    /// List items
    List {
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
        /// Filter by priority (0-4)
        #[arg(short, long)]
        priority: Option<u8>,
        /// Filter by type (task, bug, feature, epic, idea)
        #[arg(long)]
        r#type: Option<String>,
        /// Filter by phase (e.g. "phase-1", "2026-Q2")
        #[arg(long)]
        phase: Option<String>,
        /// Show all items including closed
        #[arg(short, long)]
        all: bool,
        /// Show first two lines of body text beneath each item
        #[arg(short, long)]
        verbose: bool,
        /// Sort by field (id, priority, status, title, type, due, created, updated, phase)
        #[arg(long, default_value_t = SortKey::Id)]
        sort: SortKey,
    },
    /// Show one or more items
    Show {
        #[arg(num_args = 1..)]
        ids: Vec<String>,
    },
    /// Search item content
    Search { query: String },
    /// Show the highest-priority open item
    Next,
    /// Show summary statistics
    Stats,

    // ── Creating & editing ────────────────────────────────────────────────────
    /// Create a new item
    #[command(visible_alias = "c")]
    Create {
        title: String,
        #[arg(short = 't', long = "type", default_value = "task")]
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
        /// Phase or milestone label (e.g. "phase-1", "2026-Q2")
        #[arg(long)]
        phase: Option<String>,
    },
    /// Update an item
    Update {
        id: String,
        /// New title for the item
        #[arg(long)]
        title: Option<String>,
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
        /// Phase or milestone label (e.g. "phase-1", "2026-Q2")
        #[arg(long)]
        phase: Option<String>,
        /// Remove the phase label
        #[arg(long)]
        clear_phase: bool,
        /// PR or commit URL/reference that resolved this item (e.g. "owner/repo#42")
        #[arg(long)]
        resolution: Option<String>,
    },
    /// Edit an item's title and body in an inline TUI editor
    Body { id: String },
    /// Open an item in $EDITOR
    Edit { id: String },
    /// Append a note to an item's body (shorthand for `update --append`)
    #[command(visible_alias = "a")]
    Append {
        id: String,
        /// Text to append (prefixed with today's date)
        #[arg(allow_hyphen_values = true)]
        text: String,
    },

    // ── Lifecycle ─────────────────────────────────────────────────────────────
    /// Start a timer for an item (appends `[start]` entry, sets status to `in_progress`)
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
    /// Close an item
    Close {
        id: String,
        #[arg(short, long)]
        reason: Option<String>,
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
    /// Permanently delete an item
    Delete {
        /// ID of the item to delete
        id: String,
    },

    // ── Relationships ─────────────────────────────────────────────────────────
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

    // ── Store management ──────────────────────────────────────────────────────
    /// Initialize a .crumbs store in the current directory
    Init {
        /// ID prefix to use (skips interactive prompt)
        #[arg(long)]
        prefix: Option<String>,
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
    /// Remove all closed items from the store
    Clean,
    /// Rebuild the CSV index from .md files
    Reindex,
    /// Export items to CSV, JSON, TOON, or Markdown
    Export {
        /// Output format: csv, json, toon, or markdown
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Group markdown output by field: type, priority, phase, or status
        #[arg(long, value_name = "FIELD")]
        group_by: Option<String>,
        /// Write output to a file; omit value for `crumbs_export.<ext>` (default: stdout)
        #[arg(short, long, num_args = 0..=1, default_missing_value = "crumbs_export")]
        output: Option<PathBuf>,
    },

    // ── Tooling ───────────────────────────────────────────────────────────────
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',').map(|p| p.trim().to_string()).collect()
}

/// Dispatch commands whose CLI args require non-trivial parsing before calling into the library.
///
/// # Invariant
///
/// Only `Create`, `List`, and `Update` are valid inputs. The sole caller
/// (`run_command`) guards the call with an `@`-binding that restricts to
/// exactly those three variants, so the `_ => unreachable!()` arm below
/// can never fire at runtime. If a new "structured" command is added,
/// **both** the `@`-binding in `run_command` and a matching arm here must
/// be updated — the compiler will not warn if only one side changes.
#[allow(clippy::too_many_lines)] // one branch per CLI command; no natural split point
fn run_structured_commands(dir: &std::path::Path, command: Command) -> Result<()> {
    match command {
        Command::Create {
            title,
            item_type,
            priority,
            tags,
            message,
            depends,
            due,
            points,
            phase,
        } => {
            commands::create::run(
                dir,
                CreateArgs {
                    title,
                    item_type: item_type.parse().map_err(|e: String| anyhow::anyhow!(e))?,
                    priority,
                    tags: tags.map(|t| split_csv(&t)).unwrap_or_default(),
                    description: message.unwrap_or_default(),
                    dependencies: depends.map(|d| split_csv(&d)).unwrap_or_default(),
                    due,
                    story_points: points,
                    phase: phase.unwrap_or_default(),
                },
            )?;
        }
        Command::List {
            status,
            tag,
            priority,
            r#type,
            phase,
            all,
            verbose,
            sort,
        } => {
            let type_filter = r#type
                .as_deref()
                .map(|t| {
                    t.parse::<ItemType>()
                        .map_err(|e: String| anyhow::anyhow!(e))
                })
                .transpose()?;
            commands::list::run(
                dir,
                ListArgs {
                    status_filter: status,
                    tag_filter: tag,
                    priority_filter: priority,
                    type_filter,
                    phase_filter: phase,
                    all,
                    verbose,
                    sort: Some(sort),
                },
            )?;
        }
        Command::Update {
            id,
            title,
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
            phase,
            clear_phase,
            resolution,
        } => {
            // --append wins over --message when both are supplied.
            let (final_message, final_append) = match (message, append) {
                (_, Some(a)) => (Some(a), true),
                (m, None) => (m, false),
            };
            commands::update::run(
                dir,
                &id,
                commands::update::UpdateArgs {
                    status,
                    priority,
                    tags: tags.map(|t| split_csv(&t)),
                    item_type,
                    dependencies: depends.map(|d| split_csv(&d)),
                    due,
                    clear_due,
                    message: final_message,
                    append: final_append,
                    story_points: points,
                    clear_points,
                    title,
                    phase,
                    clear_phase,
                    resolution,
                },
            )?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn run_command(dir: &std::path::Path, command: Command) -> Result<()> {
    match command {
        Command::Init { .. } | Command::Completions { .. } => unreachable!(),
        cmd @ (Command::Create { .. } | Command::List { .. } | Command::Update { .. }) => {
            run_structured_commands(dir, cmd)?;
        }
        Command::Show { ids } => commands::show::run(dir, &ids)?,
        Command::Edit { id } => commands::edit::run(dir, &id)?,
        Command::Stats => commands::stats::run(dir)?,
        Command::Next => commands::next::run(dir)?,
        Command::Body { id } => commands::body::run(dir, &id)?,
        Command::Append { id, text } => commands::update::run_labeled(
            dir,
            &id,
            commands::update::UpdateArgs {
                message: Some(text),
                append: true,
                ..Default::default()
            },
            Some("Appended to"),
        )?,
        Command::Block {
            id,
            targets,
            remove,
        } => {
            if let Some(targets) = targets {
                commands::block::run(dir, &id, &split_csv(&targets), remove)?;
            } else {
                commands::block::run_set(dir, &id)?;
            }
        }
        Command::Defer { id, reopen, until } => commands::defer::run(dir, &id, reopen, until)?,
        Command::Start { id, comment } => commands::start::run(dir, &id, comment.as_deref())?,
        Command::Stop { id, comment } => commands::stop::run(dir, &id, comment.as_deref())?,
        Command::Move { id, to } => {
            let dst = if to == "global" {
                config::global_dir()
            } else {
                config::resolve_dir(Some(PathBuf::from(&to)), false)
            };
            commands::move_::run(dir, &id, &dst)?;
        }
        Command::Import { id, from } => {
            let src = if from == "global" {
                config::global_dir()
            } else {
                config::resolve_dir(Some(PathBuf::from(&from)), false)
            };
            commands::move_::run(&src, &id, dir)?;
        }
        Command::Link {
            id,
            relation,
            targets,
            remove,
        } => commands::link::run(dir, &id, &relation, &split_csv(&targets), remove)?,
        Command::Close { id, reason } => {
            // cr-by7: prompt interactively only in the CLI layer, so the
            // library function stays non-interactive (safe for GUI and tests).
            let reason = match reason {
                Some(r) => Some(r),
                None if std::io::stdin().is_terminal() => {
                    let r = dialoguer::Input::<String>::new()
                        .with_prompt("Close reason (optional, Enter to skip)")
                        .allow_empty(true)
                        .interact_text()?;
                    Some(r)
                }
                None => None,
            };
            commands::close::run(dir, &id, reason)?;
        }
        Command::Delete { id } => commands::delete::run(dir, &id)?,
        Command::Clean => commands::clean::run(dir)?,
        Command::Reindex => commands::reindex::run(dir)?,
        Command::Search { query } => commands::search::run(dir, &query)?,
        Command::Export {
            format,
            group_by,
            output,
        } => {
            let effective_format = group_by
                .as_deref()
                .map_or_else(|| format.clone(), |field| format!("markdown?group={field}"));
            let output = output.map(|p| {
                if p.as_os_str() == "crumbs_export" {
                    let ext = if format == "markdown" || group_by.is_some() {
                        "md"
                    } else {
                        &format
                    };
                    PathBuf::from(format!("crumbs_export.{ext}"))
                } else {
                    p
                }
            });
            commands::export::run(dir, &effective_format, output.as_deref())?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Completions require no store.
    if let Command::Completions { shell } = &cli.command {
        generate(
            *shell,
            &mut Cli::command(),
            "crumbs",
            &mut std::io::stdout(),
        );
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

    run_command(&dir, cli.command)
}
