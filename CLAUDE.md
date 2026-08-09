# quarry — operational notes

The knowledge layer of this project lives in `graph/` and is reached through
the `q` binary (`target/release/q.exe`) — orient with `q query queue`,
`q query ready`, `q query shaping`; judgment layer via `q guide`. This file
holds only what the graph can't: how to build and work on this repo, on this
machine.

## Build & test (this machine)

- cargo is not on PATH: `"$HOME/.cargo/bin/cargo"` (bash) or
  `& "$env:USERPROFILE\.cargo\bin\cargo.exe"` (pwsh).
- **Use `TMP=B:\tmp TEMP=B:\tmp`** on cargo invocations while C: is
  space-constrained — the linker's temp files live on C: and a full drive
  fails with LNK1108 / "database or disk is full".
- `CARGO_BUILD_JOBS=4`, one build at a time — this machine has hung under
  parallel heavy builds.
- **Verify test results by reading the `test result:` line and its count** —
  never through a filter. A `grep -v "0 passed"` pipeline silently swallowed
  a `20 passed` line here once ("0 passed" is a substring).

## Conventions

- **Single-thread replies.** One topic per message, kept short — the user
  pulls the next thread. Multi-topic work still happens; reporting it does
  not all arrive at once. End replies with a short options list drawn from
  open threads. (User ruling, 2026-08-09, asked for repeatedly across
  sessions.)
- **Confirm before building.** When the user reflects on an approach, settle
  the pattern in conversation before writing graph or code — don't jump to
  correcting. An explicit go-ahead ("let's settle this") green-lights the
  writes. (User ruling, 2026-08-09.)
- **`QUARRY_ACTOR` is auto-injected by the session hook** (from the model
  recorded at SessionStart) for chats running in this repo — don't set it by
  hand. Export it manually only when working from outside hook coverage
  (e.g. a parent-directory session). If unset entirely, derived provenance
  safely defaults to `assistant` — user provenance is only ever explicit.
- After editing `src/teach.rs` (guide/skill/hooks): rebuild, then re-run
  `q init --claude` to regenerate `.claude/skills/quarry/SKILL.md` and hook
  wiring. After graph or docs changes: `q view` to regenerate the page.
- Editing `docs/DESIGN.md` drifts its registered doc node — expected;
  review your own change and `q affirm` it (wrap will remind you).
- Commit graph/ changes with the work they belong to; commits end with
  `Co-Authored-By: Claude <model> <noreply@anthropic.com>` naming the model
  that did the work.

## Known compromises (deliberate, watch-listed)

- `serde_yaml` is unmaintained; it works and is contained to
  `model.rs`/frontmatter — swap candidate if it ever bites.
- Lease/intent/heartbeat files use best-effort locking — adequate for one
  human orchestrating a few sessions, not for real concurrency.
- The alert check scans the whole event log — fine below ~10k events;
  shard-aware scanning is the fix if it ever shows up in hook latency.
- CLI output is title-first prose for agents and humans; scripts should not
  parse it — extract ids from the trailing parenthetical or use `q find`.
  A machine-readable output mode is a filed sketch item.
