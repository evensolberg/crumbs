use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::store;

/// # Errors
///
/// Returns an error if the item is not found, `$EDITOR` is unset or invalid, or the editor exits
/// with a non-zero status.
pub fn run(dir: &Path, id: &str) -> Result<()> {
    let editor_str = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    // Split the editor string shell-style so that values like "emacsclient -t"
    // work correctly; Command::new treats its argument as a binary name, not a
    // shell command, so without splitting spaces are passed verbatim.
    let mut parts = shell_words::split(&editor_str).context("could not parse $EDITOR value")?;
    if parts.is_empty() {
        bail!("$EDITOR is empty");
    }
    let binary = parts.remove(0);

    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, _)) => {
            let status = std::process::Command::new(&binary)
                .args(&parts)
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
