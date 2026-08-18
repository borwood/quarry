# DISPATCH REPORT — it-33bb: the acceptance gate

Sanity checks at close: full suite re-run reading the raw result lines — `test result: ok. 92 passed; 0 failed` (basic) and `1 passed` (surface_lint); release binary rebuilt and all three stations verified live against the real graph (flip refusal, brief tripwire, derived query); the ready sweep re-checked against the set as it stood at landing, including an item that went ready mid-work. The awaiting filter shows only sketches — no ready/in-flight/shaped breaches anywhere.

## Against the RETURN spec, outcome by outcome

**1. ready-gate — landed.** There is no literal `q ready` verb; the ready flip is `q set <item> status=ready`, and the gate lives there (`src/ops.rs::set`), checked after every field of the act lands so `status=ready acceptance+="..."` passes in either order within one act. The mint path (`ops::new_node`, `q new --status ready`) holds the same invariant, so no construction path reaches ready acceptance-less. Both refusals face the shaper and teach the authoring command (`acceptance+=` / `--acceptance`); demotions stay free. Verified live: the flip refused on it-njn9 with the teaching line, mutating nothing.

**2. reserve-backstop — landed.** `ops::acceptance_backstop` is the one fire-time check, called from `q dispatch` ahead of ownership/brief/lease logic (this placement also covers the re-dispatch path, which never touches reserve) and from the `q reserve` handler ahead of C8. It hard-refuses and loudly un-readies ready (and in-flight) items to shaped via a logged `q set` whose note names the gate; nothing else mutates — no brief event, no lease, no in-flight flip. Teaching is station-shaped: the dispatcher gets return-to-design and never the authoring command; the solo path gets `acceptance+=` directly. No inline authoring flags anywhere. Pinned by test including the un-ready, the log note, and the never-teach-the-dispatcher assertion.

**3. brief-tripwire — landed.** `render::brief` bails before rendering a line when acceptance is empty, naming the breached invariant (dc-p6z4); the old "no acceptance recorded — fix the graph: acceptance+=" self-author branch is deleted. The refusal deliberately contains no authoring prompt — the reader may be the joined agent. Both callers inherit it (`q brief`, `q join`), and a refused `q brief` logs no brief event, so C8 cannot be satisfied through a breach. Verified live and by test.

**4. awaiting-acceptance — landed.** `queries::awaiting_acceptance` derives at read, nothing stored: live items with no acceptance lines, any status, breaches (ready/in-flight) sorting first. Surfaced as `q query awaiting-acceptance` (alias `awaiting`) with the authoring teach. Both design-shaped wake surfaces (`q session resume` and the SessionStart orient) count the shaped-and-acceptance-less stratum beside owed threads; sketches stay quiet, and the dispatch-kind wake omits it. The wake count is pinned by a binary-driven test covering design shape shows / dispatch shape omits / authoring clears.

**5. ready-sweep — performed, zero demotions.** The pass ran over the current ready set at landing: the set held exactly one item (it-6349, flipped ready by the quarry session mid-work) and it carries acceptance lines, so nothing demoted and nothing needed naming. No standing sweep machinery was built — per the ruling, the four stations plus the derived filter make the swept state impossible by construction, and the awaiting query confirms zero live breaches.

## Tests

5 new tests (`ready_gate_refuses_acceptance_less_flip_and_mint`, `reserve_backstop_refuses_at_fire_and_unreadies`, `brief_tripwire_refuses_and_names_the_breach`, `awaiting_acceptance_derives_at_any_status_and_clears_by_authoring`, `design_wake_counts_shaped_and_acceptance_less`); 14 existing fixtures updated to state acceptance (they minted or briefed acceptance-less items — the gate caught them, which is the gate working). Suite: 92 + 1 passed, 0 failed, read raw.

## Graph writes

Veins cl-psau (`ready-gate`), cl-t7nx (`reserve-backstop`), cl-p7mv (`brief-tripwire`), cl-2sk9 (`awaiting-acceptance`); feature receipt cl-88ma (`acceptance-gate`); four supports edges vein→receipt. Files: src/ops.rs, src/render.rs, src/queries.rs, src/main.rs, tests/basic.rs.

## Provisional calls flagged (C3)

(a) "q ready" in the spec read as the `status=ready` flip since no such verb exists, and the same gate extended to mint-at-ready — the invariant is by-construction, so leaving the mint hole would have made the tripwire reachable. (b) Composed mechanical strings await ratify-or-amend per th-4w3a's pattern: the three refusal texts, the mint refusal, the wake-count line, and the awaiting query's footer — all flagged here, none silent. (c) `awaiting-acceptance` excludes done/dropped/archived on the reading that settled items await nothing; "queryable at any status" is honored across all live statuses.

## Reflections

The one genuine design friction: "q reserve refuses on both paths" reads as if the check belongs in `coord::reserve`, but that placement would miss the re-dispatch path (lease kept, reserve never called) — the check had to sit at the two command stations instead; the harvester should confirm that reading matches the ruling's intent. The solo-station ordering surprised: the backstop must run before C8, or the shaper gets told to render a brief the tripwire will refuse — a small trap the spec never mentions but the mechanism forces. The mid-work arrival of it-6349 going ready was live proof that the sweep spec's "current ready set" is a moving target; the check re-ran at the end rather than trusting the survey from an hour earlier. Doubt worth recording: the tripwire message names the breach but teaches no recovery path at all — deliberate per the ruling, but a solo human hitting it via `q brief` on their own sketch gets a colder wall than any other station; if that stings in practice, the fix is a ruling amendment, not a quiet soften.

---
Harvested by the dispatcher 2026-08-17: observed-vs-leased clean (5 files, all in-lease; the tests write observed correctly this time), suite re-run and read raw (92+1 passed, 0 failed). Harvester's confirmation on the flagged reading: the backstop at the two command stations honors dc-p6z4's intent — the wall is "every fire path passes the gate," and coord::reserve placement would have left re-dispatch ungated; the letter said reserve, the mechanism demanded the stations, the invariant is what the ruling protects. Composed strings joined th-4w3a. Landed done; the landing ratified the arc's five mints.
