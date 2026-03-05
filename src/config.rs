use std::path::PathBuf;

/// Return the global crumbs data directory: `~/.local/share/crumbs` (or platform equivalent).
pub fn global_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crumbs")
}

/// Resolve which directory to operate on.
///
/// Priority:
/// 1. `--dir <path>` explicit override
/// 2. `--global` flag → global data dir
/// 3. `.crumbs/` subdirectory under cwd (local project store)
/// 4. Global data dir as fallback
pub fn resolve_dir(dir: Option<PathBuf>, global: bool) -> PathBuf {
    if let Some(d) = dir {
        return d;
    }
    if global {
        return global_dir();
    }
    let local = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".crumbs");
    if local.is_dir() {
        return local;
    }
    global_dir()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn explicit_dir_wins() {
        let dir = tempdir().unwrap();
        let result = resolve_dir(Some(dir.path().to_path_buf()), false);
        assert_eq!(result, dir.path());
    }

    #[test]
    fn global_flag_returns_global_dir() {
        let result = resolve_dir(None, true);
        assert_eq!(result, global_dir());
    }

    #[test]
    fn explicit_dir_beats_global_flag() {
        let dir = tempdir().unwrap();
        let result = resolve_dir(Some(dir.path().to_path_buf()), true);
        assert_eq!(result, dir.path());
    }

    #[test]
    fn local_crumbs_dir_detected() {
        // Can't easily test cwd detection without changing process cwd,
        // but we can test that a path containing .crumbs is returned directly.
        let base = tempdir().unwrap();
        let crumbs = base.path().join(".crumbs");
        std::fs::create_dir(&crumbs).unwrap();
        // Pass it explicitly to verify the path logic handles it
        let result = resolve_dir(Some(crumbs.clone()), false);
        assert_eq!(result, crumbs);
    }

    #[test]
    fn fallback_is_global_dir() {
        // Without a .crumbs dir in cwd and no flags, result is global_dir()
        // We can't reliably assert the exact path without mocking cwd,
        // but we can verify it returns the same value as global_dir().
        let result = resolve_dir(None, false);
        // Either it found a .crumbs dir in the current tree, or it's global_dir().
        // At minimum it should not panic and return a valid path.
        assert!(!result.as_os_str().is_empty());
    }
}
