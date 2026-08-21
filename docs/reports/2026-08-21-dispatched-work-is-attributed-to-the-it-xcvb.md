# dispatch report: dispatched work is attributed to the dispatcher's model

it-xcvb · arc 1 · written from the agent's seat, 2026-08-21.

## against the RETURN spec

> a node's recorded actor names the model that actually wrote it: work done by a
> dispatched subagent whose model differs from its dispatcher's is never filed
> under the dispatcher's model, and the commit convention can be honoured from
> the graph alone

**Met, with one named residue.** A dispatched arc now files under the model it
was spawned on, from its first badged act onward, and the commit convention
reads off the graph. The residue: the model reaches the badge because the
*dispatcher states it* (`q dispatch --model <model>`). Omit the flag while
spawning on a different model and the old behaviour returns — silently, as far
as any hook can see. That is not a shortcut taken; it is the whole of what the
harness permits, and the measurement below is the proof.

## the open question the brief could not answer, answered

> whether the harness exposes a subagent's model to a hook at all

**It does not.** Two independent readings:

**Probed live, from inside a subagent.** A temporary dump of the raw PreToolUse
stdin, built into `q hook session`, fired from this agent's own shell, then
reverted. The whole payload:

```
session_id, transcript_path, cwd, prompt_id, permission_mode,
agent_id, agent_type, effort, hook_event_name, tool_name, tool_input, tool_use_id
```

No `model`, in any spelling. `agent_type` is `"general-purpose"`; `effort` is
`{"level":"xhigh"}`. Nothing names a model.

**Read against the docs.** Only `SessionStart` carries `model`, and the hooks
reference states it is *not guaranteed to be present* even there. `SessionStart`
fires once per session and never for a subagent — `SubagentStart` fires instead,
and its documented input names `agent_type` and `agent_id` and no model. There
is a filed upstream issue (anthropics/claude-code#19170) saying that schema
still needs verification, so absence from the docs is not proof of absence from
the payload; see the one unprobed road below.

So the per-agent row keyed on `QUARRY_AGENT` — the cure that would have asked
nothing of the dispatcher — cannot be written today. Nothing at the agent's seat
knows the answer. The brief's preferred fork, the badge stamp, is what was
built.

**One channel does exist, and it is not one to lean on.** `transcript_path` is
in the hook payload, and the parent chat's JSONL does correlate a subagent to a
model: the `Agent` tool-use block records the `model` argument the dispatcher
passed (`"model":"opus"`), and the tool-result immediately after names the
`agentId`. But it records the *alias*, not a resolved model id; it records
nothing at all when the dispatcher omits the argument; the format is
undocumented and harness-internal; and it would mean scanning a 2.2 MB JSONL
inside a hook with a 10-second budget, on every shell command. Named here for
the record, not proposed.

## what was built

One mechanism, minted as `cl-dqt4` (`attribution-stamp`, vein, asserted).

- **`coord::DispatchState.model`** — the badge carries the model the dispatcher
  spawned with. `Option`, `serde(default)`, so every legacy entry on disk loads
  unchanged. Re-stamped on every fire by construction: a re-dispatch or a steal
  builds a fresh entry, so an arc handed to a different model carries the model
  it was handed to.
- **`q dispatch --model <model>`** — the stamp's only writer. Also logged on the
  `dispatch` event, because the badge dies at harvest and the log does not.
- **`coord::badge_model` / `coord::badge_actor`** — the one derivation point,
  the first pure over the `DispatchMap` so the defect and the cure are both
  measurable without a store, the second store-reading and already through
  `safe_actor` (a display-name model would otherwise derive USER provenance).
- **`teach::session_hook_output`** prefers the badge's model over the chat-actor
  map for the joined agent's shells, and falls through whenever nothing is
  stamped — an inheriting spawn keeps the answer that is right for it.
- **`ops::join`** stamps the join event from the badge directly. The hook firing
  that injected `QUARRY_ACTOR` into the joining shell ran *before* the bind
  existed, so that one shell still carries the dispatcher's model; the join event
  is the whole of that window, and the arc's first badged act now files right.
- **Both seats state it, neither gates.** The fire says which of the two answers
  the dispatcher got, above the spawn prompt, every time — that is the one
  station that can still fix it before the agent exists. The join says it again
  to the agent, which is the only mind told its own model and the only one who
  can report a mismatch.

### one design choice worth naming

The override is keyed on the **agent identity alone**, never on a chat or
session key. Every other identity is a real chat that fired `SessionStart` and
already has its own model recorded, so reading a chat-keyed acting row buys
nothing — and it would spread the badge's model onto the *dispatching chat's own
shells* wherever parent and subagent are indistinguishable (`record_acting`'s
no-agent-id fallback). That is mis-attribution in the expensive direction,
purchased for no gain. Pinned by test in both directions.

### one placement forced by a standing invariant

The join's attribution line rides the **bind line** rather than a line of its
own. A line of its own broke `the_store_pin_carries_a_worktree_dispatch_end_to_end`
(cl-kr7f / it-rmqy: the where-you-stand banner must be the very next thing a fork
join says, nothing between). The invariant is right and the fix is better for
it — where your acts resolve and what they file under is one sentence's subject.

### instruments

`a_dispatched_arc_files_under_the_model_it_was_spawned_on` and
`the_session_hook_injects_the_badge_model_over_the_inherited_chat_actor`, both in
`tests/basic.rs`. The second runs end to end through the spawned binary, which is
the only seat where `QUARRY_ACTOR` can honestly be absent from the environment —
the state a real hook process runs in. It measures the defect first (the
subagent's shell and its dispatcher's handed the same actor), then the cure, then
the negative controls: the dispatcher's own shell untouched while its agent
flies, and a cleared badge no longer overriding.

Each half probed against the regression it guards:

| removed | fails with |
| --- | --- |
| the hook's badge preference | `claude-fable-5` still in the injected prefix |
| the join event's badge stamp | the arc's first act reads `test-user` |
| the agent-key narrowing (chat keys admitted) | the chat-row assert |

Full suite green: 142 + 2 + 1 + 1 + 1 + 3 = 150 passed, 0 failed. Verified by
reading each raw `test result:` line, never through a filter.

## a second defect, found and filed — it-j4tx

Measured while verifying the brief's own premise, and it is live right now on the
dispatching chat:

- `graph/.chat-actors.json` records chat `b8214cb4-…` as **claude-fable-5**. All
  eighteen entries in that map read claude-fable-5.
- That chat's own transcript carries **400 assistant messages, every one
  `"model":"claude-opus-5"`**, with `isSidechain` false throughout — the harness's
  own record of what produced its turns.

`coord::record_chat_actor` is called from the `SessionStart` arm alone. Nothing
ever re-reads it. A model switch after the session starts leaves the row stale
and the rest of the session files under a model that stopped writing it.

This compounds with it-xcvb rather than duplicating it: the value a dispatched
arc inherits is not even the dispatcher's real model — it is the dispatcher's
*stale* `SessionStart` row. `cl-2kxr`, minted by that chat at 12:03:22Z the same
day, carries `actor: claude-fable-5`. Filed as `it-j4tx` (kind bug, sketch, cli +
process) with the measurement and a candidate cure.

## the one road not taken, filed as it-6ekf

`SubagentStart` *does* fire for a subagent and *does* carry `agent_id`. Its
documented schema names no model, but that schema is flagged upstream as
unverified. If the real payload carries one, a `SubagentStart` arm records
`agent_id → model` exactly as the `SessionStart` arm records the chat's,
`badge_model` reads the per-agent row ahead of the badge stamp, and `--model`
becomes an override rather than the only road — attribution then costs the
dispatcher nothing and cannot be forgotten, which is what dc-zbxj asked for.

This arc could not run the probe: it needs `.claude/settings.json` edited, which
sits outside the write-set and is the user's own harness configuration. Filed as
`it-6ekf` with the exact probe.

## what was deliberately left

- **`q guide` / SKILL.md untouched.** The guide would benefit from one sentence
  on `--model`, but the `GUIDE` constant *is* SKILL.md, and editing it obliges
  `q init --claude`, which writes `.claude/skills/quarry/SKILL.md` — outside the
  lease. `q dispatch --help` carries the full teaching instead, and the fire line
  names the flag on first use, so the loop closes. One sentence in `GUIDE` plus a
  re-init is a clean follow-on for the dispatcher's own hand.
- **No gate on `--model`.** See the user-owned call below.
- **Drift not swept.** Five claims were re-read and affirmed scoped to the file
  each was verified against: `cl-z6gc` (teach.rs), `cl-6ctr` and `cl-p6aj`
  (ops.rs), `cl-kggw` (coord.rs), `cl-kr7f` (ops.rs, its worktree instrument
  re-run green). Claims whose sources I drifted *without* re-verifying their
  mechanism — `cl-2kxr` on main.rs/ops.rs/tests/basic.rs is the clearest — were
  left behind rather than stamped. An affirm I did not earn is fool's gold.

## user-owned calls:

Two, each with its thread filed under this badge.

- **Should `q dispatch` refuse without `--model`?** — `th-e5ez`. dc-zbxj is a
  user ruling and its second half reads *identity is structural, never
  discipline*. This cure puts attribution on exactly the discipline road that
  ruling took identity off: a flag the dispatcher must remember, whose omission
  reproduces the defect silently. Refusing at the fire is the cl-74qt / dc-p6z4
  pattern this machine already uses — but an inheriting spawn genuinely runs the
  dispatching chat's model, so a refusal would demand a ritual for the common
  case where nothing is wrong, and it changes the user's own dispatcher workflow.
  **Provisional path taken:** no gate. The fire states which answer it got, every
  time, with the re-fire command in hand; the join states it again to the agent.
  Two statements, no refusal. it-6ekf may dissolve the call entirely.

- **This arc's own mints file under the wrong model, and nothing can
  re-attribute a node after the fact** — `th-t2fx`. it-xcvb was fired before
  `--model` existed, so this badge carries no stamp and every node minted here —
  `cl-dqt4`, `it-j4tx`, `it-6ekf`, `th-e5ez`, `th-t2fx`, and this report's doc
  node — is stamped `actor: claude-fable-5`. The agent that wrote them is
  claude-opus-5[1m]. A trailing `export QUARRY_ACTOR=…` in each command would
  have won over the hook's prefix and made them true, but CLAUDE.md says in the
  user's own words *don't set it by hand*, and C3 sends a fork that would
  overturn a user ruling to a thread rather than into the work. **Provisional
  path taken:** minted under the injected value, surfaced here. The underlying
  gap outlives the arc — `q set` carries no `actor=`, and front-matter actor and
  every logged event are written from `Store::actor()` at act time, so *no* road
  exists to correct a node after the fact. An it-j4tx cure would want one too.

## reflections

The thing that surprised me most was finding the defect running live on my own
shell while I fixed it. `QUARRY_ACTOR=claude-fable-5` in a shell belonging to
claude-opus-5[1m] — and then, chasing that, finding the dispatching chat isn't
fable either. The brief's premise ("recorded under the dispatching chat's model")
turned out to be one layer off: it's recorded under the dispatching chat's model
*as of SessionStart*, which the chat itself has since left behind. Two bugs
stacked so neatly that the outer one looked like the whole of it. I would not
have looked at the transcript at all if I hadn't been chasing the harness for a
model field; the second defect was a side effect of the first search.

What I'm least comfortable with is the shape of the cure. dc-zbxj's "identity is
structural, never discipline" is a good ruling and this fix does not honour it —
it honours the *outcome* while taking the road the ruling names as wrong,
because the harness left no other. I built it because the item's own shaping
preferred it and because a stated-and-visible discipline beats a silent
inheritance. But I want to be plain that `--model` is a flag someone will forget,
probably soon, and the machine will not be able to tell. The two statements at
the two seats are the best I could do without a gate, and I do not know whether
they are enough. That's th-e5ez's to settle.

The probe itself was the best hour of this arc and I nearly didn't do it. The
temptation was to reason from the docs — which, read alone, would have given the
right answer for the wrong reason (the docs don't *say* PreToolUse lacks a model;
they simply don't list one, and the SubagentStart schema turns out to be
unverified upstream). Dumping the actual bytes from inside a real subagent took
one build and one `echo`, and it turned a hypothesis into a measurement. The
mirror of that is it-6ekf, where I couldn't run the equivalent probe because it
needed a settings edit outside the lease — and I notice I'm much less confident
about what SubagentStart carries than about what PreToolUse carries, entirely
because of that.

Small friction worth recording: adding one field to `DispatchState` cost eleven
mechanical edits across the test file, and adding one parameter to `ops::dispatch`
cost forty. Neither is wrong — explicit construction is why the tests catch
things — but a builder weighing "should this be a field on the badge?" is
weighing a fifty-site diff against a worse design, and that pressure points the
wrong way. If a `#[derive(Default)]` on `DispatchState` plus `..Default::default()`
at the test sites is acceptable, the next field costs one edit.

Last: the join line placement. My first instinct was a clean separate paragraph,
and worktree_proof killed it in about four seconds. That test is doing real work
— it caught a regression I had no idea I was creating, in a file I never opened.
Whoever wrote "the banner is the very next thing the join says — nothing stands
between the bind and the orientation" and then pinned it: thank you.
