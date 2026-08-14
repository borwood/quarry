# Dispatch report — it-wub5: kind-shaped wake

## Against the RETURN spec — `kind-shaped-wake` lands

- **Dispatch-kind wake briefs lead with the dispatcher's owes.** Both wake surfaces (`q session resume`, SessionStart orient) render, for a dispatch-kind session: ready in purview first (or "ready to dispatch: none in purview" when the feed is empty), in-flight items each carrying `— q harvest <id>`, then a homework-residue block (stale refs capped at 5 with a `q query behind` tail, plus unharvested dispatches whose item left in-flight without a harvest — deduped against the in-flight shelf). The owed-threads block is omitted on both surfaces (dc-wngq: threads are not a dispatch session's to settle).
- **No two-way branching (dc-ad8b honored).** `coord::wake_shape(Option<&str>) -> WakeShape` is the one place kind maps to a wake shape — a match with a catch-all, so unknown and absent kinds keep the generic brief and a new kind's wake is one new arm there. Surfaces consume the derived shape's flags (`dispatcher_lead`, `owed_threads`), never the kind string. One renderer, `render::dispatch_wake`, serves both surfaces (the charter-at-wake pattern applied again).
- **Design and kindless wakes unchanged** — verified live against the real graph: `quarry` (design) orient/resume show the owed enumeration and old in-flight block byte-for-byte; unbound-chat orient unchanged; `dispatcher` orient/resume show the new lead. The orient was restructured to resolve the wake session before printing (needed to gate the owed block), with generic output order preserved exactly.

**Where things live:** `src/coord.rs` (WakeShape + wake_shape, after `kind_line`), `src/render.rs` (`dispatch_wake`, before `harvest`), `src/main.rs` (both wake surfaces consume the shape), `tests/basic.rs` (`wake_shape_follows_kind_at_one_match_point`, `dispatch_wake_leads_with_ready_inflight_and_homework`).

**Tests:** `63 passed; 0 failed` (basic) + `1 passed; 0 failed` (surface lint), read from raw result lines. Release binary rebuilt; single build at a time throughout.

**Graph:** spine claim cl-br3p (`kind-shaped-wake`) minted under the badge, sourced `file:src/render.rs` and `file:src/coord.rs` (blob-stamped). During the smoke test the boundary guard (cl-ahzk) correctly refused the agent's own `q session resume` attempt when the hook re-injected its joined identity — fired as designed.

## Provisional calls — dispatcher's judgment at harvest

1. **"Homework residue" read as behind refs + unharvested dispatches** (the ⚠ side of print_homework, with ready covering the ✔ side). Accepted — the graph never defined the term at the wake tier; this reading is the natural one.
2. **The orient census line (including the owed count) stays for dispatch wakes** — block omitted, census kept. Accepted: the count is information, the enumeration is an assignment.
3. **Arrivals/recent-acts stay on dispatch resume** — they report change, they don't assign settling. Accepted.
4. **A queued thread may legitimately appear in a dispatch wake as the source of its own stale ref in homework.** Accepted — ref hygiene is the dispatcher's even when the ref lives on a thread; what dc-wngq withholds is settling, not maintenance. Pinned by a scoped assertion in the agent's test.
5. **`wake_shape` consults the kind string as a mapping site while cl-3rx9 says `parse_kind` is the only place the string is judged.** Accepted under judged ≠ mapped: parse_kind validates and can refuse; wake_shape's catch-all refuses nothing. If a third mapping site ever appears, fold mapping into coord behind one function and let cl-3rx9 absorb the clause.

## Reflections (agent's own)

- The one surprise was the agent's own test tripping on correct behavior — the queued thread in homework, call 4 above.
- Noticed in passing: the dispatcher session has never logged an act, so orient's "new in your purview" block floods with ~100 lines — pre-existing, adjacent to it-b2sp (dispatcher context burden), not touched.

## Harvest verification (dispatcher)

Suite re-run at harvest, raw lines: `63 passed; 0 failed`, `1 passed; 0 failed`, all other harnesses `0 failed`. Observed-vs-leased exact: four files touched, all inside the lease, nothing leased unused.
