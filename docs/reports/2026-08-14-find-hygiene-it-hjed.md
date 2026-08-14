# Dispatch report — it-hjed: find match hygiene

Relay note: this dispatch's agent finished under a branched session (the /branch took ownership of the running subagents); its report was relayed verbatim through the shared scratchpad and harvested here, where the chat identity, badge, and lease lived throughout. Graph accounting was unaffected — the badge stamped normally.

## Against the RETURN spec — both outcomes land

1. **Tiered output** — `q find cli` against the real graph leads with the area "cli" and title-word hits, then body word-boundary hits carrying `(matched in body)`, and the substring-only debris trails labeled `(loose: substring only)` — including the motivating confabulation case itself: dc-qhru (matched via "click") rides the loose tail, shown but named for what it is. No flag; the tail always prints. Covered by `find_tiers_word_hits_first_loose_tail_last` in tests/basic.rs (title-word, body-word, loose, non-match exclusion, id-substring-stays-tier-one). Re-verified live at harvest by the dispatcher.
2. **Stated divergence** — both predicates sit adjacent in src/queries.rs, each doc comment stating its own rule and naming the other: `find_word` (ASCII alphanumerics only; hyphen is a boundary — "cli" hits "cli-area", never "click") and `contains_word` (hyphen compounds stay whole — load-bearing for the naming register), both citing the declined unification (2026-08-12).

**Mechanics:** tiering lives in `queries::find_hits` (testable; strong/body/loose in graph order); the `Cmd::Find` arm only renders; every line rides `atom_line`, tier labels are suffixes in the existing register. Spine claim cl-ybth (`find-tiers`) minted under the badge, sourced `file:src/queries.rs`; `cl-89b5 -[supports]-> cl-ybth` records the lineage (kill find-areas' atom_line road and the labeled tail goes with it). The running q serve was stopped for the single cargo build and relaunched detached.

## Provisional calls — dispatcher's judgment at harvest

1. **Label wording `(loose: substring only)`** was the agent's call — accepted; register and placement were the settled parts, the text reads plainly.
2. **Title-loose/body-word nodes land tier one with the matched-in-body label** — accepted; the label states honestly that the word hit is body-only.
3. **Lineage rode `supports` (spine → dependent) because builds-on claim→claim is illegal in the matrix** — accepted per dc-grrb; if claim-level builds-on lineage is ever wanted, the matrix itself is the fork, and that is a thread, not an improvisation.
4. **cl-ybth landed kindless** where dc-yd9s expects a material species on claims — fixed at landing by the dispatcher: kind set to vein.

## Reflections (agent's own, kept)

The settled design was a pleasure to build from — THE WORK paragraph carried predicate, tiers, label placement, and the declined unification, so the only real decisions left were wording. Two frictions: the brief gave no code geography (which file owns find, where contains_word lives — minutes of re-derivation the dispatcher had already done), and the read-first shelf is ~80% dispatch-machinery history irrelevant to a matching bug — th-uacu already names this. One irony: the relatedness touches on the claim mint suggested links because it "mentions 'every'" and "mentions 'never'" — the lexicon join has its own hygiene problems; the it-nuw5/it-sc2u/it-b5tq family looks well-aimed. One doubt: `find_hits` lowercases every title and body per query — fine at this graph's size; a large graph will feel it.

## Harvest verification (dispatcher)

Suite re-run at harvest, raw lines: `66 passed; 0 failed` (basic), `1 passed; 0 failed` (surface lint), all other harnesses `0 failed`. Live tier check on `q find cli` confirmed word hits lead and the loose tail labels. Observed-vs-leased exact: three files, all inside the lease — the observed set that missed a tests write on the prior dispatch (it-bj3b) attributed correctly this time.
