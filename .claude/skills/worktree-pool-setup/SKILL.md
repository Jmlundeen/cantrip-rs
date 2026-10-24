---
name: worktree-pool-setup
description: Opt this machine into the agents' worktree pool (tools\worktree.cmd) - install its Claude Code hooks in the gitignored .claude/settings.local.json, adopt the existing pool-<n> worktrees, and check that a subagent can join one and nobody else can write in it. Use when asked to set up, install, enable, check or remove the worktree pool or its hooks, or when `worktree.cmd acquire` says the hooks are not installed.
---

# Setting up the worktree pool

The pool (`tools/maintenance/worktrees.py`, its docstring the model; `coordinator.md` step 0 the procedure) is opt-in,
machine by machine. Its hooks go in `.claude/settings.local.json`, which is gitignored: never in the committed
settings, and never pushed. Until they are installed, `worktree.cmd acquire` refuses, so a contributor who has not
opted in is never sent into the pool.

1. **Install the hooks:** `tools\worktree.cmd hooks install`. It merges three hooks into the local settings, keeping
   everything else there: `SubagentStart` (tells a subagent its agent ID), `PreToolUse` on edits and shell commands
   (denies a slot to anyone but its member) and `SubagentStop` (a stopped agent leaves its slot). Running it twice is
   the same as once. Auto mode may refuse to let an agent change settings; the user then runs it at the prompt with `! python tools/maintenance/worktrees.py hooks install` (the `!` prompt is
   bash, which drops a backslash).
2. **Restart Claude Code** (or review the hooks in `/hooks`): hooks are read when a session starts. Check with
   `tools\worktree.cmd hooks check`.
3. **Adopt the slots:** `tools\worktree.cmd reconcile` lists the `.claude/worktrees/pool-<n>` worktrees and what each
   holds. Show the user any that are not clean - uncommitted files, or commits whose patches `origin/main` lacks -
   before `reconcile --fix`: it registers the clean ones as free and the rest as needing attention, and changes no
   files. A slot needing attention is the user's to land, push or discard; `reconcile --fix` frees it once it is clean.
   Nobody can work in an unleased slot (the hooks keep everyone out), so `status pool-<n>` shows what it holds and
   `clean pool-<n> --reason "..."` throws its uncommitted changes away, on the user's say; it refuses while the slot
   holds commits on no origin branch (push them to a branch first) unless given `--discard-commits`.
   No slots yet: `tools\worktree.cmd create --count 4` (each one a warm build: minutes, every core).
4. **Prove it**, once, with the pool idle:
   - `tools\worktree.cmd acquire --for "setup check"`, then `token <lease>`;
   - start a throwaway subagent with the token's prompt line and ask it to join, write a file `setup-check.txt` in the
     tree, delete it, and leave - it must succeed;
   - start a second subagent with no token and ask it to write a file in the same tree by its path - the hook must
     deny it;
   - `tools\worktree.cmd status` shows no member once the first agent has stopped, then `release <lease>`.

   Any step that behaves otherwise is reported to the user and the pool is not used until it is fixed.

**Opting out:** `tools\worktree.cmd hooks uninstall`, then restart. The pool's state (in the git directory) stays
for a later opt-in.
