use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::store;

fn update_item_file(path: &std::path::PathBuf, item: &crate::item::Item) -> Result<()> {
    let frontmatter = serde_yml::to_string(item)?;
    let raw = std::fs::read_to_string(path)?;
    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or("");
    std::fs::write(path, format!("---\n{frontmatter}---\n{body}"))?;
    Ok(())
}

pub fn run(dir: &Path, source_id: &str, relation: &str, target_ids: &[String], remove: bool) -> Result<()> {
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
                } else {
                    if !src_item.blocks.contains(&tgt_item.id) {
                        src_item.blocks.push(tgt_item.id.clone());
                    }
                    if !tgt_item.blocked_by.contains(&src_item.id) {
                        tgt_item.blocked_by.push(src_item.id.clone());
                    }
                }
            }
            "blocked-by" => {
                if remove {
                    src_item.blocked_by.retain(|id| id != &tgt_item.id);
                    tgt_item.blocks.retain(|id| id != &src_item.id);
                } else {
                    if !src_item.blocked_by.contains(&tgt_item.id) {
                        src_item.blocked_by.push(tgt_item.id.clone());
                    }
                    if !tgt_item.blocks.contains(&src_item.id) {
                        tgt_item.blocks.push(src_item.id.clone());
                    }
                }
            }
            other => bail!("unknown relation: {other} (expected 'blocks' or 'blocked-by')"),
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
        println!("Unlinked {} {} {}", src_item.id, relation, targets);
    } else {
        println!("Linked {} {} {}", src_item.id, relation, targets);
    }
    Ok(())
}
