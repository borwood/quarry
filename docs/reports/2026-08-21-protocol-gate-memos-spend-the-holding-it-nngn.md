# dispatch report — it-nngn

**protocol-gate memos spend the holding session: the once-per-rule key rides env session, not the acting identity**

Arc: it-nngn · area cli · badge agent:ac883212e595450b5 · 2026-08-21
Model: claude-opus-5 (the harness's own record of this agent; the join stated
claude-opus-5 and that is what this agent is — the exact id is
`claude-opus-5[1m]`, the 1M-context variant of the same model, so the
attribution the arc files under is correct as stated).

## The outcome

> a protocol memo is spent by the reader that actually saw the teach: the
> once-per-rule key rides the acting identity, badge-scoped for a joined arc
> and session otherwise, so a joined agent never marks a rule delivered to
> its holding session nor inherits one that session already consumed

**Met.** `protocol::gate_if_needed` and `protocol::save_intent` now key on
`coord::attention_key(store)` — the same reader `it-csm3` installed on
watermarks and drift deliveries under `dc-pwyd`. Measured end to end through
the spawned binary in both directions, and probed against the regression.

### The defect, measured before the fix

The instrument reproduces it verbatim by restoring the one line. A joined
agent (env `QUARRY_SESSION=design`, `QUARRY_AGENT=ag-proto`, badge live)
trips a gate-tier protocol; the memo file records

```json
{"delivered":[{"session":"design","rule":"do-rqgc"}], ... }
```

— the rule marked delivered to `design`, the holding chat whose eyes never
saw the teach. The behavioural half follows immediately: the holding
session's *own* first matching mint then executes untaught (`exit 0`, node
minted, no gate), where it must gate. The intent the agent saved carried the
same wrong owner.

Post-fix the same run records `{"reader":"badge:it-5zwc", ...}` and the
holding session's own first mint gates as it should.

### What landed

**`src/protocol.rs`**

- `Delivered.session` → `Delivered.reader`, keyed on `coord::attention_key`.
  Badge-scoped for a joined arc (`badge:<item>`), session otherwise.
- `StoredIntent.session` → `StoredIntent.reader`, same key. The field was
  written and never read anywhere in the tree, but leaving it on
  `current_session` would have kept the two halves of one gate naming two
  different identities — and `save_intent`'s one caller, `main.rs`'s area
  gate, already gates on `attention_key` (it-csm3). Both halves now name one
  identity.
- Both fields carry `#[serde(alias = "session")]`, which is the whole
  continuity story: for any **bound** session the old env-session string *is*
  the new reader key (`coord::session_key`), so every real standing memo
  carries across the rename with nothing re-delivered. The one row that moves
  is the unbound fallback — protocol said `"default"` where attention state
  says `"unbound"`; unified on `session_key`'s spelling, it re-delivers once,
  machine-local. This repo's live `graph/.intents.json` holds exactly one such
  row (`{"session":"default","rule":"do-n47r"}`), so the observable cost of
  the unification here is: the journal charter gates once more for an unbound
  context, and never again.
- `take_intent`'s two refusals stopped saying "the protocol is already
  delivered to this session" (false at a joined agent's seat, where the memo
  the retry rides is the *arc's*) and now say "delivered to you", which is
  true from whichever seat holds the token. **Flagged for ratify-or-amend
  under dc-vzvf** — composed prose, two strings, presented rather than
  assumed.
- New: `protocol::clear_delivered(store, reader)`.

**`src/coord.rs`**

- `clear_dispatch` calls `protocol::clear_delivered` beside
  `clear_area_reads`, on the same `badge_attention_key`. This is not scope
  creep; it is the sentence `cl-b2z2` already asserts — *"clear_dispatch
  drops the badge rows with the badge: a re-dispatched item next agent reads
  with its own eyes"* — which badge-scoping the memo would have made **false
  for a new class of badge row** if left out. Without it, a re-dispatch or a
  steal hands the replacement agent the replaced agent's consumption, which
  is the same has-this-reader-seen-it error one seat over. `cl-gy6q` and
  `cl-aujk` were false for the same-chat arm until it-jsu5 landed for exactly
  this reason; this arc does not add a third instance.

**What deliberately did *not* change.** Saved intents are left un-pruned at
`clear_dispatch`. They are one-time tokens on a 1h TTL — work in progress,
not attention — and an orphaned token still resumes while its rule re-gates,
so nothing is stranded. Pruning them would invent a lifecycle nobody owns and
would kill a live `q resume` for no benefit. Stated as residue below.

### The instruments

Both in `tests/basic.rs`, both end to end through the spawned binary, because
the reader only diverges from the session in a real dispatched shell:

- `a_protocol_memo_is_spent_by_the_reader_that_saw_the_teach` — a gate-tier
  protocol on `q new thread`; the agent joins wearing the holding session's
  env plus its own agent id (the real dispatched shape) and gates; **the
  holding session's own first mint still gates** (the defect); the memo row
  is `badge:<item>` and nothing else; the agent's second mint executes
  untaught; the saved intent resumes from the agent's own seat; a *second*
  agent gates even though the holding session has by then consumed the rule
  (the inheritance direction); and a same-chat re-dispatch's next agent is
  taught rather than handed the replaced agent's memo.
- `a_legacy_session_keyed_memo_carries_onto_the_reader_key` — a legacy-shaped
  `.intents.json` written by hand; a bound session's standing memo carries
  with no re-delivery, the unbound fallback re-delivers once and records
  itself under the unified spelling.

Each half probed against the regression it guards, all three probes run and
reverted:

| probe | what fails |
|---|---|
| restore `current_session().unwrap_or("default")` in `gate_if_needed` | the holding-session assert (`left: Some(0), right: Some(2)` — the mint executed untaught), and the memo row reads `"design"` where `"badge:it-…"` is right |
| drop the `clear_delivered` call in `clear_dispatch` | the re-dispatch assert — the replacement agent inherits the replaced agent's consumption |
| drop `#[serde(alias = "session")]` | the legacy file fails to deserialize, `load()` falls to `unwrap_or_default()`, and **every** standing memo is silently lost |

Full suite green after the fix, read off the raw lines, never through a
filter: `basic` 152 passed / 0 failed (150 before this arc), `broken_pipe` 2,
`observed_set` 1, `store_lint` 1, `surface_lint` 1, `worktree_proof` 3.

## What this arc could not land, and filed instead

**it-sh8e** (item·bug, cli, `depends-on` it-nngn) — *the protocol gate still
says once per session: `print_gate` speaks to the wrong reader now that the
memo rides the acting identity*. `main.rs::print_gate` still opens "this act
carries project protocol, delivered once per session" and closes "this
session is now cleared for this rule". Both sentences are **false at a joined
agent's seat as of this landing**, and false in the expensive direction: an
agent reads it and concludes its dispatcher will not be taught; a dispatcher
reads it and believes it has been. The cure is already in the same file —
`main.rs::attention_scope(reader)` renders "this dispatch arc" / "this
session" and the area gate beside it uses it in exactly these two slots — and
the reader is one call away (`coord::attention_key(&store)`, with `store` in
scope at the `Cmd::New` arm that calls `print_gate`). `src/main.rs` is
outside this arc's write-set and an agent may not extend its own lease, so it
is filed on sight per `dc-ygzz` rather than left unsaid. It is a two-line
change for whoever holds `main.rs` next; its wording rides the `dc-vzvf` flag
channel like the two strings this arc did compose.

I want to be plain about this one: I have landed a fix whose user-facing
surface now lies. The lie is small and the item is filed with the exact
substitution, but the arc is not honestly "clean" until it-sh8e lands.

## Graph writes

- **cl-2j49** — `` `gate-memo` ``: a protocol gate memo is attention state —
  it rides the reader key and dies with the badge. claim·vein, asserted,
  about `ar-c7f5`, sourced on `src/protocol.rs`, `src/coord.rs`,
  `tests/basic.rs`. It deliberately does **not** restate `cl-b2z2`; it names
  the third surface, the clearing point both share, the continuity mechanism,
  and — the part I most wanted recorded — the **roster**: what rides the
  attention key (area watermarks, drift deliveries, gate memos) and what
  pointedly does not (the stored intent's own lifecycle; heartbeat, leases
  and held entries, which `dc-pwyd` leaves on the session on purpose). If a
  fourth surface is ever added, that list is the thing I wish had existed
  when I started.
- **cl-b2z2 -[supports]-> cl-2j49** — the one edge, and it is a real lean:
  this mechanism *is* `coord::attention_key`, the function `cl-b2z2` owns.
  Per `dc-ez67` it lands loud for the harvester to affirm or unlink. It bumped
  `cl-b2z2` to v4 and put `cl-gy6q` behind on that ref; I reviewed the change
  (an edge, no content moved) and affirmed it.
- **Affirms**, each scoped and each earned by an actual re-read, never a
  queue-clearing sweep: `cl-gy6q --to cl-b2z2`; `cl-b2z2 --to
  file:src/coord.rs` (re-read against the attention region I extended);
  `cl-b2z2 --to file:tests/basic.rs` and `cl-gy6q --to file:tests/basic.rs`
  (their pinned instruments are untouched and ran green in the full suite
  above). Both claims are now fully current. I did **not** touch the other
  ~140 entries on `q query behind`; they predate this arc and belong to the
  standing drift pass.
- **th-3uhm** — the user-owned call, below.
- **it-sh8e** — above.

Two mentions worth carrying that I did not turn into edges, because nothing
leans: `src/protocol.rs` carries **no** grounded claim at all before cl-2j49
— the protocol layer's only registration was `it-3wef`, done and archived —
so this module has been unsourced ground the whole time; and `cl-aujk`,
`cl-gy6q` and `cl-8h8j` all describe `clear_dispatch`'s cargo in prose that
now has one more item in it, which the dispatcher may want to amend at their
own hand rather than mine.

## user-owned calls:

One.

- **A gate-tier protocol now intercepts once per ARC rather than once per
  session, and a gate is a heavy interception** — filed as **th-3uhm**
  (thread, open, about `ar-c7f5`). `it-csm3` moved *inline* deliveries onto
  the reader key: a watermark line costs a printed paragraph. A **gate** is
  not that — it refuses the act, exits 2, saves an intent and demands
  `q resume`. Pre-fix the holding session absorbed that once and every arc it
  dispatched rode free; post-fix each joined agent pays it on its own first
  matching act. On this repo's own numbers that is roughly a dozen
  gate-and-resume round trips a day instead of one, and the floors work
  (`it-vzx6`, `it-qra3`, `dc-yd9s`'s per-kind floors as protocol content)
  multiplies the number of gate-tier rules. I built it as specified and I
  believe it is right — a memo spent by a session the agent never was is a
  teach nobody received, which is `dc-pwyd` exactly — but *whether the gate
  tier is the right vehicle for a rule aimed at dispatched agents* (versus the
  brief, which already delivers free at the join) is a protocol-layer policy
  question, and `dc-9hgd` and `dc-yd9s` put protocol content in the project's
  hands, not the engine's. Nothing was built toward the fork; the thread
  records the cost before it multiplies.

Nothing else in this arc contradicted or leaned on a standing ruling.
The continuity break was pre-authorised by the item's own body ("unifying
changes memo continuity once, machine-local"); the `clear_dispatch` extension
is `cl-b2z2`'s own sentence rather than a new policy; the `supports` edge is
the informational rel `dc-ez67` explicitly hands to questing agents; and the
two composed strings in `take_intent` ride the `dc-vzvf` flag channel —
presented for ratify-or-amend, not decided here.

## Reflections

**The lease boundary produced the one thing I am unhappy about.** The
write-set is drawn around the mechanism (`protocol.rs`, `coord.rs`,
`queries.rs`, tests) and the surface that *speaks* the mechanism lives in
`main.rs`. So a correctness fix ships with its own user-facing text made
false, and the only move available to me was to file a bug against my own
landing. That is the system working — `cl-74qt` and `cl-ue2e` are both about
the agent being denied a write it needs — but it is worth noticing that the
denial here was not a mistake in the fire: nobody would naturally think "a
memo-keying fix touches `main.rs`". The general shape, if it recurs: a fix to
a *keyed* mechanism almost always has a surface that names the key in prose,
and the surface is usually one module away. `src/main.rs` is 3000+ lines of
exactly those surfaces, which is `th-yzmj`'s infection viewed from the lease
side rather than the reading side.

**I nearly built a twin.** The obvious "your writes" move was to mint a vein
for the protocol gate. But the mechanism is `cl-b2z2`'s — I added a caller,
not a machine. What made cl-2j49 worth minting anyway was the *roster*: three
attention surfaces on one key with one clearing point, and nowhere in the
graph did that list exist. If it turns out the harvester disagrees and this
should have been an amendment to `cl-b2z2` instead, that is a fair read and
`q unlink`/refute is cheap; I chose the new claim because the house pattern
for "extends" is a sibling claim (`cl-kggw` extends `cl-6ctr` in prose) and
amending a ratified vein has consistently been the dispatcher's hand, not the
agent's.

**The serde alias is doing more work than it looks like.** I nearly renamed
the field without it, and the probe is the reason I know what that costs: the
old file does not fail loudly, it fails *silently* — `load()` swallows the
deserialization error into `unwrap_or_default()` and every standing memo
evaporates, re-teaching every rule to every reader with nothing in the output
to say why. That swallow is a general hazard in this file (`load` is
`.ok().and_then(...).unwrap_or_default()`), and it is the reason I would not
want a future schema change here made without an instrument. I did not file
it, because "a machine-local working file degrades to empty" is a deliberate
and reasonable posture for this class of state — the same posture
`.counter-voice.json` and the area-reads map take — but it is the kind of
deliberate posture that is only safe while someone remembers it is there.

**The thing I could not check.** `StoredIntent.reader` is written and read by
nothing. I renamed it for coherence and I believe that is right, but I cannot
prove the field earns its place at all — a dead field that now carries a more
precise value is still a dead field, and the honest alternative (delete it) I
declined because a saved intent with no record of whose it is would foreclose
any future "whose intent is this" question, including the one `th-3uhm` might
provoke. Worth a second opinion at harvest.

**On the model line.** The join stated the arc files under `claude-opus-5`
and asked me to say so if that is not what I am. It is: my exact id is
`claude-opus-5[1m]`, the same model with the 1M-context window, so the
attribution is right and I am recording the bracket only so nobody later
reads a mismatch into it.
