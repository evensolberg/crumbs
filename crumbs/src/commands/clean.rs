use std::path::Path;

use anyhow::Result;

pub fn run(dir: &Path) -> Result<()> {
    super::delete::run_closed(dir)
}
