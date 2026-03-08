use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::{item::Status, store};

pub fn run(
    dir: &Path,
    source_id: &str,
    relation: &str,
    target_ids: &[String],
    remove: bool,
) -> Result<()> {
    let (src_path, mut src_item) = store::find_by_id(dir, source_id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {source_id}"))?;

    let today = Local::now().date_naive();
    let mut linked = Vec::new();

    for target_id in target_ids {
        let (tgt_path, mut tgt_item) = store::find_by_id(dir, target_id)?
            .ok_or_else(|| anyhow::anyhow!("no item found with id: {target_id}"))?;

        match relation {
            "blocks" => {
                if remove {
                    src_item.blocks.retain(|id| id != &tgt_item.id);
                    tgt_item.blocked_by.retain(|id| id != &src_item.id);
                    // Reopen target if nothing else blocks it.
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
            }
            "blocked-by" => {
                if remove {
                    src_item.blocked_by.retain(|id| id != &tgt_item.id);
                    tgt_item.blocks.retain(|id| id != &src_item.id);
                    // Reopen source if nothing else blocks it.
                    if src_item.blocked_by.is_empty() && src_item.status == Status::Blocked {
                        src_item.status = Status::Open;
                    }
                } else {
                    if !src_item.blocked_by.contains(&tgt_item.id) {
                        src_item.blocked_by.push(tgt_item.id.clone());
                    }
                    if !tgt_item.blocks.contains(&src_item.id) {
                        tgt_item.blocks.push(src_item.id.clone());
                    }
                    src_item.status = Status::Blocked;
                }
            }
            other => bail!("unknown relation: {other} (expected 'blocks' or 'blocked-by')"),
        }

        tgt_item.updated = today;
        store::rewrite_frontmatter(&tgt_path, &tgt_item)?;
        linked.push(tgt_item.id.clone());
    }

    src_item.updated = today;
    store::rewrite_frontmatter(&src_path, &src_item)?;
    store::reindex(dir)?;

    let targets = linked.join(", ");
    if remove {
        println!("Unlinked {} {} {}", src_item.id, relation, targets);
    } else {
        println!("Linked {} {} {}", src_item.id, relation, targets);
    }
    Ok(())
}
