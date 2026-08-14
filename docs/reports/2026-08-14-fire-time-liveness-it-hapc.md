# Dispatch report — it-hapc: fire-time liveness

## Against the RETURN spec — all four lines land

1. **`fire-routing` derivation** — `coord::fire_routing` (src/coord.rs, after `last_seen`) is the one derivation point: from a non-dispatch session, registered sessions filter to kind=dispatch (registry data, never re-validated), purview fit (every about-area of the item inside the session's areas), and `last_seen_age_secs` against `coord::DISPATCH_LIVE_SECS` (15 min). Any live → `FireRouting::Leave` with every live session unranked. Verified live: with the dispatcher heartbeat stamped fresh, `q dispatch it-xzp7` printed the leave offer naming session, age, charter, the dc-qyr5 claim-race line, and — it-xzp7 being [shaped] — the "feed carries ready items" note. Nothing dispatched, exit 0.
2. **Wake offer** — none awake → `FireRouting::Wake` enumerates all candidates (name order), each beside `coord::wake_command`: the repo-root `<name>-session.cmd` when present, the inline `cmd /c "set QUARRY_SESSION=… && claude"` otherwise (the same two roads `q session set` advertises). One candidate offered directly; several print "the user picks". Verified live: probe found `dispatcher` never-seen and offered `dispatcher-session.cmd`.
3. **`--solo` and the never-route cases** — new flag on `Cmd::Dispatch` skips the check, no reason demanded; dispatch-kind sessions, continuations (item already live-dispatched), and items no registered dispatcher covers all return `Fire`. Area-less items fit any dispatcher vacuously (dc-wngq all-areas charters).
4. **Advisory and stateless** — offer paths run before `ops::dispatch`, write nothing (no held entry, no status flip, no log event), exit 0 — plain register, deliberately not the `Error:` register. `ops::dispatch` signature untouched.

Tests: `fire_time_routing_offers_leave_and_wake` in tests/basic.rs covers all branches. Agent read `64 passed; 0 failed` (basic) + `1 passed` (surface lint) off raw lines. Spine claim cl-9v7c minted under the badge, sourced `file:src/coord.rs`.

Operational: `target/release/q.exe` was locked by the live `q serve`; the agent stopped it, built, relaunched it detached (answering). The heartbeat file stamped during the Leave probe was restored byte-for-byte.

## Provisional calls — dispatcher's judgment at harvest

1. **15-minute live window, heartbeat semantics untouched.** Accepted: the heartbeat only updates on write verbs, so an open-but-quiet dispatcher reads asleep; the window is forgiving and ages always print, so the user's eyes overrule. The constant is a judgment call wearing a precision costume (the agent's own words) — the printed age is the real surface. If liveness ever needs to mean "chat open," the heartbeat needs its own channel — filed as a thread (see below).
2. **Kindless sessions route like design.** Accepted — dc-crea says "from a non-dispatch session"; kindless is non-dispatch.
3. **Offer stops with exit 0, no refusal register.** Accepted — "a surface, never a gate" read correctly: no denial ceremony on the override.
4. **Acceptance lines set by the agent on its own item** (dispatched empty — the dispatcher's gap). The four lines match the settled spec and are accepted, but the shape — an agent grading its own contract — is the Goodhart pattern dc-grrb warns about; the agent flagged it itself. The empty-acceptance confrontation belongs at fire time with the dispatcher, not at join with the agent.

## Findings filed from this dispatch

- **The liveness echo:** a dispatched agent's q acts heartbeat the *dispatching* session (env-injected identity), so a design session with agents in flight reads "live" to fire-time routing — an echo of its own dispatches. Thread filed.
- **Observation gap instance:** this dispatch's harvest showed `tests/**` leased-but-untouched while tests/basic.rs was genuinely modified under the badge (test present, tree diff confirms, suite counts it). The write existed; the observed set missed it. Watch filed with this instance.

## Harvest verification (dispatcher)

Suite re-run at harvest, raw lines: `64 passed; 0 failed`, `1 passed; 0 failed`, all other harnesses `0 failed`. tests/basic.rs modification verified directly against the last commit despite the observed-set miss.
