# Payload review 2 — it-hapc, written after the work and a full-graph read

Companion to payload-review-1 (written blind at join) and payload-q-account
(the honest record: I worked entirely from the payload plus src/). This one is
written knowing (a) what of the payload the work actually consumed and (b) what
the whole graph holds.

---

## B. What the graph holds that the payload did not show

Scale first: the graph is 208 nodes — 6 areas, 46 decisions, 70 items, 32
threads, 33 claims, 21 docs. The payload showed me one area's shelf (cli) plus
four READ-FIRST nodes. What lived beyond it:

- **Four whole areas invisible**: schema, storage, ui, and deepcraft-salvage
  (a staging ground for salvaging a prior project — naming registers, worldgen
  mode seams, a process autopsy doc; roughly a third of the decision inventory
  is salvage/naming material). Correctly hidden for this work — the area gate
  did its job here.
- **The process area register**, only half-visible through mentions:
  dc-kpqg (dispatch citizenship — the ruling on what the BRIEF IS and what a
  dispatched agent may do: "the brief is the floor, not the whole interface —
  mid-work q open / q find / queries stay free... zero obligatory q acts";
  claims yes, threads yes, decisions withheld; it even carries a "BRIEF FIXES
  OWED" list and the dispatcher-burden watchlist). dc-dsdm (one-line philosophy
  at the surface where the choice happens), dc-e6pp (backtick capability
  names — why my claim title looks the way it does), dc-r8de (one authority
  per fact), dc-9hgd (engine vs protocol content), dc-wngq (the dispatcher:
  all-areas, named plainly).
- **Sixteen resolved threads as institutional memory.** th-dbn4 (the dispatched
  agent as quarry citizen — the design conversation behind my own working
  conditions) and especially th-3pq2 (self-dispatch post-mortem): a session
  fused dispatcher and agent, and the audit found the norms that would have
  prevented it "rode the brief only as shelf lines... the continuity that
  carried the norm was exactly what got wiped." The graph already knows that
  shelf lines do not transmit norms — which is this experiment's own thesis.
- **Fourteen watch items**, including it-b2sp (dispatcher context burden —
  the dispatcher-side twin of the brief-dump problem) and it-fkrz (report
  homework nowhere queryable).
- **The dispatch reports are code maps.** Reading do-sdcb (kind-shaped wake)
  after the fact: it has a "Where things live" section naming the exact files
  and functions its work touched — src/coord.rs wake_shape, render.rs
  dispatch_wake, the test names. The very thing review 1 wished for ("where
  the code is") has been sitting in the doc shelf all along, flattened into 15
  indistinguishable title lines.
- **Sketch/watch items adjacent to my work** that never surfaced: it-z2sy
  (solo-lease accrual bug), it-qxjx (rule-flips deliver no read-first),
  it-2zf9 (machine-readable output — review 1 wondered about exit-code
  semantics; this is its filed home).

## C. Second review of the payload

Principle applied: there should be a good reason for everything it shows.

### What earned its place (receipts from the actual work)

- **THE WORK paragraph** — the spec. Consumed line by line; the finished
  surface quotes its phrases. Nothing in it was wasted.
- **dc-crea's body (READ-FIRST)** — the authoritative semantics; the two
  design traps (no ranking among live dispatchers; advisory means stateless)
  are stated there and I would have gotten at least one wrong without it.
- **it-skpa + cl-3rx9** — pointed me at coord::parse_kind and the registry as
  data; my entry point into the code.
- **dc-qyr5's one line about the claim point guarding the race** — quoted
  verbatim in the new surface text.
- **PROTOCOL block** — every line fired: one build at a time, raw test-line
  reading, and the q.exe-is-the-hook warning primed me for the locked binary.
- **dc-f79h (view renders per request)** — the sleeper hit. A shelf line I
  called noise in review 1 is what made killing and restarting the user's
  live `q serve` a safe, confident move. Evidence that backdrop shelves are
  not worthless — but note it earned its place by accident, not selection.
- **ACTOR RULES** — C3 produced the flagged provisional calls; "done is a stop
  signal" shaped the report; extraction-on-citation produced cl-9v7c.
- **The acceptance warning in RETURN** — acted on immediately.
- Of the 27 spine claims: cl-3rx9, cl-br3p, cl-s98g, cl-mk8b, cl-z6gc used as
  trailheads or background; cl-syj7 (atom-lint) actively shaped behavior — I
  reached for aref() instead of formatting a title because the shelf told me
  the lint would fail me. Six of twenty-seven.

### What was noise for this work

- **~21 of 27 spine claims** (mention-index, unpack click-target, q-unlink,
  find-areas, claim-title-override, carrier-replumb, ...). Real material,
  wrong item.
- **14 of 15 docs** as presented — and this is the sharpest lesson: two of
  them (do-2hcu, do-sdcb — the reports of it-skpa and it-wub5, both in my
  dependency neighborhood) contained exactly the code maps I needed, and I
  skipped them because nothing distinguished them from the other thirteen.
  Noise by presentation, not by content.
- **Six open threads at full atom lines**, all marked NOT-yours. One line —
  "6 open threads in cli, none yours: q query queue" — loses nothing.
- **Triple-rendering**: dc-crea's body verbatim twice; dc-ydvb's substance
  three times (its own READ-FIRST, inside it-u8uf's body, inside dc-crea's
  refines note). Dedup alone cuts a third of the payload.
- **it-u8uf's full READ-FIRST body** — archived history whose live content is
  already folded into dc-ydvb and dc-qyr5. One atom line would have served.

### What the graph held that should have been surfaced and wasn't

1. **it-3pab (the launcher item).** Its body ends: "Fire-time offering of the
   launcher rides it-hapc." The graph names MY item as this item's heir — and
   the payload never showed it. I rebuilt the launcher convention from
   main.rs archaeology instead. Root cause is split: it-hapc carries no edge
   to it-3pab (a graph gap), but the mention channel already derives the
   backlink — the brief renders no mentioned-by shelf for the dispatched item.
   The machinery exists (dc-wwnk mention-index); the brief just doesn't call it.
2. **dc-wngq (the dispatcher decision).** Load-bearing for my vacuous-fit
   call (all-areas charter); I got it only as a mention-unpack inside
   cl-br3p's body — a second-hand fragment where a first-hand node existed.
3. **dc-kpqg's one line: "the brief is the floor, not the whole interface —
   mid-work q open / q find / queries stay free."** My q-account shows why
   this matters: the payload read complete, so I never once consulted the
   graph, and both near-misses above (it-3pab, dc-wngq) followed directly.
   This is dc-dsdm's own doctrine — one line of philosophy at the surface
   where the choice happens — applied to the brief itself, and it is missing
   from the brief.
4. **The machine-state shelf.** THE WORK says "dispatcher-session.cmd today";
   the brief derives at join time ON the machine and could have printed the
   actual registry: dispatcher [dispatch] areas/charter/heartbeat, launcher
   script present at repo root. I read graph/sessions.json by hand instead —
   for an item ABOUT session coverage, the sessions are the subject matter.
5. **The reports of read-first items' landings** (do-2hcu at minimum, being
   it-skpa's report — and it-hapc depends-on it-skpa). Their "Where things
   live" sections are the code map review 1 asked for.

### Structural suggestions

1. **Tier the payload**: CONTRACT (work, acceptance, write-set, return,
   protocol) → SEMANTICS (read-first bodies, rendered once) → MAP (for each
   read-first item, its landing report's where-things-live line; the
   dispatched item's mentioned-by backlinks; machine state when the item is
   coordination-flavored) → BACKDROP (area decisions/spines/threads as atom
   lines only, with counts and a reach-them command). Everything in the first
   three tiers has a reason traceable to THIS item; the backdrop declares
   itself backdrop.
2. **Render each node once.** Highest tier wins; later occurrences compress to
   atom_ref + "above". No body appears twice.
3. **Rank the spine shelf** by lexicon overlap with THE WORK + acceptance
   names (the intent-delta join already computes exactly this), full bodies
   for the top few, title lines + count for the rest — th-uacu's candidate,
   which my experience confirms, with the tension against dc-grrb's
   bodies-on-the-shelf delivery resolved by the split: bodies for the ranked
   few, never for all. (The selection principle is the user's call, queued in
   th-uacu — this is evidence for it, not a settlement.)
4. **Surface the dispatched item's mentioned-by backlinks.** Cheap, derived,
   already implemented for open/view — and it would have caught the one true
   miss of this dispatch (it-3pab).
5. **Move the acceptance-empty confrontation to the dispatcher at fire time.**
   The behind-check already confronts the dispatcher at q dispatch; an empty
   acceptance is the same class of inherited debt. An agent writing its own
   acceptance is the Goodhart shape dc-grrb exists to avoid — it worked here,
   but only because the offer was to a cooperating agent.
6. **Print dc-kpqg's floor line in every brief.** One sentence; it recalibrates
   the agent's whole relationship to the graph, per the measured lesson of
   th-3pq2 that shelf lines don't transmit norms but placed lines do.
7. **Keep verbatim**: PROTOCOL, ACTOR RULES, the write-set frame. Every line
   of those fired at least once in this dispatch.

### The one-paragraph verdict

The payload's contract-and-semantics core is excellent and fully earned —
nothing in THE WORK, dc-crea, the protocol, or the actor rules was wasted, and
even one "noise" shelf line (dc-f79h) quietly paid for the whole shelf concept.
But the payload is two payloads interleaved: a curated brief (top) and an
unranked area dump (bottom), and the dump's cost is not tokens — it is that
real signal (two code-map reports, one heir item, one governing decision)
drowned at exactly the moment a complete-feeling top half had already switched
off my instinct to go look. A brief that feels complete had better be complete,
or say where it isn't; th-uacu already knows this, and this dispatch is one
more data point for it.
