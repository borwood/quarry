# DISPATCH REPORT — it-6349: ready teaches the leans

Release rebuilt clean. Sanity check at close: the three RETURN outcomes re-read against what landed — all three verified by test (raw line: `95 passed; 0 failed`) and eyeballed live on a scratch store.

## Against the RETURN spec, by outcome

**`ready-lean-prompt` — landed.** Both construction paths to ready now enumerate: the `q set status=ready` flip (Cmd::Set handler) and mint-to-ready (`do_new`), via `print_lean_prompt` in src/main.rs. The derivation is `queries::unleaned_citations` (src/queries.rs): body-cited in-force decisions and live claims (never archived/refuted/superseded) with no edge between them and the item in either direction, each paired with its ready-made command — `q link <item> depends-on <dc>` for rulings; `q link <cl> supports <item>` for claims, because item→claim has no legal forward rel: the lean runs claim→item, which is exactly how `weight_held` counts load. Never a gate — the prompt prints after the flip stands, and stays silent when nothing un-edged is cited. Verified end-to-end through the built binary (test `lean_prompt_fires_at_ready_teaches_why_and_closes_open`) and live on a scratch store.

**`lean-why` — landed.** The verbiage ships in src/framings.rs (LEAN_HEADER / LEAN_WHY / LEAN_CLOSE): the header carries "a mention references, an edge leans"; the why carries "leaned nodes pin the dispatch brief's READ-FIRST, so the working agent reads them before building" plus "mention-only is often correct — the judgment is yours"; the close asks "what else does the item stand on that no line surfaced?" naming why — the enumeration sees only body citations. Pinned by the same test.

**`link-refusal-teaches` — landed.** src/model.rs: the matrix match extracted into `edge_ok` (single authority), `legal_rels(src_ty, dst_ty)` derives from it, so refusal and teaching can never disagree. A refusal now reads: the attempted rel's shapes (unchanged), then the redirect — `legal rels for item → decision: depends-on` (the exact it-33bb near-miss, now a one-step redirect); when the forward set is empty but the reverse isn't: `the lean runs the other way: claim -[supports]-> item`; when nothing joins either way: the mention channel, taught as correct rather than a downgrade. Test `link_refusal_names_legal_rels_for_the_pair` covers all three branches; the pre-existing refusal assertions still pass untouched.

## Method

Full suite `test result: ok. 95 passed; 0 failed` (read raw, three tests added), surface lint 1 passed; release binary rebuilt after all q verbs, so the surfaces are live for harvest.

## Graph writes under the badge

Veins cl-gd6c (`lean-prompt`, source file:src/queries.rs) and cl-btnr (`legal-rels`, source file:src/model.rs); feature receipt cl-5ggz (`ready-teaches-leans`, source file:src/main.rs); both veins linked supports→cl-5ggz.

## Provisional calls, flagged

- The prompt fires only when the enumeration is non-empty — the open-ended close never rides an empty list. Read from dc-grrb presence-prompt doctrine plus the th-yzmj dismissed-warning autopsy as forbidding an unconditional every-flip question; the close's own rationale ("reflection reaches edges the recommendation cannot see") could argue it should fire even with nothing enumerated. The shaper's call at harvest.
- The reverse-direction redirect goes slightly past the spec's letter ("names the legal rels for that exact src-to-dst pair") — without it, item→claim (the commonest real lean) would dead-end into the mention teach even though a legal lean exists one inversion away.
- Settled strata never enumerate: recommending a lean on a superseded decision would collide with the deliberate `--acknowledge`; exclusion felt like the least-committal path.
- Composed strings per the standing flag channel (dc-vzvf): LEAN_HEADER / LEAN_WHY / LEAN_CLOSE, composed from the brief's own doctrine sentences, marked for ratification.

## Reflections

The direction inversion for claims was the one genuine surprise — the return spec's "no edge from the item" reads item-outward, but for claims the graph's own load semantics point the edge inward, and following the spec literally would have recommended illegal commands. Design friction: the LEAN_* constants are newly composed prose in a file whose header says ratified-only — composed from the brief's own doctrine sentences and marked "for ratification at its harvest" (the SPECIES_PROMPT precedent), but the file's contract and the mechanism's need to ship verbiage are in mild tension every time this pattern recurs. Small doubt: mint-to-ready firing the prompt duplicates one line of the mint's existing "body cites" echo when both render — visible, not harmful; folding them felt like scope creep.

---
Harvested by the dispatcher 2026-08-17: observed-vs-leased clean (5 files), suite re-run and read raw (95+1 passed, 0 failed). Harvester's judgment on the flagged calls: firing only on a non-empty enumeration is confirmed — an unconditional question at every flip is exactly the dismissed-warning shape the autopsy buried; the reverse-direction redirect honors the ruling's intent (the redirect exists so a real lean is never lost to a dead end, and item→claim is the commonest real lean); settled-strata exclusion stands. The LEAN_* strings ride the flag channel to the design session with run six's. Landed done.
