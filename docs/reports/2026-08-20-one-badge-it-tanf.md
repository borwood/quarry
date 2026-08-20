# Dispatch report — it-tanf: one badge per identity

Tenth worktree dispatch (serial run). Agent report condensed faithfully;
dispatcher's landing addendum follows.

## Outcomes against the RETURN SPEC (one acceptance line, three clauses — MET)

- **q join refuses a second live badge, naming the live badge and the roads out**:
  `coord::consume_join_token` checks the identity's standing live badge (new
  `coord::live_acting_badge`, honored only while its dispatch is held) before
  anything is spent. The refusal names the live badge by title and id, the
  mechanism prevented, and three roads: report-and-stop (harvest frees the
  identity), separate dispatches, the bundle shape (th-zzqv) as not-yet-built.
  Nothing is spent on refusal — no bind, no join event, no pin, no area-read
  delivery; the token stays live for the right agent.
- **Re-join of the same badge stays an idempotent read**, with one construction:
  a re-join restores a lost acting row (the pin-restore pattern). The one-badge
  check runs on the re-join arm too, so a pre-fix legacy blur refuses rather
  than re-rendering a brief for an item the identity no longer stamps.
- **The acting map holds one badge per identity by construction**: private
  `coord::bind_acting` is the sole writer; both insert sites route through it.
  It never overwrites a live binding — the join road refuses loudly, the env
  road declines silently (per-shell QUARRY_DISPATCH still stamps env-first where
  the override is deliberate). A row pointing at a cleared dispatch is residue,
  never a binding — harvest frees the identity by construction, pinned.
- **The attention-key caveat (carried from the it-csm3 report) — checked, rides
  along with zero extra machinery**: `attention_key` resolves through the same
  acting map, so a badge-scoped reader cannot inherit a last-join blur once the
  map cannot re-point. Pinned: a refused join delivers no area record.

## Test results (verbatim, read raw)

```
test result: ok. 128 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.60s   (tests\basic.rs)
test result: ok. 2 passed; 0 failed (broken_pipe) · 1 passed (observed_set) · 1 passed (store_lint) · 1 passed (surface_lint) · 2 passed (worktree_proof) — all 0 failed
```

New instruments: `join_refuses_a_second_live_badge_naming_the_roads_out` and
`acting_map_holds_one_badge_per_identity_by_construction`.

## Mechanics

- Branch `worktree-agent-a7f7bcc5c46a61d02`, commit `0bc07ac` ("One badge per
  identity: join refuses the second, the acting map never re-points"), 4 files,
  +237/−19. Fast-forwarded at landing. teach.rs untouched (dc-r8de puts the
  constraint teaching in the refusal and `q join --help`) — no regeneration owed.
- Graph acts under the badge: vein cl-kggw (`one-badge-per-identity`, source
  file:src/coord.rs); affirms cl-6ctr (extended by the refusal) and cl-b2z2
  (caveat verified); sibling defect filed on sight as it-jsu5 (stale acting row
  survives a same-chat re-dispatch — the replaced agent keeps stamping, and
  post-fix is also locked out of its next join until harvest).

## user-owned calls:

none.

## Agent reflections (verbatim highlights)

- Composed register flagged per dc-vzvf: the refusal text, the `q join --help`
  addition, the module-header sentence.
- Interpretive call worth eyes: "same badge" read as the identity's live acting
  badge, so a legacy blur residue refuses rather than re-joins — re-rendering
  Y's brief while every act stamps X would recreate the exact misreport being
  fixed. Post-fix the state cannot arise.
- The consequence is real: an agent that finishes early cannot pick up a second
  item — report-and-stop is the only road until the bundle shape lands. it-jsu5
  makes it slightly worse (a replaced agent is locked out through no act of its
  own), hence filed rather than noted.
- Doubt, minor: the env road's silent decline — a shell wearing QUARRY_DISPATCH
  over a different live badge gets env-first stamping with no note that the map
  declined to learn. Judged right for the deliberate-override case; the decline
  point is one function if design disagrees.

## Dispatcher's landing addendum

Tenth consecutive worktree arc, fast-forward again, dc-g5x5 clean again. The
harvest surfaced a genuine misjoin in the fresh user-owned machinery: the
reconcile parsed the *carried* it-csm3 report (supports-linked into this item's
brief for its caveat) as if it were this arc's return, and flagged its missing
section falsely. Filed on sight as it-p8rp (ready): a report predating the
arc's dispatch is never its return. Registration of this report clears the
false flag by newest-wins — the next harvest line should read the declared
"none" and reconcile at 0/0.
