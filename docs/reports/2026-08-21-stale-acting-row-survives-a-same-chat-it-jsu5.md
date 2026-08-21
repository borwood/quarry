# dispatch report: stale acting row survives a same-chat re-dispatch: the replaced agent keeps stamping

arc: it-jsu5 · badge agent:ab3c81b3dc8f06162 · 2026-08-21

## outcome, against the return spec

The one acceptance line is met.

> a same-chat re-dispatch leaves no stale binding behind it: the replaced agent's
> acting row is cleared with the replacement, so its later acts stamp into no arc
> and its identity is free to join other work, exactly as the steal arm already
> guarantees

- `ops::dispatch` now routes **both** replacement arms — the cross-chat steal and the
  same-chat re-dispatch — through `coord::clear_dispatch` at the upsert. They are the
  same code, not two arms agreeing by discipline; they differ in who holds afterward
  and in nothing else.
- After a same-chat re-dispatch: `badge_for` resolves nothing for the replaced agent,
  the acting map holds no row toward the item, the freed identity joins other work
  (before the fix this refused with cl-kggw's "one agent, one badge"), and exactly one
  agent wears the badge once the replacement joins. The held entry survives its own
  replacement with holder intact and `joined` reset, and the lease is kept.
- Pinned by `a_same_chat_re_dispatch_frees_the_replaced_agent_the_way_a_steal_does`
  and `a_refused_fire_leaves_the_live_dispatch_standing` in `tests/basic.rs`. Full
  suite green: **147 + 2 + 1 + 1 + 1 + 3 = 155 passed, 0 failed** (read off the raw
  `test result:` lines, never through a filter).
- Each half probed against the regression it guards, confirmed failing and reverted:
  disabling the clear fails the new instrument **and** `dispatch_steal_takes_the_dispatch_whole`
  (`assertion failed: badge_for(&s, Some("ag-old"), None, None).is_none()`); restoring
  the clear at the decision point fails `a_refused_fire_leaves_the_live_dispatch_standing`.

## why the whole clear, not just the acting row

The acceptance names the acting row. I cleared the badge whole, and the argument is
that the other two pieces are the same residue wearing different keys:

- **Badge-scoped attention.** cl-b2z2's body already states "clear_dispatch drops the
  badge rows with the badge: a re-dispatched item's next agent reads with its own
  eyes." That sentence was **false** for the same-chat arm — the only arm whose name it
  uses — because nothing cleared there. It is now true, and the instrument asserts both
  halves: the residue is gone at the fire and re-delivered by the next agent's own join.
- **Store pins.** cl-aujk: "the pin lives and dies with the badge." A pin keyed to the
  replaced identity outlives the binding it served, and `clear_pins` prunes by item, so
  after the replacement the only pin it can remove is the replaced agent's.

Both re-plant themselves for whoever joins next, so the clear costs a same-agent
recovery nothing it does not immediately get back at its own re-join.

## the placement is the second half of the fix

The steal's clear used to run at the **decision** — where the held entry is read, ~100
lines before the upsert. Between those two points sit every remaining refusal: a lease
held by another session, a C7 overlap on the re-reserve, an empty write-set. A steal
that refused there had already destroyed the victim's dispatch: **the arc lost by a
command that failed.** Reachable today — release an in-flight arc's lease, let a third
session take an overlapping one, then steal, and `reserve` refuses on C7 while the
victim's held entry, token and joined agent are already gone.

So the replacement is now *decided* early and *committed* late, past the last refusal.
The steal **event** moved down with the clear for the same reason: a refused steal no
longer logs a take-over it never performed. This is the rule the `--files` shape floor
already keeps at the top of the same function (cl-74qt) — a refused fire has no side
effects — and it is the reason the same-chat clear could not simply be pasted where the
steal's was.

This is scope beyond the acceptance line, taken deliberately: the acceptance asks for
the re-dispatch to behave "exactly as the steal arm already guarantees", and the honest
way to get that is one mechanism rather than a second copy. Flagging it because it
touches the steal path, which the brief did not ask me to change.

## graph writes

- **cl-gy6q** — `` `replacement-clear` `` (vein, asserted): the mechanism above, about
  cli, sourced on `src/ops.rs` and `tests/basic.rs`; `supports` cl-kggw and cl-b2z2
  (the two claims whose stated guarantees it makes true). Body carries the measured
  hole, the both-arms clear, the commit-late rule, and what each instrument pins.
- **it-mjk6** — filed bug (sketch): the fire announces the steal's release and says
  nothing about the re-dispatch's. See flag 1.
- **Affirms**, each after re-reading the code the ref points at, scoped with `--to` to
  the file I actually verified: `cl-kggw`, `cl-b2z2`, `cl-s98g` (`--to file:src/coord.rs`),
  `cl-aujk`, `cl-6ctr` (`--to file:src/ops.rs`). cl-kggw's `supports`→cl-dqt4 ref and
  cl-b2z2's `tests/basic.rs` ref are deliberately left behind — ground I did not review.
- `coord::clear_dispatch`'s doc comment now names its four callers and why a replacement
  calls it at the upsert; it previously listed three.

## flags for the dispatcher

1. **The fire says nothing about a same-chat replacement, and now it releases an agent.**
   `main.rs` prints the loud `⚠ STOLEN from …` line for the steal and only
   `[re-dispatch: lease kept]` for the re-dispatch — which, after this landing, has cut
   the previous agent's badge, pin and attention loose. `src/main.rs` is outside this
   arc's write-set, and `DispatchOutcome` carries no field for it; the shape is a
   `replaced: Option<String>` off `live.joined`, already in hand at the commit point.
   Filed as **it-mjk6** (bug, sketch) rather than left in prose — drop it if you would
   rather shape it differently.
2. **cl-aujk's body enumerates three clearing callers and there are now four.** "harvest,
   land, and the steal take-over all clear it" is no longer the whole list. What it
   asserts stays true, so I affirmed it rather than leaving it behind, but the
   enumeration misleads by omission now. Amending a ratified body is the dispatcher's
   hand (the cl-dqt4 / cl-v2vh precedent).
3. **`store::clear_pins`' doc comment carries the same stale enumeration**
   ("harvest, land, steal-takeover") and `src/store.rs` was outside the lease, so it is
   untouched. One line, whenever `store.rs` is next open.
4. **The recovery flow now costs a re-join.** A dispatcher re-dispatching to recover the
   *same* agent leaves it unbadged until it runs the new `q join`, so its writes into the
   leased zone deny in the meantime (cl-2pc9). That is the acceptance working as written
   — "its later acts stamp into no arc" cannot be true and its writes keep landing — and
   both the spawn line and the deny name `q join`, so the road out is signposted. Naming
   it because it is a real behaviour change to a documented flow.
5. **`q set <node> title=…` does not re-slug the filename.** cl-gy6q's file is still
   `cl-gy6q-eplacement-clear-…md` from the mangled first title (my shell ate the leading
   backtick; see reflections). The id is the key so nothing resolves wrong, but if the
   slug is meant to track the title, this is a small gap; if it is meant to be stable
   from mint, ignore this.

## user-owned calls:

none

No fork here contradicts or overturns a user ruling. The one judgment I stretched — 
touching the steal path to unify the clear — is a code-shape call inside my own lease,
argued above and pinned by an instrument, and flag 1's surface question is filed as work
rather than left as a call. The standing rulings this arc leaned on (dc-qyr5's per-item
keying, cl-kggw's one-badge rule, dc-ygzz's defects-are-bugs-on-sight) all point the same
way as the fix.

## reflections

**The bug is a comment that was true when it was written and never re-read as a
specification.** The steal's clear carried its own rationale inline — "so a stolen-from
agent's later acts stop stamping into an arc it no longer works" — and that sentence is
exactly as true of the re-dispatch arm twenty lines above it. Nobody had to decide that
the re-dispatch did not need it; the two arms were simply written at different times and
the second one never looked up. What makes this class survivable is that the *reason* was
recorded next to the code. I found the fix by reading the steal's comment, not the steal's
behaviour.

**I nearly shipped the smaller version, and it would have been worse.** The obvious patch
is `clear_dispatch` in the same-chat match arm, three lines, mirroring the steal exactly.
That reproduces the steal's own latent defect (state destroyed before the refusals that
follow it) into a second place, and it leaves two clears that have to keep agreeing.
Working out *why* the steal could afford an early clear — it cannot, it just never got
caught — was the useful hour of this arc.

**The hazard I could not fully price: the fork case.** After the clear, a replaced agent
sitting in a worktree fork has no store pin, so its next act resolves the store by
discovery, which in a fork finds the fork's own checked-out `graph/`. Unstamped-in-canon
(what happens if the pin survives) is arguably *more* visible than silently-written-into-a-
fork-copy. I went with the clear because the steal already behaves this way and because a
replaced agent is meant to stop, but I want to be honest that I chose consistency over a
measurement I did not take. If a fork agent has ever been re-dispatched mid-flight on this
machine, the log would show it better than my reasoning does.

**On affirming.** I scoped every affirm with `--to` and left two refs behind, the same
call the it-xwpw report describes wrestling with. The scoped surface told me exactly what
I was leaving ("1 other ref(s) on this node are still behind"), which made the
conservative choice cheap to take and easy to report — cl-2kxr's landing works. I still
do not know whether an arc that drifts a file owes the whole node; it is the second arc in
two days to stop at the same fork, which suggests the question wants a ruling rather than
two agents' judgment.

**Friction worth naming, and it is this repo's own subject matter.** My first `q claim`
mangled its own title: PowerShell parsed the backtick opening `` `replacement-clear` `` in
a double-quoted argument as an escape (`` `r `` is a carriage return), so the node minted as
"eplacement-clear" with a literal CR in its YAML. The fix was `q set title=` inside a
single-quoted here-string. Two things follow. The repo's deliberate-name floor (dc-qvtz)
makes backticked names the house style for veins, and the house shell eats backticks in the
one quoting form an agent reaches for first — that collision will keep happening. And the
graph now has at least one node whose filename slug disagrees with its title, which is flag
5. Neither is a quarry defect; both are the shape of thing quarry's own write-shape parser
work (cl-zj2c, cl-p4k2) exists to handle from the other direction.

**The model this arc ran on.** The join said the acts file under `claude-opus-5`. My exact
id is **`claude-opus-5[1m]`** (Opus 5, 1M context); the harness's own record agrees at the
grain it can see — all 107 assistant entries in
`…/subagents/agent-ab3c81b3dc8f06162.jsonl` say `"model":"claude-opus-5"`, and the sidecar
says `{"agentType":"general-purpose",…,"model":"opus"}`. The `[1m]` variant appears on
neither road, exactly as the it-xwpw report found. Attribution is right at the grain the
graph works in; noted only because the join asked.
