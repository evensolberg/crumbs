use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::{item::Status, store};

fn update_item_file(path: &std::path::PathBuf, item: &crate::item::Item) -> Result<()> {
    let frontmatter = serde_yaml_ng::to_string(item)?;
    let raw = std::fs::read_to_string(path)?;
    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or("");
    store::atomic_write(path, &format!("---\n{frontmatter}---\n{body}"))?;
    Ok(())
}

/// `crumbs block <source> <targets>` — link source→targets and mark targets as blocked.
/// `crumbs block <source> <targets> --remove` — unlink and reopen targets if no other blockers.
pub fn run(dir: &Path, source_id: &str, target_ids: &[String], remove: bool) -> Result<()> {
    let (src_path, mut src_item) = store::find_by_id(dir, source_id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {source_id}"))?;

    let today = Local::now().date_naive();
    let mut linked = Vec::new();

    for target_id in target_ids {
        let (tgt_path, mut tgt_item) = store::find_by_id(dir, target_id)?
            .ok_or_else(|| anyhow::anyhow!("no item found with id: {target_id}"))?;

        if remove {
            src_item.blocks.retain(|id| id != &tgt_item.id);
            tgt_item.blocked_by.retain(|id| id != &src_item.id);
            // Reopen target only if nothing else blocks it anymore.
            if tgt_item.blocked_by.is_empty() && tgt_item.status == Status::Blocked {
                tgt_item.status = Status::Open;
            }
        } else {
            if !src_item.blocks.contains(&tgt_item.id) {
                src_item.blocks.push(tgt_item.id.clone());
            }
            if !tgt_item.blocked_by.contains(&src_item.id) {
                tgt_item.blocked_by.push(src_item.id.clone());
            }
            tgt_item.status = Status::Blocked;
        }

        tgt_item.updated = today;
        update_item_file(&tgt_path, &tgt_item)?;
        linked.push(tgt_item.id.clone());
    }

    src_item.updated = today;
    update_item_file(&src_path, &src_item)?;
    store::reindex(dir)?;

    let targets = linked.join(", ");
    if remove {
        println!("Unblocked: {} no longer blocks {}", src_item.id, targets);
    } else {
        println!("Blocked: {} blocks {}", src_item.id, targets);
    }
    Ok(())
}

/// `crumbs block --status <id>` — directly set an item's status to blocked (no link).
pub fn run_set(dir: &Path, id: &str) -> Result<()> {
    let (path, mut item) = store::find_by_id(dir, id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {id}"))?;
    if item.status == Status::Blocked {
        bail!("{} is already blocked", item.id);
    }
    item.status = Status::Blocked;
    item.updated = Local::now().date_naive();
    update_item_file(&path, &item)?;
    store::reindex(dir)?;
    println!("Blocked {}", item.id);
    Ok(())
}
