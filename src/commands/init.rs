use std::path::Path;

use anyhow::{Context, Result};

pub fn run(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).context("create .crumbs directory")?;
    println!("Initialized crumbs store at {}", dir.display());
    Ok(())
}
