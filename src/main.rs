use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
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
    Init,
    /// Create a new item
    Create {
        title: String,
        #[arg(short = 't', long, default_value = "task")]
        item_type: String,
        #[arg(short, long, default_value = "2")]
        priority: u8,
        #[arg(long)]
        tags: Option<String>,
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
    },
    /// List items
    List {
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
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
    },
    /// Close an item
    Close {
        id: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Rebuild the CSV index from .md files
    Reindex,
    /// Search item content
    Search { query: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // For Init: --global initializes the global store; otherwise use cwd/.crumbs.
    if matches!(&cli.command, Command::Init) {
        let target = if cli.global {
            config::global_dir()
        } else {
            std::env::current_dir()?.join(".crumbs")
        };
        return commands::init::run(&target);
    }

    let dir = config::resolve_dir(cli.dir, cli.global);

    match cli.command {
        Command::Init => unreachable!(),
        Command::Create {
            title,
            item_type,
            priority,
            tags,
        }
        | Command::C {
            title,
            item_type,
            priority,
            tags,
        } => {
            let item_type: ItemType = item_type.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let tags = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            commands::create::run(&dir, title, item_type, priority, tags)?;
        }
        Command::List { status, tag } => {
            commands::list::run(&dir, status.as_deref(), tag.as_deref())?;
        }
        Command::Show { id } => {
            commands::show::run(&dir, &id)?;
        }
        Command::Update {
            id,
            status,
            priority,
            tags,
        } => {
            let tags = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
            commands::update::run(&dir, &id, status, priority, tags)?;
        }
        Command::Close { id, reason } => {
            commands::close::run(&dir, &id, reason)?;
        }
        Command::Reindex => {
            commands::reindex::run(&dir)?;
        }
        Command::Search { query } => {
            commands::search::run(&dir, &query)?;
        }
    }

    Ok(())
}
