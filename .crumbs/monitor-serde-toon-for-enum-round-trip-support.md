---
id: cr-n4x
title: Monitor serde_toon for enum round-trip support
status: open
type: feature
priority: 4
tags:
- cli
- gui
- import
created: 2026-04-08
updated: 2026-04-10
dependencies: []
phase: ''
---

# Monitor serde_toon for enum round-trip support

serde_toon currently cannot round-trip enum variants (e.g. Status::Open) via from_slice — it serialises them as bare strings but the serde-derived Deserialize impl rejects visit_str.

This blocks TOON support in 'crumbs import --file' and the GUI Import button.

Periodically check serde_toon_format crate for updates or a workaround. When fixed: add 'toon' to file_import::infer_format and parse_items, add .toon to the GUI file picker filter, add end-to-end TOON import test, update skill and README.

[2026-04-10] Can we use the json2toon craate as a (temporary) workaround?

[2026-04-10] Plan: fork bnomei/serde_toon, fix src/decode/serde.rs — remove 'enum' from forward_to_deserialize_any! (line 403-405), implement explicit deserialize_enum that calls visitor.visit_enum(s.into_deserializer()) for NodeKind::String nodes. Wire in via [patch.crates-io] in workspace Cargo.toml. Then enable 'toon' in file_import.rs infer_format() and parse_items(), add round-trip integration test in crumbs/tests/commands.rs, open PR upstream. One-file fix; same pattern as serde_yaml.

[2026-04-10] Full implementation plan:

Root cause: serde_toon_format v0.1.2 (github.com/bnomei/serde_toon), src/decode/serde.rs lines 403-405 — 'enum' is in forward_to_deserialize_any!, so deserialize_enum forwards to deserialize_any which calls visit_str. But #[derive(Deserialize)] enum visitors implement visit_enum, not visit_str. Last upstream release 2026-02-04, zero issues filed.

Fix: Remove 'enum' from forward_to_deserialize_any! and add explicit deserialize_enum on ArenaDeserializer:
- NodeKind::String: get the string, call visitor.visit_enum(s.into_deserializer()) — handles unit variants like 'status: open'
- NodeKind::Object with child_len == 1: wrap in EnumObjectAccess (a small struct implementing de::EnumAccess + de::VariantAccess) for newtype/struct/tuple variants
- _: return Err('expected enum')
Pattern mirrors serde_yaml's identical fix.

Steps:
1. gh repo fork bnomei/serde_toon on branch fix/enum-unit-variants
2. Apply fix to src/decode/serde.rs; add test in tests/enum_roundtrip.rs
3. Add to /Volumes/SSD/Source/Rust/crumbs/Cargo.toml:
   [patch.crates-io]
   serde_toon_format = { git = 'https://github.com/evensolberg/serde_toon', branch = 'fix/enum-unit-variants' }
4. In crumbs/src/commands/file_import.rs: add 'toon' to infer_format(), add Format::Toon arm in parse_items() calling serde_toon::from_slice, remove placeholder error
5. In crumbs/tests/commands.rs: add export-then-import round-trip test
6. Open PR upstream against bnomei/serde_toon
7. crumbs close cr-n4x

Verification:
  cargo check -p crumbs
  cargo nextest run -p crumbs
  crumbs export --format toon --output /tmp/test.toon && crumbs import --file /tmp/test.toon

Note: [patch.crates-io] removed once upstream publishes fix. GUI import (Tauri import_items command) benefits automatically — no separate GUI change needed.
