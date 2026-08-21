# the chat-actor row goes stale on a model switch (it-j4tx)

Arc report. Acceptance met: the recorded actor now names the model actually
producing a session's turns, re-derived at every hook fire; where it cannot be
derived the guide states the SessionStart grain in those words. The road was
chosen by measurement — the numbers are below, and every one was taken on this
machine rather than reasoned about.

## the defect, measured before anything was built

`coord::record_chat_actor` had one caller — the `HookCmd::Orient` arm, at
SessionStart — and `coord::chat_actor` had exactly one production reader,
`teach::session_hook_output`, which read the row back unquestioned. Nothing
between them ever re-derived.

Read live, 2026-08-21, on the dispatching chat: `graph/.chat-actors.json` held
eighteen rows, every one `claude-fable-5`, including chat `b8214cb4`. All 400
assistant entries in that chat's own transcript said `claude-opus-5`. The chat
was running Opus 5 and filing everything under Fable 5.

That row is now `claude-opus-5` on disk — corrected by the first real hook fire
after the build, before this report was written. The fix is live on the machine
it was measured on.

## the road taken, and why the numbers allowed it

The PreToolUse payload carries no model (`cl-dqt4`, re-confirmed by this arc's
own probe) but it does carry `transcript_path`, and the last assistant entry
there names the model producing turns now. The item filed that as the candidate
cure and called it unprobed, with two things to settle: the cost per hook fire,
and whether an undocumented harness-internal format could be leaned on.

**Cost.** Measured in the real hook process, against the real 2.37 MB
transcript:

| | |
|---|---|
| tail read + reverse scan, in-process | **404–517 µs** (two live fires) |
| whole `q hook session`, pre-fix | median **10.5 ms** (n=20, min 9.7, max 20.3) |
| whole `q hook session`, post-fix | median **10.0 ms** (n=20, min 9.2, max 12.2) |
| hook timeout budget | **10 s** |

The end-to-end delta sits inside run-to-run noise; the isolated cost is ~4% of
a hook fire and 0.005% of the budget. Crucially the cost does not grow with the
session, because the read is a tail seek and not a scan — that property is what
made the road takeable at all. A whole-file scan of the same transcript would
have been the thing the item was right to be wary of.

**Window size, from data not taste.** Across the 23 real transcripts on this
machine the last assistant entry sat at most **32,820 bytes** from EOF (median
~5 KB), and the largest single line anywhere in them was **144,652 bytes**.
`TRANSCRIPT_TAIL_BYTES` is 512 KiB — three times their sum. A miss is not a
failure: the recorded row stands.

**The format is undocumented, so every judgment is a filter rather than an
assumption.** An entry counts only if `type` is `assistant`, `isSidechain` is
not true, and `message.model` is a real name. `<synthetic>` is the measured
counter-example — it appears as a model value in 4 of the 23 transcripts — and
any other angle-bracketed placeholder falls with it. Anything unrecognised is
skipped and the walk continues. A truncated tail drops its first line, which is
a fragment by construction.

## the measurement that decided the design

**Inside a subagent, `transcript_path` is the PARENT CHAT's.** Probed live from
this agent's own seat — a temporary raw-stdin dump built into `q hook session`,
fired, read, and reverted; `src/main.rs` carries no net change. The payload:

```
session_id, transcript_path, cwd, prompt_id, permission_mode,
agent_id, agent_type, effort, hook_event_name, tool_name, tool_input, tool_use_id
```

`transcript_path` resolved to `…/b8214cb4-….jsonl` — the dispatching chat's
file — while this agent's own transcript sat one directory down at
`…/b8214cb4-…/subagents/agent-a11cf5b918215e053.jsonl`. The prior arc's report
assumed this without stating it; it is now measured.

Two consequences, and they are the whole shape of the cure:

1. The derived value is the **chat's** model whoever fires, so storing it under
   the chat key can never smear a subagent's model onto its dispatcher. The row
   stays chat-keyed **by construction**, not by discipline — which is the exact
   hazard `coord::badge_model` refuses from the other side, and the reason it
   was safe to let a subagent's fire write here at all.
2. `cl-dqt4`'s badge stamp keeps owning a joined agent's model.
   `teach::session_hook_output` still asks `badge_actor` first and falls through
   to the refreshed chat row, so an inheriting spawn now inherits its parent's
   **live** model instead of its stale one — strictly better, same shape.

**A subagent's transcript entries all carry `isSidechain: true`** — all 78
assistant entries of this arc's own. That makes the sidechain filter
load-bearing rather than defensive: older transcripts on this machine
interleaved sidechain entries into the parent file, and without the filter a
subagent's model would become the chat's.

## what was built

One mechanism, minted as **`cl-cv92`** (`live-actor`, vein, asserted),
supporting it-j4tx, sourced on `src/coord.rs`, `src/teach.rs`,
`tests/basic.rs`.

- **`coord::last_assistant_model(tail, truncated)`** — the pure core. Walks a
  tail slice backwards, applies the three filters, returns the first real model
  it meets. Pure over bytes, so the defect and the cure are both measurable
  without a store or a file.
- **`coord::transcript_model(path)`** — the I/O half. Seeks to
  `len - min(len, TRANSCRIPT_TAIL_BYTES)`, reads, hands the slice to the core.
  Best-effort by construction: a missing, unreadable or unrecognisable
  transcript returns `None`. A hook must never fail a shell over this.
- **`coord::refreshed_chat_actor(store, chat_id, transcript)`** — the one
  derivation point, and now the only place the row is read. Derive, fall back to
  the recorded row, and rewrite the row **only when the value moved**, so a
  settled session's fires are reads.
- **`teach::session_hook_output`** — one line: `chat_actor` became
  `refreshed_chat_actor`, in the slot it already held. The resolution order is
  unchanged in shape (env, badge, chat); only the chat half got honest.
- **The guide's ENVIRONMENT section** now states the grain: the actor is
  re-derived at every fire; the badge outranks it; and where the transcript
  cannot answer, the recorded row stands and *that* row is SessionStart-grained
  — the model the session started on, not necessarily the one writing now.

`src/main.rs` is unchanged. SessionStart still seeds the row, which is right:
it is the answer before any assistant entry exists.

## the instrument

`the_chat_actor_row_refreshes_from_the_transcript_at_every_fire` in
`tests/basic.rs`. Suite green: **143 passed; 0 failed** in `tests/basic.rs`,
and 151 across all six binaries, read off the raw `test result:` lines.

It carries: the pure core on the switch shape; each filter on its measured
counter-example; the truncation rule pinned with a fragment that *parses* (a
half-line usually parses to nothing and would be skipped anyway, so only a
surviving shape actually pins the rule); the defect measured on the derivation
itself; all three cannot-answer roads keeping the recorded row; a transcript
twice the window read from its end; the end-to-end injection through the
spawned binary with and without a transcript in the payload; the badge
outranking it; and the guide's grain statement.

Each half probed against the regression it guards:

| removed | what fails |
|---|---|
| the `refreshed_chat_actor` call at the hook | the incident verbatim: `QUARRY_ACTOR='claude-fable-5'` where opus is right |
| the `isSidechain` filter | `Some("claude-opus-5")` where the chat's own `claude-fable-5` is right |
| the `<synthetic>` filter | `Some("<synthetic>")` adopted as a model name |
| the truncation drop | a fragment read as an entry |
| the write-back | the row on disk keeps lying after a correct read |
| the tail seek | a transcript past the window answers `None` |
| the badge-then-chat order | **both** this instrument and `cl-dqt4`'s `the_session_hook_injects_the_badge_model_over_the_inherited_chat_actor` |

That last row is the one worth noting: inverting the precedence breaks the
neighbouring vein's instrument too, which is the cheapest possible evidence
that the two mechanisms are wired to each other rather than merely adjacent.

## affirms, and what grounds them

`cl-dqt4` was re-verified directly — read in full, probed against its own
instrument, and its "the harness has nothing to give" paragraph re-confirmed
(the *payload* carries no model; the subagent transcript exists but is not what
the hook is handed, so the claim stands as written).

The other 35 stale refs on `src/coord.rs`, `src/teach.rs` and `tests/basic.rs`
were affirmed **scoped to those files**, grounded mechanically rather than by
assertion: `git diff --numstat` shows `tests/basic.rs` at **280 added, 0
removed** (a pure append — every existing instrument stands byte-identical),
`src/coord.rs` at **0 removed**, and `src/teach.rs` at **exactly one line
removed**, the `chat_actor` call named above. Nothing any of those claims
describes was touched, and the suite is green. Behind on my three files: 127 →
90 total, 0 remaining on the files this arc wrote.

The remaining 90 belong to `src/main.rs`, `src/ops.rs`, `src/queries.rs`,
`src/render.rs` and `src/view.rs` — the dispatcher's own day of work, not this
arc's, and deliberately left alone.

## landing homework for the dispatcher

- **`q init --claude`.** This arc edited `src/teach.rs`, so
  `.claude/skills/quarry/SKILL.md` is drifted against `GUIDE`. The regen writes
  outside this arc's lease, and landing is the dispatcher's seat.
- **The guide paragraph rides the dc-vzvf flag channel.** The new ENVIRONMENT
  text is composed register prose; per dc-vzvf it is presented for
  ratify-or-amend at harvest rather than settled here. It is in `teach.rs`
  beside the text it extends, not in `framings.rs`, following cl-stkv's
  precedent for the same situation.

## user-owned calls:

One, and its thread is already filed.

1. **Whether the machine should read a subagent's own transcript to learn its
   model — which would turn `q dispatch --model` from the only road into an
   override.** This arc found the road while measuring something else: a
   subagent's transcript exists at
   `<projects>/<chat>/subagents/agent-<agent_id>.jsonl`, its assistant entries
   name the real model (`claude-opus-5`, 78 of 78 for this arc), and its
   sibling `.meta.json` even carries the dispatcher's `--model` argument. That
   is exactly what **it-6ekf** probes for and what **th-e5ez** frames as the
   user's fork: dc-zbxj ruled identity structural rather than discipline, and
   the `--model` flag is the one place attribution went back onto the
   discipline road. Reading the subagent transcript would close that gap
   structurally.

   Not built, deliberately. It needs the `isSidechain` filter relaxed for that
   one caller — the filter this arc just made load-bearing — and it leans
   harder on an undocumented layout than the chat-keyed read does, since it
   derives a path rather than being handed one. Taking it would also re-open a
   ruling this arc had no mandate over. The least-committal path was to measure
   it, build nothing, and carry the evidence to the threads that own it:
   **th-e5ez** and **it-6ekf**, both cited in cl-cv92's body so the backlinks
   reach their next reader.

## reflections

**The item's own framing was right, and being right made it cheap.** It named
the candidate cure, named exactly what was unproved about it (cost, and leaning
on an undocumented format), and named the honest alternative. That turned the
arc into two measurements and a small build rather than a design argument. Most
of the work here was reading files on disk, not writing code — the diff is 119
lines of `coord.rs` and one changed line elsewhere.

**The measurement I nearly skipped is the one that decided the design.**
Whether `transcript_path` inside a subagent points at the parent or at the
subagent's own file looked like trivia — the item is about chats, and the
subagent case was already solved by cl-dqt4. But the answer is what makes it
*safe* to let a subagent's fire write the chat row: had it pointed at the
subagent's own transcript, the obvious implementation would have quietly
smeared the agent's model onto its dispatcher, in the expensive direction,
undetectably. I would have shipped that. The prior arc's report asserted the
parent-transcript reading in passing without measuring it, and I would have
inherited the assumption. Rebuilding the binary to dump one hook payload cost
about four minutes.

**A surprise worth recording: the flush is racy.** Across two consecutive
fires, the current turn's assistant entry was already in the transcript once
and not the other. So the read gives the current turn's model *usually* and the
previous turn's otherwise — meaning a `/model` switch can be honoured one shell
late, self-correcting at the next fire. That is a genuinely different grain
from what I would have written down had I not looked, and it is now in the vein
rather than in my head.

**A friction, honestly reported: the affirm sweep was the hardest judgment in
the arc**, and not because it was subtle. "Affirm every claim your fix
re-verified" is clear; what is not clear is what to do about the two dozen
claims whose blob stamps drift merely because you appended to a file they share.
Affirming them risks stamping something current that isn't; leaving them
manufactures 35 false rot entries that every future brief and design wake pays
for. I resolved it by making the emptiness of the change *mechanical* — 0
deletions in two files, 1 located deletion in the third — and affirming scoped
to those files on that ground. But that reasoning lives only in this report and
in the affirm events. There may be room for the affirm surface to know the
difference between "this file changed" and "this file grew", since an
append-only diff cannot invalidate a claim about code that did not move. I did
not file that as an item; it is a shaping thought, not a defect, and it belongs
to whoever owns the sediment/rot classifier.

**One thing I chose not to build, and I am not certain I was right.** When the
refresh detects that the model moved, it corrects the row silently. It would
have been easy to say so once through the hook's `additionalContext` channel —
and a mid-session model switch is a real event with graph consequences, since
every node minted before that fire is now misattributed and th-t2fx says
nothing can re-attribute them. I kept silence because dc-dty5 makes silence the
default and the correction is structural, which is what dc-zbxj asks for. But
the session is the only party that could report the switch in its own wrap, and
it now has no way to know one happened.

**On my own attribution, since the join asked.** The badge stamped
`claude-opus-5` and my transcript agrees on all 78 assistant entries. My system
prompt names me `claude-opus-5[1m]` — the same model, with a context-window
suffix the badge does not carry. Not a mismatch worth acting on; recorded
because the join asked to be told either way, and because a suffix the graph
drops is the kind of thing that matters later when someone reads the corpus by
model.
