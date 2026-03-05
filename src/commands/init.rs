use std::path::Path;

use anyhow::{Context, Result};
use dialoguer::Input;

use crate::{config, store_config::{self, StoreConfig, suggest_prefix}};

pub fn run(dir: &Path, prefix_override: Option<String>) -> Result<()> {
    if dir.is_dir() {
        println!("Already initialized at {}. Nothing to do.", dir.display());
        return Ok(());
    }
    std::fs::create_dir_all(dir).context("create .crumbs directory")?;

    let prefix = if let Some(p) = prefix_override {
        // --prefix given: skip interactive prompt
        p.trim().to_lowercase()
    } else {
        // Derive suggestion from directory context.
        let suggested = if dir == config::global_dir() {
            "glob".to_string()
        } else {
            // Suggest based on the project root (parent of .crumbs), not .crumbs itself.
            suggest_prefix(dir.parent().unwrap_or(dir))
        };

        let entered: String = Input::new()
            .with_prompt("ID prefix")
            .with_initial_text(&suggested)
            .interact_text()
            .context("read prefix from terminal")?;

        let entered = entered.trim().to_lowercase();
        if entered.is_empty() {
            suggested
        } else {
            entered
        }
    };

    let cfg = StoreConfig {
        prefix: prefix.clone(),
    };
    store_config::save(dir, &cfg)?;

    println!(
        "Initialized crumbs store at {} (prefix: {})",
        dir.display(),
        prefix
    );
    Ok(())
}
