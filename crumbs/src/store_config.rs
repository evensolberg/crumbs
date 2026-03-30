use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_PREFIX: &str = "cr";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub prefix: String,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            prefix: DEFAULT_PREFIX.to_string(),
        }
    }
}

#[must_use]
pub fn load(dir: &Path) -> StoreConfig {
    let path = dir.join(CONFIG_FILE);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return StoreConfig::default();
    };
    toml::from_str(&raw).unwrap_or_default()
}

/// # Errors
///
/// Returns an error if the config cannot be serialized or written to disk.
pub fn save(dir: &Path, cfg: &StoreConfig) -> Result<()> {
    let path = dir.join(CONFIG_FILE);
    let raw = toml::to_string(cfg).context("serialize config")?;
    std::fs::write(path, raw).context("write config.toml")?;
    Ok(())
}

/// Derive a suggested prefix from a directory name.
/// Takes the first letter of each word (split on `-`, `_`, and spaces),
/// lowercased, max 4 chars. Falls back to `DEFAULT_PREFIX`.
#[must_use]
pub fn suggest_prefix(dir: &Path) -> String {
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(DEFAULT_PREFIX);

    // Strip leading dot (e.g. ".crumbs" → "crumbs")
    let name = name.trim_start_matches('.');

    let initials: String = name
        .split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter_map(|part| part.chars().next())
        .take(4)
        .collect::<String>()
        .to_lowercase();

    if initials.is_empty() {
        DEFAULT_PREFIX.to_string()
    } else {
        initials
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn suggest_prefix_single_word() {
        assert_eq!(suggest_prefix(&PathBuf::from("crumbs")), "c");
    }

    #[test]
    fn suggest_prefix_hyphenated() {
        assert_eq!(suggest_prefix(&PathBuf::from("my-cool-app")), "mca");
    }

    #[test]
    fn suggest_prefix_underscored() {
        assert_eq!(suggest_prefix(&PathBuf::from("my_project")), "mp");
    }

    #[test]
    fn suggest_prefix_strips_dot() {
        assert_eq!(suggest_prefix(&PathBuf::from(".crumbs")), "c");
    }

    #[test]
    fn suggest_prefix_max_four() {
        assert_eq!(suggest_prefix(&PathBuf::from("a-b-c-d-e-f")), "abcd");
    }

    #[test]
    fn suggest_prefix_global_fallback() {
        // "glob" would come from the caller passing a path whose last component is "crumbs"
        // for the global store — tested indirectly via the init command
        assert_eq!(suggest_prefix(&PathBuf::from("glob")), "g");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = StoreConfig {
            prefix: "ma".to_string(),
        };
        save(dir.path(), &cfg).unwrap();
        let loaded = load(dir.path());
        assert_eq!(loaded.prefix, "ma");
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = load(dir.path());
        assert_eq!(cfg.prefix, DEFAULT_PREFIX);
    }
}
