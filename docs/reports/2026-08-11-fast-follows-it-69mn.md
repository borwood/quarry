# Dispatch report — it-69mn "fast follows: status labels, mentions section, boundary-verb guard, wrap view regen, q unlink"

Authored by the dispatched build agent (claude-fable-5 subagent), 2026-08-11.
Registered at harvest by the dispatcher. Dispatcher addendum at the end.

## Outcomes against the RETURN spec

**1. Unpack labels carry live status everywhere bodies render — MET, test-covered.** `label()` in src/mention.rs:77 renders status always: live targets as `id [type status: title]` (space form), dead statuses keeping their comma emphasis (`[claim, refuted: ...]`), archived riding along (`[item done, archived: ...]`). One authority serves open, brief, and view. Live: `q open dc-wcyc` renders `dc-jsvp [decision in-force: the matrix grows builds-on...]`. Four existing label assertions updated plus the archived-with-status case.

**2. Derived mentions-outbound section, both surfaces, both labeled — MET.** New `mention::mentions_out()` (src/mention.rs:138) mirrors `mentioned_by` — resolved targets only, citation order, self excluded, nothing stored. `q open` renders "mentions → (derived from this body's citations — a mention references; an edge leans)" (src/render.rs:239) with archived-hidden counts; the inbound label gained the ← arrow for parallelism. The view inverts the already-embedded mention index — no new data in the blob (src/view.rs:555). Live on dc-jsvp: both sections render. Test: `open_and_view_show_outbound_mentions`.

**3. Boundary verbs refuse under an active badge — MET, verified live by the agent's own refusal.** `coord::boundary_refusal()` (src/coord.rs:336) reads the badge — env var OR machine-local .dispatch.json — and wires into wrap (src/main.rs:1411), session resume (:1040), session retire (:1168), before any cursor moves. The agent ran q wrap once under its badge and its own guard refused it, exit 1, no cursor consumed — transcript in the report body it returned; the refusal names the incident class and exits at q harvest. Tests: `boundary_verbs_refuse_under_active_badge` (state channel, clearing reopens) and `wrap_refuses_badged_then_regenerates_view_when_clear` (real binary via CARGO_BIN_EXE_q; env refusal for all three verbs; state refusal without env).

**4. Wrap regenerates the view at close — MET; harvest got it free.** End of the wrap arm (src/main.rs:1712), best-effort with a printed warning on failure so the lint always lands; harvest regenerates too (src/main.rs:1319). Binary test: a refused wrap regenerates nothing; an unbadged wrap prints the regen line and the page exists after.

**5. `q-unlink` lands — MET, verified live on the target mislink.** `ops::unlink()` (src/ops.rs:285): edge leaves the source frontmatter, NO bump on either node (the affirm rationale, stated in code and confirmation), log event carries op/rel/to/at/actor/note with session and badge auto-stamped. Nonexistent-edge retirement refuses and lists what the source carries; a dangling dst (deleted target) is still retirable via the literal id. Live: `q unlink dc-wcyc builds-on dc-wwnk --note "mislink minted 2026-08-11..."` → retired, no bumps, log shows link then unlink at the same v with the note. Test: `unlink_retires_edge_logged_without_bump`.

**6. cargo test — MET.** Raw, unfiltered: `test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.32s`. 48 prior + 4 new. One build at a time throughout.

## Notable calls
- Dead-status emphasis kept as the comma form vs the live space form — dead reads louder without a second mechanism.
- View outbound derives client-side by inverting the embedded mention index — one authority, no blob growth.
- The dispatcher's own boundary verbs also refuse while a machine-local badge is live (state exists whenever a dispatch is unharvested); the refusal's exit — q harvest — is the dispatcher's correct next act either way.
- teach.rs gained one guide sentence on unlink (judgment layer, no flags); q init --claude re-run.
- The agent's graph writes were the unlink on dc-wcyc and log events only; it left the dispatcher's pre-existing uncommitted graph work and the two behind markers on it-ymsj/it-n3fu untouched.

## Reflections (the agent's own words)
- The q wrap refusal firing on me was a strange and satisfying loop — building the fence, then walking into it as proof. Worth noting: had my guard had a bug that let wrap through, the verification step itself would have re-run the incident. A safer protocol for future guard-building dispatches: verify in a temp store first (I did — the binary test), and only then fire the live check. That ordering saved me; it was luck of habit, not the brief's instruction.
- The env-channel race in tests pushed me to test env-badge behavior only via child processes and state-file behavior in-process — the suite runs threaded and temp_store() strips QUARRY_DISPATCH globally, so any in-process set_var would bleed across tests. That asymmetry is invisible in the test names; a future contributor could reintroduce the race easily.
- Friction: the harvest view-regen line prints after the landing homework and reads slightly orphaned. Cosmetic.
- Doubt: boundary_refusal blocks session retire even when the badge belongs to an unrelated arc and the retiree is some third ephemeral session. That's the spec as written and I think it's right — one badge means the machine is mid-dispatch and boundaries wait — but a dispatcher juggling multiple sessions may one day find it overbroad. Watch-list, not fix.
- The mentions → section makes long decision bodies render a tidy outbound bibliography — it surfaced immediately how much of the graph's connective tissue lives in prose. The upgrade-to-real-edge judgment call now has a much better surface to fire from.

## Dispatcher addendum (at harvest, 2026-08-11)

Independent verification: `test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` in the bound environment. The boundary guard verified live from the dispatcher's own chair — my q wrap refused with the badge active, naming it-69mn and pointing at harvest, exactly as designed; the unlink trail on dc-wcyc confirmed (link then unlink at the same v, note carried, dc-wwnk untouched). Observation 8/8 touched files. All six outcomes hold; landed, with it-ymsj and it-n3fu landed by the same arc (batch lease, noted in their logs). The agent's guard-building safety observation (temp-store verification BEFORE the live check) is written into this report as protocol for future guard dispatches; its session-retire overbreadth doubt stays in this report as a held observation per dc-wcyc's residue clause.
