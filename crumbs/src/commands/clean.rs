use std::path::Path;

use anyhow::Result;

/// # Errors
///
/// Returns an error if closed items cannot be loaded or deleted.
pub fn run(dir: &Path) -> Result<()> {
    super::delete::run_closed(dir)
}
