# dispatch report: a scoped affirm prints the unscoped all-clear (it-awhz)

`--to` a target with no behind edge said "nothing behind". Fixed at the message,
and the same false all-clear found and closed one branch over on the unscoped
road, plus a scope leak in the act itself.

## outcome

**Acceptance — "a zero-count affirm says which zero it is: a scoped affirm that
restamped nothing names the scope it was given and never reads as a statement
about the whole node, while an unscoped zero keeps meaning every ref is
current." — met.**

Measured on the real graph with the release binary, the two incident commands
from the brief, both still zero-count and therefore non-mutating:

```
> q affirm cl-zj2c --to it-dt68
nothing behind toward "a bash heredoc body tokenizes as command text: ..." [item done] (it-dt68) — that ref is current.
  --to scoped this to one target: 1 other ref(s) on this node are still behind — q query behind, then q affirm cl-zj2c unscoped to restamp them all.

> q affirm cl-up6s --to it-dprv
nothing restamped — this node carries no ref toward "a newline is not a command boundary: ..." [item done] (it-dprv).
  --to scoped this to one target: 2 other ref(s) on this node are still behind — q query behind, then q affirm cl-up6s unscoped to restamp them all.
```

Both printed `nothing behind — no restamp needed.` before this change. The
second is the brief's own measurement reproduced exactly — cl-up6s behind on
two refs while the surface said otherwise — and the first shows the third
state the one line also stood for: a ref toward the target that really is
current, over a node that is not.

## what changed

**`src/main.rs` — `print_affirm`, one print point for the whole verb.** A
scoped affirm now names the scope in its own line and always follows it with
the node's reach beyond that scope. Three scoped zeros, told apart:

- `nothing restamped — this node carries no ref toward <target>.` (the
  incident: the `--to` selected nothing at all)
- `nothing restamped toward <target> — N ref(s) there are behind but not
  restampable (a dangling target or a missing file): q query behind.`
- `nothing behind toward <target> — that ref is current.`

and the caveat line under every scoped result, restamp or not:

- `  --to scoped this to one target: N other ref(s) on this node are still
  behind — q query behind, then q affirm <id> unscoped to restamp them all.`
- `  --to scoped this to one target; nothing else on this node is behind either.`

A scoped restamp names its scope too (`✔ restamped N ref(s) toward <target>`).
Node targets render through `aref` so the reader sees the title they cited,
not a bare id; a `file:` ref stands as written.

**`src/queries.rs` — `behind_node` and `affirm_scope`.** `behind_node` is the
classifier's per-node core, extracted so `behind` maps it over the store: one
classification point still (cl-34ra undisturbed), and the affirm surface can
ask about the single node under review without walking and re-hashing the
graph. `affirm_scope` splits that set at the scope the caller gave —
`target_known` (an edge toward it, or a doc's own registered path), `toward`,
`elsewhere`. Read **after** the restamp, so every line reports the state the
caller now holds rather than a prediction of it.

**`src/ops.rs` — the scope binds the act.** See "two forks" below.

**`src/main.rs` — `q affirm --help`.** The file-ref affordance the brief asked
to settle now has examples and a sentence:

```
  q affirm cl-up6s                           every behind ref on the node
  q affirm cl-up6s --to it-dprv              one node target
  q affirm cl-up6s --to file:src/teach.rs    one file of a multi-file claim
```

with the note that `file:<path>` is exactly how the behind and homework lines
print the target, and that a scoped result speaks for that target alone.

## two forks beyond the stated acceptance, both deliberate

**1. The unscoped zero was a false all-clear too.** A ref that cannot be
restamped at all — a dangling target, a missing file (`queries::behind`
severity 2) — leaves the count at zero, so the unscoped `nothing behind — no
restamp needed.` printed over refs that are behind and staying behind. Same
species as the reported defect, one branch over, on the road the acceptance
line calls safe. The unscoped zero now keeps its line **only** when the node's
behind set is empty; otherwise it says how many refs it could not clear, and a
nonzero unscoped restamp says what it left behind. This makes the acceptance
sentence true rather than nominal, which is why I did it inside this arc
rather than filing it.

**2. `ops::affirm` restamped outside its scope.** The brief says "Nothing about
the restamp behaviour is wrong — `--to` correctly limits to one target." That
is true of the edge loop and false of the path-backed-doc self-blob restamp
sitting after it, which ran regardless of `--to`. So `q affirm <doc> --to
<some edge>` on a drifted registered doc restamped the doc's own file and
returned `1` — the caller can only read that as the named target having moved.
It is the same misreport in the other direction, and it makes the count
unreadable, so a message-only fix could not have reached it. Now gated on the
scope naming that file, spelled `file:<path>` — the spelling the homework line
already advertises, so the advertised scoped command still works. This is a
fork from a sentence in the brief; it is not a fork from a user ruling
(dc-ygzz points the other way: defects are bugs on sight). Named here for the
dispatcher's judgment at harvest.

## measured

- **Tests:** full suite green — `test result: ok. 140 passed; 0 failed` in
  tests/basic.rs, plus 2 / 1 / 1 / 1 / 3 / 0 across the other targets, zero
  failures anywhere. Three tests added, all in tests/basic.rs:
  `a_scoped_affirm_zero_is_a_statement_about_its_scope_alone` (the incident in
  shape on the derivation itself: `target_known` false with `elsewhere` 2, then
  the file-ref restamp leaving `elsewhere` 1, then the unrestampable ref),
  `a_scoped_affirm_restamps_only_within_the_scope_it_was_given` (the doc-blob
  leak measured on `ops::affirm`), and
  `the_affirm_surface_says_which_zero_it_is` end to end through the spawned
  binary.
- **Probed against the regressions they guard**, each half separately:
  restoring the old two-line surface reproduces the incident reading verbatim
  (`the unscoped all-clear never speaks for a scoped zero: nothing behind — no
  restamp needed.`); collapsing the scope to `None` fails the scope-naming
  assert; removing the doc-blob gate fails the out-of-scope restamp assert
  (`left: 1, right: 0`).
- **On the real graph:** the two incident commands above, and the two affirms
  this arc owed (below), which are the first uses of the new scoped restamp
  line in anger.

## graph writes

- `cl-2kxr` minted, `kind: vein`, about `ar-c7f5`, sourced on all four touched
  files, `supports it-awhz`: `` `scoped-affirm` ``: --to acts and reports on one
  target alone.
- `q affirm cl-34ra --to file:src/queries.rs` and `q affirm cl-rr2j --to
  file:src/queries.rs` — the two claims this diff re-verified by re-reading:
  the behind classifier (whose one classification point the `behind_node`
  extraction preserves) and the species-shaped affirm teaching (whose single
  resolution point is unchanged; only its call site moved into
  `print_affirm`). Both restamped 1 and reported nothing else behind — the
  accounting the old surface could not give.
- **Mentioned, not linked:** cl-2kxr cites cl-34ra in its body. It does not
  lean on it — if cl-34ra were refuted the new mechanism would still stand;
  the sentence is a compatibility note, not a foundation — so the mention
  stands and no edge was minted. The other direction was also considered and
  declined: `cl-34ra supports cl-2kxr` would bump cl-34ra and put its citers
  behind for a relationship that is not load-bearing.

## flagged for ratify-or-amend (dc-vzvf, dc-dsdm)

Composed mechanical strings, new this landing, all in `main.rs::print_affirm`
beside the verb's other composed output rather than in `framings.rs` (which is
outside this arc's lease, and whose ratified slot is the user's pen). Riding
the standing flag channel: compose, flag, present for ratify-or-amend.

1. `nothing restamped — this node carries no ref toward <target>.`
2. `nothing restamped toward <target> — N ref(s) there are behind but not restampable (a dangling target or a missing file): q query behind.`
3. `nothing behind toward <target> — that ref is current.`
4. `✔ restamped N ref(s) toward <target>`
5. `  --to scoped this to one target: N other ref(s) on this node are still behind — q query behind, then q affirm <id> unscoped to restamp them all.`
6. `  --to scoped this to one target; nothing else on this node is behind either.`
7. `nothing restamped — N ref(s) on this node are behind but not restampable (a dangling target or a missing file): q query behind.`
8. `  N ref(s) on this node are still behind — q query behind.`
9. The `q affirm` after_help block quoted above.

`nothing behind — no restamp needed.` is kept verbatim for the unscoped
all-clear; only the condition under which it prints narrowed.

## residue, named and not fixed

- **`--to` does no key resolution.** It is a raw string compare against the
  edge's `to` field, so a slug or a title — which every other node-key argument
  in the CLI accepts — silently selects nothing. That now degrades honestly
  (`this node carries no ref toward <what you typed>`) instead of reading as an
  all-clear, which is why I left it: making `--to` resolve keys is a change to
  the verb's contract, not to its report, and it was not this item's. Worth a
  sketch item if the dispatcher agrees.
- **`q guide` says nothing about `--to`.** Its affirm sentence ("affirming ONLY
  what you actually re-read") is about honesty and still reads correctly, so
  nothing there is now wrong — but `src/teach.rs` is outside this lease and I
  could not have extended it anyway.
- **`files_cited`-style looseness is untouched**; nothing in this arc came near
  it.

## user-owned calls:

**none.** The two forks above are dispatcher-visible design calls, not the
user's: the first makes the accepted sentence true, the second is a defect
under dc-ygzz's standing ruling (defects are bugs on sight), and neither
contradicts or overturns a user ruling. The composed strings this landing adds
are the one thing here the user's pen governs, and dc-vzvf routes them: "future
landings keep the same channel: compose, flag per dc-dsdm, present for
ratify-or-amend." They are flagged in their own section above rather than
filed as a thread, because filing one would contradict that standing ruling —
threads are for calls the channel does not already route.

## reflections

The thing that struck me hardest is that this recipe is not something an agent
had to invent by copying a report. The machine composes it: `print_homework`,
`render::homework`, and the brief's behind-check all hand out `q affirm <node>
--to <target>` themselves. So the surface was handing out a call whose answer
it could not read back, at exactly the moment an agent is being told to clear
its homework. That reframes the severity for me — this is not "an agent copied
a stale recipe", it is a closed loop where the machine's own advice was
unfalsifiable in one direction.

I very nearly shipped the message-only fix. What stopped me was writing the
scoped restamp line: `✔ restamped 1 ref(s) toward <target>` is a lie if the
count can include a restamp the scope excluded, and that is exactly what the
doc self-blob branch did. The message forced the behaviour question. I would
not have found it by reading `ops::affirm` for correctness, because it reads
correct — the doc branch is simply written as if `only_to` did not exist.

The unscoped dangling case is the one I am least sure about landing inside
this arc rather than filing. It is not what I was dispatched for. My reasoning:
the acceptance line I was handed asserts the unscoped zero "keeps meaning every
ref is current", and I could not honestly report that met while the surface
printed it over a dangling ref. Fixing it was cheaper than the thread that
would have explained why I did not.

Doubt about the new surface's verbosity: every scoped affirm now prints two
lines instead of one, including the boring success case. I weighed a
silent-when-clean shape (say nothing when `elsewhere` is zero) against the
explicit "nothing else on this node is behind either", and chose explicit,
because silence is what caused the incident — an agent reading a scoped result
needs to know the surface actually looked. But it is a real cost in a verb that
gets run in batches at wrap time, and if the register feels noisy at the
judgment seat, dropping line 6 (only) is a clean amend that keeps every warning
intact.

Small friction worth recording: `q affirm --to` takes the raw edge target while
the positional node key takes id/slug/title, and I only noticed because I was
writing a message about what `--to` selected. Two argument grammars in one
verb, undocumented until this landing. The help text now says `file:<path>`
explicitly, which will carry most agents through, but the asymmetry itself is
still there.

Last, on edges: I wrote the body citing cl-34ra and then talked myself out of
an edge in both directions. The claim-to-claim `depends-on` I first reached for
is not legal (the refusal named the matrix, which was useful), and the legal
direction — `cl-34ra supports cl-2kxr` — would have bumped someone else's
ratified-adjacent claim and put its citers behind for a relationship I would
not call load-bearing. Mention what relates, link what leans; this one relates.
Registering this report surfaced **th-rb2j** — "a claim that stands on another
claim has no edge to say so: builds-on stops at decisions and docs, supports
points the load the other way" — which is this exact friction, already filed by
someone before me. My run is one more instance for it: the thing I wanted to
say was "cl-2kxr was built with cl-34ra's structure in hand", and the only
legal edge inverts the load and bumps the wrong node to say it. I found that
thread by accident, in the mint echo of the report I was registering — worth
noting as evidence for how well the touches echo works.
