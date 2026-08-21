---
id: cl-stkv
type: claim
title: '`user-owned-extent`: the declaration section runs to the next heading, so a lead-in is prose and prose that is not "none" is never a zero'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T11:06:57Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/queries.rs
  at: 53fa6ae103db
- rel: supports
  to: it-drsu
  at: 6
- rel: source
  to: file:src/render.rs
  at: 3d795e6d98e7
---

`user-owned-extent`: the declaration section runs to the next heading, so a lead-in is prose and prose that is not "none" is never a zero

The section-extent floor under the parse half of cl-cjbb (it-drsu). queries::declared_user_owned_calls ended its walk at the first non-blank, unindented line that was neither a list entry nor the bare word none — which is exactly what a lead-in sentence before a list is. do-yeum's section read "## user-owned calls", blank, "**One.**", blank, "1. **The ratified user_owned_await wording now carries a second meaning**": the loop broke on "**One.**" with n == 0 and returned Some(0), so the it-p8rp harvest printed "report declares 0, thread(s) filed under this badge: 1" against a report that plainly declared one call and named the thread it filed. The agent was honest; the parser misread it. Two of six agents wrote that shape on one day with no contact between them — the it-2eqk agent caught its own draft only by building a throwaway parse instrument — so the shape is what a careful writer reaches for, not an outlier.

THE WALK NOW ENDS WHERE A SECTION ENDS, not at its first sentence: a markdown heading (unindented, always), a fresh unindented paragraph once entries or a lead-in have already been read (an un-headed report's next section), or the end of the report. A blank line closes a paragraph and never the section, which also retires a second under-count the old blank-after-entries break carried: a loose list, blank lines between its entries, now keeps its full count instead of stopping at the first.

THE COUNT IS THE ENTRIES, AND SILENCE IS FORBIDDEN IN THE ZERO DIRECTION. With no entries the section still speaks: prose opening with the whole word none declares zero — "none.", "none declared by the agent (...)", "**none.** One near-miss checked rather than assumed" are all real corpus shapes, and "nonetheless" is deliberately not one — while any other prose is one declaration, the rule the head line's own tail already used. The asymmetry is the repair. An under-count to zero renders framings::user_owned_reconcile's "reconciled" over a call the report declared, and the dangerous case is declared 0 against filed 0: a false all-clear on the one accounting no hook can see. An over-count only asks the judge a question, which is the posture that line already takes.

THE CONTRACT NOW STATES THE SHAPE IT IS READ IN. render::brief's RETURN spec carries one line under framings::USER_OWNED_SLOT — one entry per call under that head, a lead-in sentence before the list is fine, indented continuations belong to their entry, the section ends at your next heading. Nothing said this before: the requirement was discoverable only by reading this function or building an instrument, which is precisely what the it-2eqk agent had to do. The line is composed in render.rs beside the brief's other structural lines rather than in framings.rs, which is outside this arc's lease and whose ratified slot is the user's pen under dc-vzvf; it rides that flag channel for ratify-or-amend.

MEASURED ON THE CORPUS, not reasoned. Across all 48 registered reports in docs/reports — 11 carrying the section, 37 predating it — the pre-fix and post-fix readings differ on exactly one file: 2026-08-21-arc-bounded-return-it-p8rp.md, the incident itself, Some(0) becoming Some(1). Nothing else in the corpus moves.

Pinned by a_lead_in_before_the_list_never_reads_as_a_declaration_of_none in tests/basic.rs — the witnessed shape verbatim in shape, the heading bound with a reflections list below it, a loose list, both real none-prose shapes, prose that is not none, "nonetheless", an empty section, and an un-headed report whose next section must not be swallowed — and by an added arm of return_spec_and_first_echo_carry_the_user_owned_calls_rule for the contract line. Each half probed against the regression it guards: restoring the break-on-any-prose reproduces the incident reading exactly (Some(0) where Some(1) is right), removing the heading bound lets an empty section count the next section's list, and reading prose-only sections as zero fails the never-a-silent-zero assert.

Residue, both in the safe direction and both named honestly: a lead-in of two or more paragraphs before the first entry still ends the walk at the second paragraph, deliberately, so that an un-headed report cannot swallow its next section's list — such a section reads as one declaration rather than its true count, a gap the judge is asked about and never a false zero. And an agent still cannot see what the machine read of its own report; no verb prints the parse, and building that surface needs main.rs, outside this lease.
