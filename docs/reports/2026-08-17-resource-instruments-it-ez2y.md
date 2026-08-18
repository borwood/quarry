# DISPATCH REPORT — it-ez2y "re-source the standing measured claims to their instruments"

**Against the return spec — "lands nothing new: the three claims source their instrument blobs, species-affirm classifies them instrumented, old mechanism-file sources retire via q unlink":**

- **Nothing new landed.** No nodes minted, no code touched. Six q acts total: three links, three unlinks.
- **All three claims now source `file:tests/basic.rs`** at current blob 67dde7205950, each verified live via `q open`: cl-c34g (v5), cl-en99 (v5), cl-qm6t (v6). The `method: integration tests (tests/basic.rs)` field and the source edge now agree.
- **Species-affirm classifies all three instrumented.** `queries::affirm_teaching` keys AFFIRM_INSTRUMENT on a measured claim carrying a `file:` source with a `tests` path component — all three now satisfy it; `affirm_teaching_is_species_shaped` pins the behavior. No live demonstration is possible until drift occurs (the teaching only prints on an actual restamp).
- **Old mechanism-file edges retired via q unlink**, each with a logged why, no version bump: cl-c34g ⇸ src/coord.rs, cl-en99 ⇸ src/queries.rs, cl-qm6t ⇸ src/teach.rs.

**The judgment the dispatch left untaken — each instrument verified by reading the test against the assertion:**

- **cl-c34g** (area gate + watermarks): `area_watermark_lifecycle` exercises FirstTouch, cursor recording, foreign-drift surfacing including the same-second log-index case, and cursor advance — the whole watermark mechanism.
- **cl-en99** (mint-time relatedness): `relatedness_forward_and_reverse` is the assertion verbatim — forward surfacing, reverse-at-nodehood, never-self, silence default. Its old src/queries.rs edge was flagged drifted; the drift was mechanism churn, not instrument change, and the unlink note says so.
- **cl-qm6t** (dispatch chain gates): `c8_briefed_gate_and_lease_check` covers both halves — `briefed_this_session` and the full `lease_check` matrix including join-gate and out-of-repo semantics.

**Sanity:** instrument re-run green after the swaps — raw line read directly: `test result: ok. 96 passed; 0 failed`. `q query behind` shows none of the swap edges; the 65 standing wants-action entries are pre-existing drift belonging to it-4p7m's source-drift pass.

**No vein owed:** the repair extracted no mechanism — pure edge swaps. **No affirm act recorded:** the link acts stamped current blobs; the re-read that affirm-on-instrument means happened as the verification above, and the link/unlink notes carry it in the log.

## Reflections

- **The wiring residual.** Each instrument tests the *mechanism* (touch_area, relatedness, lease_check/briefed_this_session), while the claims assert *delivery* ("at write time", "at mint", "gates reserve"). The delivery wiring lives in main.rs/hooks and is not directly exercised — if a wiring call were deleted, the instrument stays green while the claim's delivery half could go false. tests/basic.rs judged the honest instrument regardless, but the gap is real and feels like exactly what the contract species at symbol grain will eventually own.
- **Re-sourcing bumps.** `q link` bumped each claim a version (edge addition is content). Ratify and affirm deliberately avoid bumps so citers never go behind over bookkeeping — a grounding *swap* on an unchanged assertion sits arguably closer to bookkeeping than content. Harmless here, but any future bulk re-sourcing might want a ruling on whether source-edge swaps deserve the no-bump treatment.
- **No live surface shows the instrumented classification while edges are current** — the teaching only prints on restamp; verification went through the classifier predicate and its pinning test instead.

---
Harvested by the dispatcher 2026-08-17: seven badge events, all q verbs, zero file writes — the leased `graph/nodes/claim/**` was dead weight by construction, since q-mediated graph writes never consult the file lease; noted as a small gap (dispatch demands --files even for graph-only work). Each instrument verified against its claim in the report and spot-checked live (cl-qm6t opened: source fresh, method agrees, mechanism edge gone). The re-sourcing-bumps question rides the flag channel. Landed done; no mints to ratify (zero-mint arc ratifies nothing, per the assay office's own rule).
