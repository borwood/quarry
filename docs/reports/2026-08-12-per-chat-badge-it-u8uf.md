# Dispatch report: per-chat badge lands (it-u8uf)

Dispatched 2026-08-12 under dc-ydvb — the first ruling-compliant hand-off:
dispatched from the decisions session, built by a spawned agent, harvested by
the dispatcher. Lease `src/**` + `tests/**`; 7 files observed, all inside.
Verified twice: agent-run and dispatcher-run `cargo test`, raw `test result:`
lines both reading 53 passed / 0 failed (basic) and 1 passed / 0 failed
(surface lint).

## Agent's report, against the RETURN spec

### `per-chat-badge` — lands

`graph/.dispatch.json` is now a `DispatchMap` (src/coord.rs): `held` maps one
dispatching-chat key → `DispatchState`, plus `acting` (below). Key choice:
`chat:<id>` where the session hook's injection reaches the dispatching shell,
`session:<name>` fallback (`coord::dispatch_key`). The chat id reaches q
processes because the session hook now injects `QUARRY_CHAT` into every
Bash/PowerShell command alongside SESSION/ACTOR (src/teach.rs) — the hook is
the only process that knows it; env wins as with the others. `ops::dispatch`
refuses only when this chat's key already holds a different item ("parallel
dispatch belongs to parallel chats"); a parallel chat dispatches freely —
asserted in `dispatch_one_act_then_harvest`, which now runs two live badges
side by side. Backward compat: a legacy single-slot file parses, migrates
under its session key in memory, and persists in the new shape at first save
— verified live on this dispatch's own legacy state file, which migrated
mid-flight without losing cursor or acceptance. Test:
`legacy_single_slot_dispatch_state_migrates`.

### `badge-chat-resolution` — lands

`coord::badge_for(store, chat_id, session)` resolves for the ACTING context
only: env `QUARRY_DISPATCH` → this chat's held entry → this chat's learned
`acting` association → this session's entry → None. Consumers rewired:
`store::log_event` stamping, the guard hook (main.rs passes the hook's
session_id + bound session), `boundary_refusal`, `observe_write`, and
`render::harvest` (per-item `dispatch_for_item`). The mechanism that makes
this real for agents in foreign chats: a badged q act carrying both
`QUARRY_DISPATCH` and `QUARRY_CHAT` records `acting[chat:<id>] = badge`
(`note_acting_chat`, called from log_event), so hook processes — blind to
shell env — resolve that chat's file writes from then on. Verified live: the
agent chat's association appeared in the real state file after its first
claim mint; its writes accrued under `item:it-u8uf` throughout (the trace
shows exactly its seven files). Boundary semantics per the ruling: wrap
refuses for a chat with its own badge (held, acting, or env — child-tested
for all three transports); a chat with no badge wraps freely while other
chats' dispatches fly — the decisions-session shape, asserted in
`wrap_refuses_badged_then_regenerates_view_when_clear`.

Spine claims minted under the badge: cl-amza (`per-chat-badge`), cl-vsew
(`badge-chat-resolution`), both sourced to src/coord.rs and linked supports
it-u8uf; cl-vsew also supports it-9p3v.

### Documented holes (the honest key's edges)

1. An agent's file writes before its first badged q act resolve no badge —
   the association is learned, so observation is ordering-dependent; the
   payload now says export early, before the first file write.
2. An agent that never exports the badge is invisible to both stamping and
   observation — the old singleton caught those writes but attributed them
   wrongly under parallelism; no-attribution was chosen over mis-attribution.
3. Same class for boundary verbs: a forgetful agent chat with no badged act
   yet could wrap — the ruling's trade, since machine-global refusal is what
   blocked the decisions session.
4. Two hook-uncovered shells sharing one session name still collide on
   `session:<name>` — with hook coverage, chat keys make this moot.
5. The same item dispatched from two chats of one session yields two held
   entries; drift-cursor updates land on whichever `dispatch_for_item` finds
   first.

### Agent's reflections (verbatim substance)

The design surprise: "keyed by the session-hook binding" underdetermines the
mechanism — the binding alone gives hook processes resolution, but q
processes in the dispatching shell would have only session identity, and
agents in foreign chats nothing. Injecting `QUARRY_CHAT` was the load-bearing
move; once the chat id rides every shell command, the keying, resolution, and
learned association fall out. Doubt: `acting` is a second mechanism where the
acceptance implies one, and it is learned state — scoped to live dispatches
and cleared with them, but a dispatcher on pure-env badges gets no
association at all. Friction: the threaded test suite's env-race constraint
pushed positive transport tests into child processes; the in-process tests
read as a wall of negative controls, and the flip sites carry comments saying
the negatives ARE the semantic flip. Lingering unease: watching its own
dispatch's state file migrate under it while the old hook binary still
stamped its writes — it worked because both codepaths agreed on `item:<id>`
as the accrual key, which was prior design luck, not control.

## Dispatcher's judgment

Both outcomes hold: independently re-run tests agree, the harvest seat showed
all seven observed files inside the lease, and the resolution-order seam
(env → held → acting → session → None) reads as designed. Landed done and
released. it-9p3v (badge trace machine-scoped) closed done on cl-vsew's
support with one honest residue recorded there: a chat that dispatches and
keeps acting stamps its own mid-flight graph acts under its held badge —
correct-by-key, still noise-by-role; under dc-ydvb's preferred flow
(dispatching happens from a steward chat) the residue shrinks to the steward's
own boundary acts, and harvest clears it. it-e6wq (retire overbreadth under
an unrelated badge) left open pending its own look. Operational note from the
agent: the release build was blocked by a running `q serve` holding the exe
lock (ui-cleanup's serve loop); the agent stopped it and relaunched it
detached on the new binary.
