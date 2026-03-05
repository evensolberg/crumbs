use std::path::Path;

use anyhow::{Result, bail};

use crate::store;

pub fn run(dir: &Path, id: &str) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, _)) => {
            let status = std::process::Command::new(&editor)
                .arg(&path)
                .status()?;
            if !status.success() {
                bail!("editor exited with status: {status}");
            }
            store::reindex(dir)?;
        }
    }
    Ok(())
}
