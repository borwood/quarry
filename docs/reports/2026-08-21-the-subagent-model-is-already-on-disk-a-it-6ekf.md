# dispatch report — it-6ekf: the subagent model is already on disk

**Arc:** it-6ekf · badge agent:a63385e55bdf536e2 · 2026-08-21
**Model:** claude-opus-5 (the badge stamp matched: the join said claude-opus-5 and that is what I am).
**Result:** the acceptance line is met. Landing belongs to the dispatcher — `q harvest it-6ekf`.

## Against the acceptance line

> a dispatched agent's graph writes file under its own model with nothing for the
> dispatcher to remember: the model is resolved from the harness record keyed by
> the injected agent id, `--model` survives only as an override, and the
> undocumented-layout dependency is stated where it is read

All four clauses hold, each pinned and each probed against the regression it guards.

**Resolved from the harness record, keyed by the injected agent id.**
`coord::agent_model(transcript_path, agent_id)` derives
`<chat-transcript minus .jsonl>/subagents/agent-<id>.{jsonl,meta.json}` and answers
from them. `teach::session_hook_output` spends it between the badge stamp and the
refreshed chat row. Nothing new is stored; nothing has to arrive in a payload.

**Nothing for the dispatcher to remember.** Confirmed end to end through the
spawned binary on an UNSTAMPED dispatch: the agent's shell gets its own model, its
dispatcher's shell keeps the chat's, and the two are different values.

**`--model` survives only as an override.** The resolution order is env →
`badge_actor` → `agent_actor` → refreshed chat row. `badge_model` is agent-keyed and
every subagent has a harness record, so putting the record first would make the flag
unreachable rather than optional. Pinned: with a stamp the shell gets the stamp;
clear the badge and the same shell falls to the agent's own model — where before
this arc it fell all the way back to the dispatcher's.

**Stated where it is read.** The `coord::subagent_records` / `sidecar_model` /
`agent_model` doc comments carry it, and the guide's ENVIRONMENT section — the one
place the injected actor is explained to an agent — now names the layout, says
`--model` is an override, and says **UNDOCUMENTED AND HARNESS-INTERNAL** in those
words. Pinned by assert on `teach::GUIDE`.

## Measurements, with their methods

Every number below was taken on this machine on 2026-08-21. The survey ran over
**268 subagent records** across all 14 project directories under
`~/.claude/projects/*/*/subagents/` — every `.meta.json` paired with its `.jsonl`,
parsed line by line in Python, counting assistant entries with a real
`message.model`.

| finding | measurement | what it decided |
|---|---|---|
| the sidechain filter must invert | 261 of 268 transcripts hold a readable assistant entry; **all** of their entries carry `isSidechain: true`, with **zero** non-sidechain entries anywhere among them | cl-cv92's filter is fatal over a subagent's own file — it is now a parameter, not a constant |
| the sidecar alias cannot be resolved by table | `opus` → `claude-opus-5` in 128 arcs, `claude-opus-4-8` in 43 | the sidecar is a fallback, never the source; the transcript is the only road to the resolved name |
| a sidecar's silence is not inheritance | of the 88 records with no `model` key, **9 ran on a model their parent chat was not running** — a `claude-code-guide` arc on haiku under a fable chat, four depth-2 `Explore` arcs on opus under fable chats, four arcs whose chat switched model after the spawn | gating the lookup on the sidecar would have missed one unstamped subagent in ten; the transcript leads |
| the model never moves mid-arc | first assistant model == last assistant model in 268 of 268 | reading the tail is as true as reading the head, and far cheaper |
| the transcript is written live | this arc's own file held 57 entries / 423 KB mid-session, growing between reads | the transcript road is reachable at hook time at all, which is the premise of the whole design |

**Cost**, measured in the real `q hook session` process against the real files
(n=20 per road, medians, temp store, `QUARRY_ACTOR` cleared so the hook actually
resolves):

| road | transcript read | median | min–max |
|---|---|---|---|
| chat (no agent id) | 2.48 MB chat file | 12.0 ms | 10.9–22.4 |
| agent (real agent id) | 0.79 MB subagent file | 11.6 ms | 10.8–13.0 |
| agent, no record on disk | 2 failed opens, then the chat file | 11.4 ms | 11.0–21.8 |
| no transcript at all | none | 11.4 ms | 10.3–12.4 |

The agent road is a **substitution** for the chat read, not an addition, and on real
data it is the cheaper of the two because a subagent's transcript is smaller than
its chat's. Every delta sits inside run-to-run spread, against a 10-second hook
budget. This does not isolate the read from process startup — cl-cv92 did that for
the same code path and got 404–517 µs — and I did not re-isolate it, because the
substitution makes the whole-hook comparison the load-bearing one.

**Live confirmation on real harness data.** The real binary, handed this repo's own
chat transcript and agent `ab979fc19cd654d43` — a `claude-code-guide` arc at
spawnDepth 2 whose sidecar carries **no model key at all** — injects
`QUARRY_ACTOR=claude-haiku-4-5-20251001`, where the chat road says
`claude-opus-5`. That is the exact class the sidecar cannot see and the chat road
gets wrong, resolved correctly.

## The road was shorter than the item was filed for, and shorter again

The item was filed to probe `SubagentStart` for a model field. It did not need to:
the dispatcher's own note in the brief had already found the files. What the brief
did not settle, and what this arc measured, is that **the transcript must lead the
sidecar** — the brief leaned toward the transcript for spelling reasons, but the
decisive reason turned out to be coverage (the 9-in-88 finding above), which no
amount of reasoning would have produced.

`SubagentStart` remains unprobed. It needs a `.claude/settings.json` edit, which is
the user's own harness configuration and no agent's to make unasked. If it carries
the model it is a cheaper and better-documented road than either file — see
th-t842 option (c).

## Changes

- `src/coord.rs` — `scan_last_assistant_model` (the old `last_assistant_model` body
  with the sidechain judgment as a parameter) behind two faces,
  `last_assistant_model` (chat) and `last_agent_model` (agent); `tail_model` behind
  `transcript_model` and `agent_transcript_model`; new `subagent_records`,
  `sidecar_model`, `agent_model`, `agent_actor`.
- `src/teach.rs` — the hook's actor resolution gains one `.or_else` between the
  badge and the chat row; the guide's ENVIRONMENT section gains a paragraph.
- `tests/basic.rs` — `a_subagents_model_resolves_from_the_harness_record_keyed_by_its_agent_id`.

Full suite green: 144 + 2 + 1 + 1 + 1 + 3 = **152 passed, 0 failed** (read off the
raw `test result:` lines, not through a filter).

**Probed against the regression each half guards** — each of these was applied,
run, and reverted:

| reverting… | fails |
|---|---|
| the hook's `.or_else(agent_actor)` | the unstamped-subagent assert, with the chat's model still in the prefix |
| the relaxed sidechain filter | the native-mark assert (`last_agent_model` answers `None`) |
| the sidecar fallback | the pre-first-turn assert |
| by-name strip → `with_extension("")` | the suffixless-path assert |
| badge/record precedence, inverted | the override assert |

Note on that last one: **cl-dqt4's own instrument does not catch the inversion**,
because it runs with no harness record on disk. The new test is the only thing
holding that order. My first cut of two of these asserts was non-discriminating
(the truncation fixture and the dotted-path fixture both passed under the mutation);
both were sharpened until the mutation actually failed them. Probing is the only
reason I know the others hold.

## Graph writes

- **cl-jp4q** (vein) — `subagent-model-record`: the mechanism, its measurements, the
  filter inversion, the precedence, the degradations. `supports it-6ekf`.
- **cl-wjdb** (feature) — `structural-attribution`: the receipt for the capability.
  `supports it-6ekf`.
- **it-xwpw** (bug) — the defect this landing creates outside its own lease, below.
- **th-t842** (thread) — the user-owned call, below.

Both claims cite th-e5ez and cl-cv92 in body, so the backlinks carry this evidence
to their next readers.

**Mentioned, not linked — an edge I judged not mine.** cl-jp4q arguably *refutes* one
paragraph of cl-dqt4: "THE HARNESS HAS NOTHING TO GIVE … the per-agent row keyed on
QUARRY_AGENT, the cure that would have asked nothing of the dispatcher, cannot be
written: nothing at the agent's seat knows the answer." Something at the agent's
seat now does. `refutes` is a settling-shaped rel and dc-ez67 leaves me
depends-on / about / source / supports, so I named it here instead of linking it.
The rest of cl-dqt4 stands unharmed — the stamp is still the override.

## Out of lease, owed at landing

1. **`.claude/skills/quarry/SKILL.md` is stale.** It is generated from
   `teach::GUIDE` (`teach::install_claude`) and I edited GUIDE, but the file sits
   outside the lease. Per CLAUDE.md the landing seat runs `q init --claude` after a
   `src/teach.rs` guide edit — it is owed and I did not do it.
2. **it-xwpw** — the fire's and the join's attribution lines in `src/main.rs`, plus
   the `arc_actor` doc comments in `src/ops.rs`, still describe the pre-it-6ekf
   world in the present tense and now read as false. Filed as a bug on sight, with
   the repro and the shape of the fix. Worth noting the fix is not only wording:
   **the join can do better than a prediction** — it runs inside the agent with
   `QUARRY_AGENT` injected, so `coord::agent_model` resolves the real answer there,
   which would also close the first-fire alias grain cl-jp4q names.
3. `q view` regeneration after these graph writes — the dispatcher's seat.

## user-owned calls:

Two calls this work met that belong to the user. Both are filed; neither is settled
in code.

- **Accepting a second undocumented harness-internal dependency — th-t842.**
  cl-cv92's own closing words reserved this exact fork for the user: reading a
  subagent's transcript "leans harder on an undocumented layout, so the fork belongs
  to the user." it-6ekf was fired to build it and the RETURN spec named the
  dependency, so I built it and stated the exposure at both seats that read it —
  that is the least-committal path consistent with the fire. What remains the user's
  is what the graph now *accepts*: whether this becomes a `watch` item with a named
  trigger (CLAUDE.md reserves `watch` for exactly this shape — accepted compromises
  with a named trigger), whether to bound it back to the chat transcript alone, or
  whether to probe `SubagentStart` first and take a documented road instead. I filed
  no watch item: the trigger is the user's to name, and a watch without one is the
  thing CLAUDE.md rules against.

- **Whether `q dispatch` should refuse without `--model` — th-e5ez, untouched.**
  th-e5ez's own last paragraph said this item "may dissolve the call entirely," and
  the evidence says it largely does: an unstamped fire no longer mis-attributes, so
  the FOR argument ("it converts silence into a refusal") loses its silence. The
  AGAINST argument gets *stronger* in one direction and weaker in another — the
  9-in-88 finding shows the "inheriting spawn is correct today" premise was
  measurably false about a tenth of the time, but the new road fixes those cases
  without any ritual. **Provisional path taken: no gate, no change to the fire's
  refusal behaviour, and the badge stamp keeps precedence** so the flag still means
  exactly what it meant. Settling is the user's; I added evidence via the claim
  backlinks and left the thread open.

## Reflections

**The measurement changed the design twice, and reasoning would not have.** I came
in intending the sidecar as the primary source — it exists from spawn, it is small,
it is explicitly "the spawn record." Two measurements killed that: the alias is
ambiguous across time (`opus` → two different resolved ids), and 9 of the 88
model-less sidecars belonged to agents running a model their chat wasn't. I had
constructed an argument for gating the whole lookup on the sidecar's `model` key,
and it was simply wrong. The 268-record survey cost about four minutes and is the
only reason this landed correctly.

**The thing I am least sure of is the first-fire fallback.** I could not measure it.
Whether the harness flushes an arc's first assistant entry before that arc's first
PreToolUse hook fires is a race I have no instrument for from inside a subagent —
cl-cv92 measured the analogous race on the parent transcript and got it *both ways*
across two consecutive fires, which is why I kept the sidecar road at all. If that
race never actually loses, the sidecar branch is dead code carrying a
`claude:opus`-shaped wart, and the honest thing would be to delete it. If it does
lose, deleting it would put the very first shell of every unstamped arc back on the
dispatcher's model. I chose the branch that is never wrong-in-family over the branch
that is sometimes wrong outright, and I would like the next person who can measure
it to actually measure it.

**A coarse actor spelling is now reachable.** `claude:opus` can enter the graph's
actor field, where every other road produces `claude-opus-5`. I think this is
correct — family-correct beats the dispatcher's model — but it is a new *kind* of
value in a field the commit convention reads, and it is worth someone deciding
deliberately rather than discovering it in a log. Related and unfixed: th-t2fx's gap,
that nothing re-attributes a node after the fact, so a node minted under a coarse
alias stays coarse forever.

**Landing a mechanism whose own explanation sits outside the lease is uncomfortable.**
The write-set was exactly right for the *code* — coord and teach are where the
mechanism belongs — but the two sentences an operator reads about attribution live
in `main.rs`, and the guide-derived SKILL.md lives in `.claude/`. So the arc
correctly refused to touch them and correctly made them false. it-xwpw exists so
that is not silent, but the pattern is worth noticing: a lease scoped to the
mechanism will systematically exclude the prose that describes it. If write-sets are
authored from "what code changes," the statements about that code drift by
construction.

**One small thing that went well.** Probing each half against the regression it
guards caught two of my own asserts being decorative rather than discriminating.
Both looked fine and passed; neither would have failed if the code had been wrong.
The discipline is worth more than it costs — I would not have found either by
reading.
