---
id: cr-njj
title: Branch-independent crumbs persistence
status: open
type: idea
priority: 4
tags:
- project/crumbs
- git
created: 2026-03-07
updated: 2026-04-07
dependencies: []
phase: ''
---

# Branch-independent crumbs persistence

## The Problem

`.crumbs/` is committed to git. When you switch branches, git checks out the branch's version of `.crumbs/` — or removes it entirely if the branch predates crumbs. Tasks created on `main` vanish when you check out a feature branch. There is currently no mechanism to keep one persistent task list across all branches.

---

## Approaches (ranked best-to-worst)

### 1. Git Worktree + Orphan Branch (Recommended)

Mount `.crumbs/` as a separate git worktree pointing at a dedicated orphan branch (e.g., `crumbs-store`). Branch switches on the main worktree never touch a worktree, so crumbs are always there regardless of what branch you are on.

One-time setup (could become `crumbs worktree-init`):

```sh
git checkout --orphan crumbs-store
git rm -rf .
mkdir .crumbs && crumbs init --prefix cr
git add .crumbs/ && git commit -m "Init crumbs store"
git checkout main
echo ".crumbs" >> .gitignore
git worktree add .crumbs crumbs-store
```

What happens:
- `.crumbs/` is physically present in the working tree but controlled by its own branch
- `crumbs` tool sees a normal `.crumbs/` directory — zero changes to crumbs required
- `git switch feature-xyz` → `.crumbs/` unaffected
- `git push origin crumbs-store` → tasks backed up and synced to remote
- `git log crumbs-store` → full task history

Pros: Branch-independent, remote-synced, full git history, zero crumbs changes needed
Cons: `.crumbs` must appear in main branch `.gitignore`; setup is multi-step

Optional crumbs support:
- `crumbs worktree-init [--branch <name>]` — automates the setup
- `crumbs worktree-init --migrate` — moves existing `.crumbs/` into the new branch

---

### 2. `.git/crumbs/` Git-Local Store

Add a new tier to the store resolution chain: look for `.git/crumbs/` inside the git root before falling back to the global store. `.git/` is never touched by checkouts.

New resolution chain:
1. `--dir <path>` explicit override
2. `--global` flag
3. `.crumbs/` under cwd (current)
4. NEW: `.git/crumbs/` under the git root (branch-independent, local-only)
5. Global store as fallback

Changes required:
- `config.rs`: walk up from cwd to find `.git/`, then check `.git/crumbs/`
- `commands/init.rs`: add `--git-local` flag
- No other changes

Pros: Zero setup; survives all branch operations; no orphan branch needed
Cons: `.git/` is not pushed to remote — tasks are local-only; `git clone` loses all tasks

---

### 3. Nested Git Repository

Make `.crumbs/` its own git repo. The outer repo`.gitignore` excludes `.crumbs/`. The inner repo tracks items independently and can be pushed to a separate remote.

Pros: Fully independent, full git semantics
Cons: Most complex; two repos to manage; crumbs has no awareness of the inner git repo

---

### 4. Git Hook — Stash and Restore

`post-checkout` hook that saves/restores `.crumbs/` on branch switch.

Pros: No architectural changes
Cons: Error-prone; partial failures corrupt state; hooks must be installed per-clone

---

### 5. Global Store (already exists)

Use `crumbs --global`. Already fully supported.

Pros: Zero changes, works today
Cons: Not per-project; no git history for tasks

---

## Recommended Path Forward

Short-term: Document the worktree pattern in README (no code changes needed).

Medium-term: Add `crumbs worktree-init [--branch name] [--migrate]` command.

Long-term: Add `.git/crumbs/` resolution tier + `crumbs push-store` / `crumbs pull-store` for teams who do not want an orphan branch.

---

## Files to Change (worktree-init)

- `crumbs/src/main.rs` — Add WorktreeInit subcommand
- `crumbs/src/commands/worktree_init.rs` — New: orchestrates git commands
- `crumbs/src/commands/mod.rs` — Expose new module
- `README.md` — Add branch-independent setup section

## Files to Change (.git/crumbs/ tier)

- `crumbs/src/config.rs` — Add find_git_root() + new resolution step
- `crumbs/src/main.rs` — Add --git-local flag to init
- `crumbs/src/commands/init.rs` — Handle --git-local target path
