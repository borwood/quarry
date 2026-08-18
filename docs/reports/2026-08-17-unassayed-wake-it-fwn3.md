# Dispatch report — it-fwn3: load-bearing unassayed claims surface on the design wake

Sanity at close: full suite green (basic: `test result: ok. 96 passed; 0 failed`; surface_lint: 1 passed), release binary rebuilt, `q query load` smoke-run against the real graph, and the diff confined to the leased write-set (src/main.rs +31, tests/basic.rs +80).

## Against the RETURN spec

**Outcome 1 — lands `unassayed-wake`: MET.** Both design-shaped wake surfaces — `q session resume` and the SessionStart orient (`q hook orient`) — now count `queries::load_bearing_unassayed` beside owed threads, inside the same `shape.owed_threads` gate as the shaped-awaiting-acceptance sibling (cl-2sk9), with the query command in hand (`q query load`). Sites: src/main.rs (orient ~line 1110, resume ~line 1441). Method: pinned by the new test `design_wake_counts_load_bearing_unassayed` — design wake shows the line, dispatch wake omits it, ratifying the claim clears it. Full suite 96+1 passed, 0 failed, read off the raw `test result:` lines.

**Outcome 2 — holds the boundary: MET, by construction.** The wake reuses `queries::load_bearing_unassayed` (cl-hv3m), which is weight-filtered (`w > 0`) — zero-holds unassayed cannot reach the wake through any code path; it stays pull-only. Method: the test mints two unassayed claims, one holding a build and one holding nothing, and asserts the wake counts exactly 1. th-jvwe (general shaping-stall pressure) is untouched.

## Graph writes

- **cl-2zna** — vein claim `unassayed-wake`, kind-explicit, sourced file:src/main.rs, about ar-c7f5; body cites it-fwn3, dc-drr6, dc-p6z4, cl-hv3m, cl-2sk9, dc-wngq, th-jvwe (all id-echoes resolved at mint).
- **cl-2zna supports it-fwn3** — the landing edge.

## Composed strings (flagged per the standing flag channel, dc-vzvf)

- Orient: `load-bearing unassayed: {N} claim(s) builds stand on with no judge on record — fool's gold risk rises with weight (q query load)`
- Resume: same with `in your purview` after `claim(s)`.
Register drawn from the ratified ASSAY_WARNING; the "assay on next touch, or refute" teach left to the query header the command leads to, keeping the wake line sibling-length.

## Interpretive calls (flagged)

1. **Gating**: "beside owed threads" read as the same `shape.owed_threads` gate, not a new WakeShape field — dispatch omits via the existing shape; kindless and unknown kinds count, matching cl-2sk9's precedent.
2. **Scoping**: purview-filtered at resume, graph-wide at orient — mirroring the awaiting-acceptance sibling exactly; claims scope through their `about` edges via `coord::in_purview`, same as items.
3. **No cap**: any count ≥ 1 prints. The real graph currently carries roughly twenty such claims (standing pre-dc-drr6 mints stay asserted until assayed), so design wakes will read a double-digit count immediately.

## Reflections

The count will be loud from day one — arguably the point (the user noticed the set was invisible), but a number that sits at ~20 for weeks risks becoming the dismissable wallpaper th-yzmj autopsied; if it wallpapers, the retroactive assay pass dc-drr6 holds open on ask is the drain, not a threshold on this line. Second: the orient and resume blocks are hand-copied siblings (the house shape for these two surfaces); a render-once seam for wake pressure lines exists if a third count ever joins. Third: a `WakeShape` field was considered and the lighter same-gate read chosen — if design later wants assay pressure on some non-design kind, that's one arm in `wake_shape`, not a refactor.

---
Harvested by the dispatcher 2026-08-17: observed-vs-leased exact (2 files), suite re-run and read raw (96+1 passed, 0 failed). Both outcomes judged met — the boundary held by construction through the weight filter, which is the strongest form the acceptance line could take. The ~20-count wallpaper risk is real but pre-drained by design: the retroactive assay pass stands open on ask. Landed done; the landing ratified cl-2zna.
