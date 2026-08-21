# a prose lead-in zeroes the user-owned count

Dispatch report for **it-drsu** (bug, area `cli`). Vein: **cl-stkv**
(`user-owned-extent`), sourced to `src/queries.rs` and `src/render.rs`,
supporting it-drsu.

## The acceptance line

> a report's declared user-owned calls are counted whatever prose stands
> between the section head and its entries: a lead-in line never reads as a
> declaration of none, and the count still ends at the next heading

**Met**, pinned by `a_lead_in_before_the_list_never_reads_as_a_declaration_of_none`
in `tests/basic.rs`, and measured on the real corpus below.

## What landed

**1. `queries::declared_user_owned_calls` walks the section instead of
stopping at its first sentence.** The old loop ended on the first non-blank,
unindented line that was neither a list entry nor the bare word `none` — which
is exactly what a lead-in sentence before a list is. The walk now ends only
where a section really ends:

- a markdown heading (unindented, always);
- a fresh unindented paragraph once entries *or* a lead-in have already been
  read — an un-headed report's next section;
- the end of the report.

A blank line closes a paragraph and never the section. That retires a second,
unreported under-count the old `blank line after entries closes the section`
rule carried: a **loose list** — blank lines between its entries — used to stop
at the first entry, and now keeps its full count.

**2. The count is the entries; with no entries the section still speaks.**
Prose opening with the whole word `none` declares zero (`none.`, `none declared
by the agent (…)`, `**none.** One near-miss checked rather than assumed` — all
three are real shapes in `docs/reports`; `Nonetheless` is deliberately not one).
Any other prose is **one** declaration, the rule the head line's own tail
already used.

The asymmetry is the repair, not a detail. An under-count to zero prints
`user_owned_reconcile`'s *reconciled* over a call the report declared, and the
dangerous case is `declared 0` against `filed 0` — a false all-clear on the one
accounting no hook can see. An over-count only asks the judge a question, which
is the posture that line already takes.

**3. The contract now states the shape it is read in.** `render::brief`'s
RETURN spec carries one line under `framings::USER_OWNED_SLOT`: *one entry per
call under that head — a lead-in sentence before the list is fine, indented
continuations belong to their entry, and the section ends at your next
heading.* Nothing said this before; the requirement was discoverable only by
reading the parser or building an instrument, which is exactly what the it-2eqk
agent had to do. Pinned by an added arm of
`return_spec_and_first_echo_carry_the_user_owned_calls_rule`.

## Measured on the corpus, not reasoned

Both binaries were run over every registered report in `docs/reports` (48
files: 11 carry the section, 37 predate it) through a throwaway instrument,
now deleted. Pre-fix and post-fix readings differ on **exactly one file**:

```
2026-08-21-arc-bounded-return-it-p8rp.md    Some(0)  ->  Some(1)
```

That is the incident itself — do-yeum, the report that declared one call and
named `th-t4j3`, against which the it-p8rp harvest printed *report declares 0,
thread(s) filed under this badge: 1*. Nothing else in the corpus moves: the
`none.` reports still read `Some(0)`, the list-shaped ones still read their
counts, the pre-2026-08-20 reports still read `None` (section absent, itself
the flag).

## Probes

Each half was probed against the regression it guards before being trusted.

| reverted | result |
| --- | --- |
| break on any fresh prose line (the old rule) | the witnessed shape reads `Some(0)` where `Some(1)` is right — the incident verbatim |
| heading bound removed | an empty section counts the *next* section's list |
| prose-only sections read as zero | the never-a-silent-zero assert fails |
| RETURN-spec shape line removed | the brief assert fails |

## Test results (verbatim, read raw)

```
     Running unittests src\lib.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src\main.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\basic.rs
test result: ok. 135 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.49s
     Running tests\broken_pipe.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests\observed_set.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests\store_lint.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests\surface_lint.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\worktree_proof.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
   Doc-tests quarry
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`target/release` was rebuilt so `q` on PATH carries the fix — the harvest that
reads *this* report reads it with the repaired parser.

## Graph acts

- **cl-stkv** minted (`vein`, asserted) — `about` ar-c7f5, `source`
  `file:src/queries.rs` and `file:src/render.rs`, `supports` it-drsu.
- **cl-cjbb** affirmed on `file:src/queries.rs`: its sentence about
  `declared_user_owned_calls` (section head, `None` / `Some(0)` / `Some(n)`)
  is still exactly true, and its named pin
  `declared_user_owned_calls_parses_the_section_shapes` still stands and still
  passes. cl-stkv mentions it rather than leaning on it — the pattern cl-hct3
  set for the same claim.
- Nothing else affirmed. The repo carries a 107-entry drift backlog, almost all
  of it on files this arc never opened; affirming those would claim review of
  changes never read.

**Composed register (dc-vzvf, ratify-or-amend):** one new string — the RETURN
spec's *shape it is read in* line. It is composed inline in `render.rs` beside
the brief's other structural lines rather than in `framings.rs`, which is
outside this arc's lease and whose `USER_OWNED_SLOT` is ratified prose on this
channel. Nothing in the ratified slot was touched; the new line sits under it
and adds only the read shape.

## user-owned calls

none.

The one place the user's pen was in reach — amending `USER_OWNED_SLOT` itself
to carry the shape — was declined rather than taken, and the shape landed in a
new line beside it instead. Composing a new string is the dc-vzvf channel,
which is this report and not a thread (the it-x4bb precedent); amending the
ratified one would have been the thread, and th-t4j3 already holds an open
request of exactly that kind against a neighbouring string.

## Reflections

**The item's fork was right, and its warning was the load-bearing half.** "Skip
prose lines rather than breaking on them" is the fix; "the break must still
fire on a genuine new section" is what stops the fix from being worse than the
bug. A naive skip-until-heading would have made an un-headed report swallow its
next section's list — six reflections read as six declared calls. What I
actually built ends the walk at the *second* prose paragraph, which is why an
un-headed report is safe.

**The residue that leaves, in the safe direction.** A lead-in of two or more
paragraphs before the first entry still ends the walk at the second paragraph
and reads as one declaration instead of the true count. That is a gap the judge
is asked about, never a false zero, and it is the price of the un-headed-report
guard above. If it ever bites, the honest cure is a report-wide section index
(find every heading first, then count inside the one you want) rather than more
line-by-line heuristics — that is a rewrite, not a patch, and nothing has asked
for it yet.

**The residue that stays open, and is not mine to close.** The item named a
second cure: *an agent should be able to see what the machine read.* I could
not build it. There is no verb that prints the parse of a report, and adding
one means `main.rs`, outside this lease. Every agent that wants to check its own
section still has to do what the it-2eqk agent did — write a throwaway test,
read `Some(1)`, delete it. I did the same thing this arc to measure the corpus.
That is three agents in two days building the same disposable instrument, which
is a fairly loud signal. Not filed as a thread (it is a feature gap, not a
user-owned call, and a stray badge thread would skew this arc's own
reconciliation) — flagged here for the dispatcher to file. `q brief --report
<path>` or a `--parse` arm on something is a small surface.

**What made this findable at all was that the previous arc wrote its report
honestly.** The parser said zero, the file said one, and the mismatch was
visible only because a human sat at the harvest and read both. The dangerous
version of this bug never surfaces: `declared 0` against `filed 0` prints
*reconciled* and everyone moves on. I keep noticing that the accounting's whole
value rests on the count being wrong in the direction that makes someone look.

**Small surprise.** The corpus sweep was worth far more than I expected going
in. I had a unit test that encoded the witnessed shape and thought that
settled it; running both binaries over all 48 real reports is what actually
told me the fix changes exactly one reading and nothing else. A parse repair
without a corpus diff is a hypothesis.
