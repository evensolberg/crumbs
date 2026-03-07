use std::path::Path;

use anyhow::{Context, Result, bail};
use dialoguer::Input;

use crate::{
    config,
    store_config::{self, StoreConfig, suggest_prefix},
};

/// Validate that a prefix contains only lowercase ASCII letters and digits.
/// This prevents path-traversal and malformed IDs caused by a crafted prefix.
fn validate_prefix(prefix: &str) -> Result<()> {
    if prefix.is_empty() {
        bail!("prefix must not be empty");
    }
    if !prefix
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        bail!(
            "prefix {:?} contains invalid characters; only lowercase letters and digits are allowed",
            prefix
        );
    }
    Ok(())
}

pub fn run(dir: &Path, prefix_override: Option<String>) -> Result<()> {
    if dir.is_dir() {
        println!("Already initialized at {}. Nothing to do.", dir.display());
        return Ok(());
    }
    std::fs::create_dir_all(dir).context("create .crumbs directory")?;

    let prefix = if let Some(p) = prefix_override {
        // --prefix given: skip interactive prompt
        let p = p.trim().to_lowercase();
        validate_prefix(&p)?;
        p
    } else {
        // Derive suggestion from directory context.
        let suggested = if dir == config::global_dir() {
            "glob".to_string()
        } else {
            // Suggest based on the project root (parent of .crumbs), not .crumbs itself.
            suggest_prefix(dir.parent().unwrap_or(dir))
        };

        loop {
            let entered: String = Input::new()
                .with_prompt("ID prefix")
                .with_initial_text(&suggested)
                .interact_text()
                .context("read prefix from terminal")?;

            let entered = entered.trim().to_lowercase();
            let candidate = if entered.is_empty() {
                suggested.clone()
            } else {
                entered
            };
            match validate_prefix(&candidate) {
                Ok(()) => break candidate,
                Err(e) => eprintln!("error: {e}"),
            }
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
