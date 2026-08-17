# DISPATCH REPORT — it-nmzn "sediment and rot" (agent:a769b405185262231)

File footprint: src/** + tests/** (in lease) plus q-verb graph writes; intent-delta shows `reading-kind` joined (absent from both unlanded and unintended); all suites green; every ratified line verified verbatim against live output.

## Against the RETURN spec

**`reading-kind` lands: reading and measured split at mint with the species prompt; micro-pass re-kinded the four standing measured claims.**

- `q claim --kind` carries any species string at mint (engine stays kindless-but-kind-aware, dc-yd9s): `ops::claim` gained the `kind` param (src/ops.rs), `Cmd::Claim` the flag (src/main.rs). A method-carrying claim minted kindless draws the ratified species prompt in `print_mint_surfaces` — one point covering `q claim`, `q new`, and `q rule` mints — followed by a settle hint (`q set <id> kind=reading (or kind=measured)`). Prompt, never a gate; verified live in a scratch store, verbatim.
- **Micro-pass** (graph acts, badge-stamped): cl-eb8j re-kinded `reading` with a note carrying the judgment — and archive-on-consumption fired on that very act, archiving it on the spot (its consumers dc-kpqg and dc-6gn9 are both in force). cl-c34g, cl-en99, cl-qm6t judged and confirmed `measured` — their instrument (integration tests, tests/basic.rs) re-ran green as part of this build (`test result: ok. 84 passed; 0 failed`), and each was affirmed under the ratified manual line. No no-op kind writes on the three (a re-set would bump v and cascade behind for nothing).

## The rest of the work body

- **Behind classifier**: `Behind` carries `sediment: bool`, classified once in `queries::behind` (src/queries.rs) — ordinary drift (sev 3/4) under a reading is sediment; refuted/superseded/dangling targets and measured drift are never sediment. Six surfaces render the split: `q query behind` (wants-action header + ratified sediment line + strata reach-line), the SessionStart wake count (`behind: N · sediment: M`), `q wrap`, dispatch-wake homework residue (src/render.rs), mint homework (`print_homework` collapses reading citers, dead targets stay loud), and the view's Behind panel (src/view.rs). The brief's dispatcher behind-check needs no change — its src is always the item.
- **Archive-on-consumption**: `ops::consume_readings` sweeps readings whose last live consumer settled, using blast's leaning vocabulary (inbound depends-on/source/builds-on + the reading's own supports targets; inbound supports is evidence, not a lean). Settledness by type: item done/dropped, thread resolved, decision in-force/superseded, claim refuted/superseded, doc always; consumerless readings never sweep. Called from `set` (status/kind flips), `archive`, `rule`, `refute`, and `link` when a reading's lean is declared late. Each sweep prints what it hid. `q archive` now admits readings at any ladder status (they settled when they landed); every other species keeps the settled-status gate. The brief backdrop now counts archived claims it hides (area-open already did).
- **Species-affirm**: `queries::affirm_teaching` resolves the ratified line at one point — file:tests source → instrument line (re-read); other measured → manual line (re-run); readings → the READINGS first sentence. Printed by `q affirm` before the restamp count. Fired live three times (manual) plus the reading teaching in smoke; instrument path unit-tested.
- **Verbatim verbiage**: all four ratified lines ship in src/framings.rs exactly as the item records them (spaced hyphens preserved).

## Verification

84 passed / 0 failed integration (7 new tests: kind plumbing, sediment-vs-rot, dead-source-rots-under-reading, species affirm teaching, last-consumer sweep + idempotence, in-force-decision consumption + measured-stays-live, species-not-ladder archive gate), surface lint green, doc-tests green — read from the raw `test result:` lines. Release binary rebuilt; every print surface exercised live (real graph or scratch store).

## Graph writes (YOUR-WRITES)

Veins: cl-34ra `sediment-classifier`, cl-aruk `consume-readings`, cl-rr2j `affirm-teaching`; receipt: cl-9z95 `reading-kind` (satisfies the acceptance name — intent-delta shows it joined). Each vein supports the receipt; the receipt supports dc-6gn9.

## Flags for the dispatcher

- **Mechanical register composed at build** (th-4w3a's ratify-or-amend channel): the wants-action header, the strata reach-line ("the strata: … — q open <id> reads one at its date"), the settle-it hint under the species prompt, the wake `· sediment: N` suffix, the sweep line ("⚑ reading archived on consumption — …"), and the backdrop hidden-claims count. The four ratified lines themselves are verbatim; the view's sediment note duplicates the ratified sentence as a JS literal (the view's house pattern).
- **dc-8z38 (archive cooling) adjacency**: consumption archives immediately at the settling act, while cooling holds settled *work* for a later wrap. dc-6gn9 read as the explicit, later, species-specific ruling — no conflict taken, named here rather than queued.
- **Open seam, deliberately not taken**: the three standing measured claims still source their mechanism files, so they classify *manual* at affirm until someone re-sources them to their instrument (dc-6gn9's "sourced to the test blob" design for future mints). Re-sourcing felt like judgment beyond the micro-pass's re-kind mandate.
- The 55 wants-action entries in the real graph are pre-existing drift (largely from this build's own src edits) — the dispatcher's to confront at harvest/wrap as usual.

## Reflections

The live firing of the sweep at cl-eb8j's own re-kind was the design proving itself unprompted — the "only one end exists" principle made the micro-pass self-completing, which was not planned. Friction: the consumer definition wanted a ruling that had to be derived — "settles" for a decision had to mean *in-force*, which reads odd until you see live_blockers already says it; and inbound-supports-as-evidence vs outbound-supports-as-lean took the edge matrix to untangle — the doc comment on `consume_readings` carries that reasoning so the next builder doesn't re-derive it. Doubt worth voicing: the sediment count includes archived readings' drift forever (behind deliberately sees archived nodes), so the count creeps monotonically — geologically honest, but if it ever reads as noise, the fix belongs in the classifier, not the collapse. The link-time sweep was found by smoke-testing, not design — the scratch-store pass earned its keep.

---
Harvested by the dispatcher 2026-08-17: observed-vs-leased clean (7 files, all in-lease), suite re-run and read raw (84 passed, 0 failed), the four ratified lines verified verbatim in src/framings.rs against the item's record. The composed mechanical strings join th-4w3a's ratify-or-amend channel; the re-sourcing seam filed as debt. Landed done.
