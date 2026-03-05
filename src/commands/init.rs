use std::path::Path;

use anyhow::{Context, Result};

pub fn run(dir: &Path) -> Result<()> {
    if dir.is_dir() {
        println!("Already initialized at {}. Nothing to do.", dir.display());
        return Ok(());
    }
    std::fs::create_dir_all(dir).context("create .crumbs directory")?;
    println!("Initialized crumbs store at {}", dir.display());
    Ok(())
}
