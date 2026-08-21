# dispatch report — it-2eqk

**the leased return silences the land-time vein prompt: a registered report is a doc inside the observed set**

## outcome

> a dispatched landing whose agent registered its report still draws the
> landed-uncited prompt when no claim or doc cites the code it touched, and the
> arc report path alone never satisfies the check

Met. Measured at both ends and probed against the regression it guards.

## what landed

One new predicate and two one-line consumer calls.

`queries::landmark_globs(&[String]) -> Vec<String>` (src/queries.rs) drops every
glob `coord::is_arc_report` matches. Both consumers of the land-time landmark
check route their globs through it before `files_cited` sees them:

- `main.rs::vein_check` — the release seat and the `status=done` flip;
- the wrap boundary backstop in `main.rs`'s wrap arm.

`files_cited` itself is untouched. Its doc comment now says so explicitly: the
predicate answers literally about whatever it is handed, and callers making the
landmark check filter first.

### why at the consumer, not inside `files_cited`

Because the predicate is not wrong. A registered doc whose own path sits inside
the checked globs really is a citation of that path — that clause is deliberate
(dc-grrb: presence of citation, never quality). What is wrong is *which globs
the landmark check hands it*: the arc's return is accounting, never a landed
capability, so it can neither cite one nor be cited for one. Pushing the
exclusion into `files_cited` would have made a general predicate lie about its
own name for the benefit of one caller.

### filtered before the prompt, not only before the check

The prompt names the files it wants a vein for. That list has to carry the code
that landed, never the paperwork about it, so the filter sits ahead of both the
check and the `{:?}` that prints the globs.

### the second consumer is reachable, not defensive

The brief left it conditional ("if the second call site … is repaired the same
way"), so I checked rather than assumed. `q harvest` calls
`coord::clear_dispatch`, which prunes the held entry, the acting rows, the pins
and the badge attention — and never the lease globs. So a dispatcher who
harvests and lands without releasing reaches the boundary with the arc's report
path still in the held globs, and the wrap backstop reads the **lease**, not the
observed set. Same defect, second seat. Repaired the same way and measured end
to end.

The exclusion does not reach a dispatcher's own broader `docs/reports/**` glob:
`coord::is_arc_report` matches concrete paths only, so an authored write-set
covering the reports zone survives the filter, and a doc landing under it stays
a real citation. That is the it-3prx boundary honored, not re-litigated.

## how it was measured

`the_arcs_own_return_never_satisfies_the_land_time_landmark_check` in
tests/basic.rs.

**The defect, on `files_cited` itself.** A real dispatch fires; the observed set
is `["src/geo/pass.rs", <the arc's leased report path>]`; nothing in the graph
cites either, and `files_cited` is false. The agent then does exactly what the
RETURN spec demands and nothing else — writes the report at the leased path,
registers the doc, links `supports` — and the same call flips to **true**. No
claim, no file edge, nothing said about the code. That assert stands in the
instrument as the pre-fix half, so the silence can never come back unnoticed.

**The fix, pure.** `landmark_globs` on that observed set yields exactly
`["src/geo/pass.rs"]`, which is still uncited. The report path alone judges to
the empty set — nothing to ask about, never a citation. `docs/reports/**`
survives untouched.

**Both consumers, end to end through the spawned binary.** `q set <item>
status=done` under the holding session with both touches accrued: the
`landed uncited:` line speaks, names `src/geo/pass.rs`, and does not name the
report path. Then `q harvest`, an assert that the lease *outlived* the harvest
still carrying the report path, and `q wrap`: the boundary backstop speaks,
names `src/geo/**`, and does not name the report path.

**Probed against the regression each half guards.** Removing the `vein_check`
call and re-running: the landing prompt vanishes entirely from a dispatched
landing (test fails at the `find` for `landed uncited`, with the full `q set`
output showing homework and the view line and no prompt). Removing only the
wrap-site call: the landing half still passes and the wrap backstop goes silent.
Both restored; full suite green.

## test results, read raw

```
tests\basic.rs        — test result: ok. 134 passed; 0 failed; 0 ignored
tests\broken_pipe.rs  — test result: ok. 2 passed; 0 failed; 0 ignored
tests\observed_set.rs — test result: ok. 1 passed; 0 failed; 0 ignored
tests\store_lint.rs   — test result: ok. 1 passed; 0 failed; 0 ignored
tests\surface_lint.rs — test result: ok. 1 passed; 0 failed; 0 ignored
tests\worktree_proof.rs — test result: ok. 3 passed; 0 failed; 0 ignored
Doc-tests quarry      — test result: ok. 0 passed; 0 failed; 0 ignored
```

`basic` was 133 before this arc; the new test is the 134th. Release binary
rebuilt after the last source edit, so `q` on PATH carries the fix.

## graph writes

- **cl-8h8j** — `` `landmark-globs`: the arc's own return drops out of the
  land-time citation check — one filter point, both consumers `` — claim·vein,
  about cli + process, source `file:src/queries.rs` and `file:src/main.rs`,
  supports it-2eqk. Written to survive assay: it carries the measured defect,
  the reason the exclusion sits at the consumer, the reachability argument for
  the wrap seat, the surviving `docs/reports/**` case, and the named instrument
  with its probes.
- **th-27fg** — the user-owned call below, queued.
- **do-hdbq** — this report, registered from the agent's seat at the leased
  path, `supports` it-2eqk.

No affirms. Nothing my diff touched is sourced by a claim my hands invalidated:
cl-ue2e sources `src/coord.rs`, which this arc did not open to write. The 106
`behind` entries the store carries are pre-existing blob drift across the whole
tree, not this arc's residue, and affirming into them would be claiming re-reads
I did not do.

No lineage edges to cl-ue2e, cl-up6s or dc-grrb: the house convention for veins
(cl-rs2r, cl-zj2c, cl-rr2j) keeps claim-to-claim and claim-to-decision lineage
as body mentions, and the legal rels available to me here (`supports`,
`refutes`) would state something stronger and backwards. Mentioned, not linked.

## user-owned calls:

- **th-27fg** — `files_cited`'s doc-path clause counts any registered doc inside
  the touched set as citation — does it want tightening?

The brief raised this as "worth deciding while here … a separate question, not
this item's". I did not decide it, and I think the reservation is correct on
stronger grounds than scope: dc-grrb is a user-provenance ruling that says what
this check *means* — presence of citation, never quality, a prompt and never a
gate. Dropping the arc's own return is a defect repair with a measured silence
behind it. Narrowing what counts as a citation in general is a semantics change
to a user-ruled construction, and that pen is not mine. The thread states the
clause, why the fork is the user's, and both directions with their costs.

That is the only one. The other forks this arc met — filter at the consumer
versus inside the predicate, one shared point versus two inline filters, whether
to repair the second call site — are build-time judgments, taken and argued in
the vein body.

## reflections

**The measurement was already done, and that changed how I worked.** The brief
carried the defect measured on a throwaway instrument at the it-3prx build, then
deleted. That is a strange artifact to inherit: the finding survived, the
instrument did not. My first real act was rebuilding it — the two-assert shape,
the second one the failure — as a permanent half of the new test. I think that
is the right instinct generally: if a measurement is load-bearing enough to file
an item over, the instrument that made it belongs in the suite, not in a
scratch file. Filing the item without the instrument cost this arc maybe twenty
minutes of re-derivation, which is cheap here and would not be cheap on a
finding nobody could reproduce.

**The conditional in the brief was the most useful sentence in it.** "and
src/queries.rs if the second call site at main.rs's release path is repaired the
same way" — hedged, and slightly wrong in its geography (the second call site is
the wrap backstop, not the release path; release goes through `vein_check` like
the done-flip does). Following the hedge instead of the label is what turned up
the real question: is the wrap site even reachable with a report path in its
globs? It is, and only because `clear_dispatch` deliberately does not touch
lease globs. I would not have found that by trusting either the label or my own
first reading of the flow.

**Where I nearly went wrong.** My first instinct was to put the filter inside
`files_cited`. One place, no call sites to keep in sync, and the two consumers
are the only callers today. I talked myself out of it on the strength of the
predicate's name, and I still think that is right — but the honest version is
that a general predicate with exactly two callers is barely general, and the
argument would flip the moment someone wanted `files_cited` for something that
is not the landmark check. The vein body states which side of that line the
mechanism sits on, so a future builder inheriting a third caller has the
reasoning rather than just the code.

**Friction worth naming: the graph's own drift is loud.** `q query behind`
reports 106 entries, nearly all file-blob drift on `src/*.rs` from arcs that
edited files other claims source. My arc adds to it — every claim sourcing
`src/main.rs` or `src/queries.rs` is now one blob behind because of me, and none
of them are wrong. It means "behind" is not a signal a working agent can act on
at this size; it is background. The last commit message mentions a drift pass
sketched, which reads like the same observation from the design seat.

**Small thing I got wrong and fixed.** I filed th-27fg by passing the whole
multi-line body as the `<TITLE>` argument to `q new thread`, and the node took
all of it as its title. `q claim` derives a title from the first line and offers
`--title` for the truncation case; `q new` takes `<TITLE>` literally and wants
`--body` separately. Repaired in place (`q set title=`, then `q edit --body`),
at the cost of three versions on a fresh node. The two verbs' argument shapes
diverging that way is a real trap for an agent that has just used the other one
— not a defect I'd file, but worth the sentence.

**One piece of debris in the observed set, deliberately made and removed.** I
did not want to hand the harvest seat a report whose user-owned section silently
parsed as zero, so I wrote a throwaway `tests/zz_scratch_parse_check.rs` that
ran `queries::declared_user_owned_calls` over this file, confirmed `Some(1)`,
and deleted it. It was needed because my first draft *would* have parsed as
zero: I led the section with a bold line rather than a list entry, and the
parser breaks on the first non-entry, non-indented line. The file is gone but
its path will show in this arc's observed set — that is the touch accounting
working, not a stray file. Worth noting that the reconcile's own correctness is
invisible to the agent producing it: nothing in the RETURN spec tells you the
section is list-shaped, and there is no verb that shows you your own parse. A
`q` surface that renders what harvest would read out of a report path would have
saved the throwaway — probably the same shape as the existing homework line.

**On the prompt this restores.** Reading dc-grrb after fixing it, I notice the
prompt's whole design is that it be ignorable — presence, never quality, never a
gate, "skip freely if nothing durable landed". A check that costs nothing to
skip and that went silent on one class of landing for an unknown number of arcs
is a quiet failure by construction: nobody misses a prompt they were allowed to
ignore. The defect's real cost is unmeasurable from here, which is an argument
for the instrument being permanent rather than for the prompt being louder.
