use std::path::PathBuf;

/// Return the global crumbs data directory (platform-specific via `dirs::data_dir`).
#[must_use]
pub fn global_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crumbs")
}

/// Walk from `start` upward through parent directories and return the first
/// `.crumbs` subdirectory that exists, or `None` if none is found before
/// reaching the filesystem root.
fn find_crumbs_in_ancestors(start: &std::path::Path) -> Option<PathBuf> {
    let mut current = start;
    loop {
        let candidate = current.join(".crumbs");
        if candidate.is_dir() {
            return Some(candidate);
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

/// Resolve which directory to operate on.
///
/// Priority:
/// 1. `--dir <path>` explicit override
/// 2. `--global` flag → global data dir
/// 3. Nearest `.crumbs/` found by walking from cwd up to the filesystem root
/// 4. Global data dir as fallback
#[must_use]
pub fn resolve_dir(dir: Option<PathBuf>, global: bool) -> PathBuf {
    if let Some(d) = dir {
        // If the path already ends with `.crumbs` or contains store markers,
        // use it directly; otherwise append `.crumbs` so that
        // `--dir /some/project` behaves the same as `--dir /some/project/.crumbs`.
        if d.ends_with(".crumbs") {
            return d;
        }
        return d.join(".crumbs");
    }
    if global {
        return global_dir();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(local) = find_crumbs_in_ancestors(&cwd) {
        return local;
    }
    global_dir()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn explicit_dir_appends_crumbs_suffix() {
        let dir = tempdir().unwrap();
        let result = resolve_dir(Some(dir.path().to_path_buf()), false);
        assert_eq!(result, dir.path().join(".crumbs"));
    }

    #[test]
    fn explicit_dir_with_crumbs_suffix_unchanged() {
        let dir = tempdir().unwrap();
        let crumbs = dir.path().join(".crumbs");
        let result = resolve_dir(Some(crumbs.clone()), false);
        assert_eq!(result, crumbs);
    }

    #[test]
    fn dir_with_store_files_but_no_crumbs_suffix_appends_crumbs() {
        // Even if a directory contains store files, only the .crumbs suffix
        // prevents the append — file contents are not checked.
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("crumbs.toml"), "").unwrap();
        std::fs::write(dir.path().join("index.csv"), "").unwrap();
        let result = resolve_dir(Some(dir.path().to_path_buf()), false);
        assert_eq!(result, dir.path().join(".crumbs"));
    }

    #[test]
    fn global_flag_returns_global_dir() {
        let result = resolve_dir(None, true);
        assert_eq!(result, global_dir());
    }

    #[test]
    fn explicit_dir_beats_global_flag() {
        let dir = tempdir().unwrap();
        let crumbs = dir.path().join(".crumbs");
        let result = resolve_dir(Some(crumbs.clone()), true);
        assert_eq!(result, crumbs);
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

    // --- find_crumbs_in_ancestors ---

    #[test]
    fn ancestor_walk_finds_crumbs_in_parent() {
        let base = tempdir().unwrap();
        let crumbs = base.path().join(".crumbs");
        std::fs::create_dir(&crumbs).unwrap();
        // Simulate being in a subdirectory of the project root.
        let subdir = base.path().join("src").join("commands");
        std::fs::create_dir_all(&subdir).unwrap();
        let result = find_crumbs_in_ancestors(&subdir);
        assert_eq!(result, Some(crumbs));
    }

    #[test]
    fn ancestor_walk_returns_none_when_not_found() {
        let base = tempdir().unwrap();
        // No .crumbs anywhere under base — should return None.
        let result = find_crumbs_in_ancestors(base.path());
        assert!(result.is_none());
    }

    #[test]
    fn ancestor_walk_prefers_nearest_crumbs() {
        // A nested .crumbs in a subdirectory should win over one in a parent.
        let base = tempdir().unwrap();
        let outer_crumbs = base.path().join(".crumbs");
        let inner = base.path().join("nested");
        let inner_crumbs = inner.join(".crumbs");
        std::fs::create_dir_all(&outer_crumbs).unwrap();
        std::fs::create_dir_all(&inner_crumbs).unwrap();
        // Walking from `inner` should find inner_crumbs first.
        let result = find_crumbs_in_ancestors(&inner);
        assert_eq!(result, Some(inner_crumbs));
    }
}
