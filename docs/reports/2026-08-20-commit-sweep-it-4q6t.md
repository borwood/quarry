# Dispatch report — it-4q6t: commit sweep guarded

Second worktree dispatch (serial run, day 2). Agent report verbatim; dispatcher's
landing addendum follows.

## Outcomes against the RETURN SPEC, line by line

The single acceptance line, decomposed; each clause with its method:

- **Sweep-shaped staging denies at the shell hook with the asker's own git add line
  in hand** — built. `teach::staging_guard` rides the existing `q hook session`
  (Bash|PowerShell PreToolUse); `main.rs` calls it ahead of identity injection and
  exits 2 on deny. Detection is `teach::sweep_shape`: pure string matching over
  `git add graph` (and graph dirs, backslash and cwd-relative forms), `-A`/`--all`/
  `-u`, `.` (cwd-scoped), globs into graph/, and `git commit -a` (modeled as
  tracked-only capture — it cannot sweep an untracked, freshly minted node file, and
  the guard knows). Method: pinned by
  `staging_guard_denies_the_sweep_with_the_askers_line_and_offers_adoption_when_stale`
  and `sweep_shapes_match_bulk_staging_and_explicit_paths_pass` in tests/basic.rs,
  plus a live smoke feeding hook JSON to the built binary: `git add -A` → exit 2 with
  the C6 refusal; the refusal hands `git add <asker's exact files>` plus the
  exempt-ledger ride-along.
- **Ownership of mixed files derives last-writer-from-the-log** — built.
  `queries::commit_sets` (pure): a node file's owner is the session with the last
  stamped event on the node since the file's prior commit (per-file boundary via
  `git log -1 --format=%cI`); earlier editors in the window are carried, annotated
  ("carries X's earlier edits — worth naming in the commit message"), so a mixed file
  lands in exactly one commit-set. Unstamped windows land in an unowned remainder
  that blocks no sweep. Method:
  `commit_set_ownership_derives_last_writer_and_annotates_carried_edits` pins owner,
  carry, boundary exclusion, and unstamped fall-through.
- **Foreign files list with owner last-seen age; stale flips to explicit-path
  adoption** — built. Fresh owners (heartbeat under `OWNER_STALE_SECS` = 1h; agent's
  constant choice, see reflections) render leave-for-the-owner lines with age; stale
  or never-seen owners flip to `git add <exact paths>` + `git commit -m "adopt
  <owner>'s uncommitted graph nodes: <ids> (owner last seen ...)"` — which passes the
  guard by construction. Method: both branches pinned in the guard test; both
  observed live (fresh: "last seen 0m ago"; backdated: full adoption offer).
- **Retire offers last-rites commit** — built. `q session retire` reads the
  retiree's commit-set before the heartbeat clears and prints the labeled commit
  offer. Method: live smoke — retiring a session with one pending node printed the
  file, the add line, and the labeled commit message.
- **Wrap echoes the commit-set** — built, replacing the old bare "N uncommitted
  change(s)" count: per-owner split with `[yours]` marking, exempt-ledger line,
  shared files, teaching close. Method: live smoke output confirms the full render.
- **Explicit-path staging always passes** — holds by construction (only
  directory-shaped/bulk tokens match) and is pinned in both guard tests plus the
  live hook smoke (explicit add → exit 0, injection proceeds normally).
- **`q query commit-set` is the advertised pull handle** — built (`Query::CommitSet`,
  kebab name `commit-set`); both the deny and the wrap echo advertise it; the query
  renders full detail with atom refs, adoption offers, ledger and shared buckets.
  Method: live smoke.
- **Exempt ledger** — `graph/log/*.jsonl` never splits, never blocks, rides along
  with any commit; `git add graph/log` is shape-matched but captures no node files,
  so it stays silent. Pinned in the sweep-shape test.
- **Worktree forks are skipped** — a guard firing where `work_root != root` would
  judge the wrong tree; the fork's inert graph checkout is not this graph's staging
  concern. Encoded and commented in `staging_guard`.

## Test results (verbatim, read raw)

```
test result: ok. 111 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.15s   (tests/basic.rs — includes the 3 new pins)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s     (store_lint)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s     (surface_lint)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s     (worktree_proof)
```

## Mechanics

- Branch `worktree-agent-a7b48315e4933788d`, commit `e9d5484` ("The sweep meets its
  guard: the log owns each pending node, explicit paths pass"). Fast-forwarded into
  main at landing.
- Files: src/teach.rs (sweep_shape, SweepShape, staging_guard), src/queries.rs
  (commit-set section, OWNER_STALE_SECS, age_phrase), src/main.rs (hook arm wiring,
  wrap echo, retire last-rites, Query::CommitSet), tests/basic.rs (3 tests).
- Graph writes under the badge: vein claim cl-2jas (`commit-set`, about cli+process,
  source file:src/teach.rs), linked supports it-4q6t; affirmed cl-qm6t (its
  instrument, tests/basic.rs, moved under the agent's hand and re-run green).
  Source/instrument blob stamps hashed at the fork per dc-g5x5 — true after the
  fast-forward merge.

## Owed at the canonical tree — dispatcher's disposition

1. `cargo build --release` — **done at landing** (guard now live; hooks run the new
   exe).
2. `q init --claude` — **done at landing; confirmed no-op** (code-only teach.rs
   change, wiring untouched).
3. The canonical tree's uncommitted graph nodes from this arc — committed at landing
   under the new mechanism itself (explicit paths via `q query commit-set`).

## Agent reflections (verbatim)

- **The mechanism guarded its own birth.** Midway through, the canonical graph
  accrued this dispatch's own uncommitted nodes under my badge — exactly the
  collision class the item describes, now visible in `q query commit-set` once
  built. The design's "the log is the ownership oracle" held with zero special
  cases: my events stamp my session, so my mints are my commit-set.
- **One genuine surprise:** `git status --porcelain` collapses a fully-untracked
  directory to one `?? graph/` line, which would have let every node file in a fresh
  repo evade ownership. `-uall` fixes it; the first test run caught it. Worth
  remembering: the guard's git interrogation has to be adversarial about git's own
  summarization.
- **A judgment call the graph didn't make for me:** `OWNER_STALE_SECS` = 1 hour,
  with never-seen counting stale. The brief said "stale owners flip" without a
  threshold. Deliberately more forgiving than `DISPATCH_LIVE_SECS` (15m) since a
  design session mid-conversation goes log-quiet legitimately — and the flip changes
  only the *teaching*, never enforcement, so the cost of being wrong is one
  suboptimal suggestion. If the user wants a different number or wants it derived,
  it is one constant.
- **A known best-effort edge:** the plain `git commit` (no `-a`) of an index another
  session already staged is not detected — the harm happened at *their* staging
  time, which the guard now polices; and `git -C <path>` / `git -c k=v` global-flag
  forms can slip the tokenizer. All within the sanctioned write-guard posture
  (string-match, wrap backstops, evasion self-inflicted), but a future reader should
  know the seams.
- **Marginal behavior note:** a denied Bash call exits before `session_hook_output`,
  so it neither counts a counter-voice turn nor consumes a pending adopt-request
  that call — both self-heal on the next tool call.
- **Worktree dispatch friction (for th-mbbb):** the arc was smooth end to end —
  join rendered the brief, q acts landed at the canonical store, file writes stayed
  in the fork, the write-guard echoed the contract on first edit, and blob stamps
  hashing at the fork is the right call for post-merge truth. Two frictions:
  (1) the brief's 149KB join output far exceeds what a context reads comfortably —
  I had to page it from a persisted file; (2) verifying hook-arm behavior live
  requires a scratch store with *seven* env vars manually cleared per shell because
  the session hook re-injects them into every command — the injection that makes
  dispatch identity seamless makes stepping outside it laborious. Both are
  observations, not blockers.

## Dispatcher's landing addendum — serial-run evidence for th-mbbb

Second consecutive worktree arc, same collection order as the first
(harvest → judge → fast-forward merge → rebuild → init → register → done+release →
explicit-path commit → remove worktree and branch); dc-g5x5 held again with nothing
to collect on the graph side. New evidence this run: (1) the join brief rendered at
149KB — brief size under supports-carried prior reports is a scaling question for
dispatch context; (2) scratch-store hygiene under injected identity took seven
manual env unsets — the pin's convenience inverts for sandboxing (second sighting;
first was the counter-voice run); (3) the landing commit itself ran under the
freshly built staging guard with explicit paths from `q query commit-set` — the
landed capability collected its own landing.
