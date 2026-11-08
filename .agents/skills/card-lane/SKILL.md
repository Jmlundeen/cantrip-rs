---
name: card-lane
description: Queue missing Cantrip cards on a canonical Git base, dispatch isolated implementors, and hand their checked commits to independent reviewers. Use for a set, list or batch of cards, or a solo card packet.
---

# Card lane

Use `tools/maintenance/card-lane.py` for dispatch and mechanical gates, `add-cards` for exact rules behavior, and
`write-game-script` for scenarios. An author pass is not independent review. Queue/run identity and handoffs live in
the Git common directory; regenerable packets live in each worktree's `.card-lane/`. Targets stay private.

## Coordinate

Resolve the canonical remote, issue repository, requested models/concurrency, engine scope and delivery first.
User choices override role defaults. For a contributor fork, specify upstream explicitly:

```sh
python tools/maintenance/card-lane.py queue --queue trk --file cards.txt --base-ref upstream/main --repo cantrip-rs/cantrip-rs --concurrency 2 --implementor-model gpt-6.1-sol --reviewer-model gpt-6.1-sol --engine --delivery pr
```

Other sources: `--format F`, repeated `--card Name`, or `--backlog [--list Cube | --file list.txt]`. The tool fetches
and inventories that exact Git tree, including faces. The original base SHA stays pinned. Repeating queue
reconciles pending entries on refreshed upstream; active claims stay intact. Use a new queue to change immutable
run choices. Intake also checks open PRs in the canonical repository: missing cards covered by changed card content
or Rust files become `deferred-pr`, with PR links, rather than implemented. Open/closed membership is cached five
minutes; `queue --refresh-prs` refreshes it immediately. File inventories are reused only for unchanged head/base
SHAs; face aliases come from complete current JSON. Closed unmerged work returns to pending, merged canonical
content retires it, and active claims remain intact. A failed refresh warns and applies no stale PR exclusions.
Build the queue before allocating slots; if no cards are pending, report implemented/deferred counts and stop. Pool growth remains
the user's choice; prior authorization suffices. Preserve unrelated leases.

Acquire with `tools/worktree.cmd acquire --for "card: next trk" --queue trk`; it uses the pinned base and caps
implementor leases at requested concurrency. Prepare the selected provider/model and prompt before minting the
single-use token, then mint it immediately before launch. Each role's first command is `join`. Pass token, queue,
worker id and tool location, never a slot path or lease. [PROMPT.md](PROMPT.md) is the role template; `prompt`
generates it from the manifest. Bind runtime settings to the user's chosen model; never silently substitute.

After worker tool completion, use `ready --queue trk` to dispatch from durable state. Do not wait for a prose
handoff when the artifact exists. Ready requires absent author membership, an existing lease, matching source SHA
and a clean tree. Give review a fresh token on that same lease to retain the author's artifacts. One member per
slot; evict only an agent known to have ended. Recover lost final chat responses from ready state. Retry recorded
issue-log failures without discarding a successful implementation.

Publish the shared feature branch for a PR. PR delivery never commits on main. Keep the lease through review and
validation when practical, then release after its commits are published to the fork. Explicit main delivery is
available only when the user requests it. The lane's own unmerged tool commits stay out of card branches.
Until merged, invoke the lane and its companion helpers from the supplied lane checkout, with the joined worktree as
the working directory. They read/write that worktree and use its private targets; borrowed tool locations never
select the source checkout. Generated prompts name the companion helpers. Custom tool bundles can set
`CANTRIP_LANE_TOOLS_DIR`; their actual bytes also participate in author receipts.

## Implement

After join, run `next --queue trk --id w1`. Repeating it recovers the same claim. `--issue N` binds the card issue;
engine scope comes from the authorized manifest or `--engine`. Read the packet and relevant rules/skill sections.
Fill `report.json` with printed-line implementation, test-point assertions, ruling proof or explanation, LOOK
answers and equivalent mutation explanations. Its schema and reference are generated with the packet.

Use `check --queue trk --id w1 --fast` during edits. Cheap scope/report/Oracle/scenario/lint/citation checks precede
focused corpus execution. Fast corpus uses the small registry in the `mutate` profile with debug assertions and
`CANTRIP_VERIFY=1`; missing dependencies or panics retry the full registry. Every run builds current source.
`--fast` skips coverage and mutation only.

Run full `check` once ready, then `finish --queue trk --id w1`. Finish runs author gates or reuses a recent full pass
whose actual source, scenarios, ignored evidence, tools/configuration and environment match. Scope and Oracle
remain commit-time checks. Changed input invalidates reuse; HEAD tree alone is never a cache key. Finish writes
Engine/Mechanics/Client commits by category and one `Card: Name` commit with `Closes #N`, shares it, freezes the
packet, logs the issue, records awaiting-review and leaves membership. Stop after finish. `commit` alone is a
deliberate partial handoff; resume with finish. `check-live.json` exposes phase, child PID, heartbeat, gate times
and return codes. Serialize edits/checks; recover an operation lock only after verifying its process ended.

Use `block --rule CR --note ...` for a genuine rule gap, or `release --reason ...` for unfinished work; both preserve
its diff. Never approximate a rule to pass a gate.

## Review and validate

Review the exact handoff SHA against add-cards requirements. Run `card-mutate.py Name --check`, coverage, automatic
mutations and semantic probes independently. `card-mutate.py Name --probes probes.json` accepts named entries with
repo-relative Rust `file`, unique `before`/`after` text and `expected_failing_assertion`. Each must fail its intended
assertion. Build errors and missing dependencies are not proof. The tool restores exact bytes and rebuilds/runs
the baseline. Review relevant shared callers/layer contracts when touched. Author receipts cannot replace reviewer
proof. Leave restored and clean, then end membership. Findings go to the author; don't silently change behavior.

`prepare --queue trk --card Name` warms only the focused path and runs no games. Pool creation warms common
fast-profile engine/mechanics, not all cards or the full suite. This follows the local mtgish split between focused
card checks and integration checkpoints. `prepare --full` prepares checked workspace test binaries, native host
and .NET on a validation lease at the pinned base. Prepare early within measured CPU/memory capacity, retain
artifacts through final validation, and never share/copy target directories.

After independent review, run required plain `tools/test.cmd` on the final tree, plus wall-clock tests only when
their scope changed. It emits fresh corpus accounting from scenario tests: pass/fail, known bugs, fixed markers
and missing cards. The consumer verifies run identity/completeness. Use it instead of a second global game run.
Measure gate times, selection read/parse counts, warm preparation/rebuild and final suite on identical inputs;
don't promise savings from a warm target alone. Coverage clears raw profiles and retains compatible builds.
