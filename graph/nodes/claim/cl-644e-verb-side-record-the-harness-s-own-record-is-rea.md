---
id: cl-644e
type: claim
title: '`verb-side-record`: the harness''s own record is reachable from a verb — the injected chat id locates the transcript; the fire names the road, the join names the model it read'
v: 9
status: ratified
provenance: assistant
created: 2026-08-21T14:14:49Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 6147cd939364
- rel: about
  to: ar-xa38
  at: 1
- rel: source
  to: file:src/ops.rs
  at: c2084e9b6dd2
- rel: source
  to: file:src/main.rs
  at: 9cabeb54d755
- rel: source
  to: file:tests/basic.rs
  at: 75f4c2e53c44
- rel: supports
  to: cl-jp4q
  at: 5
- rel: supports
  to: cl-z6gc
  at: 6
---

`verb-side-record`: the harness's own record is reachable from a VERB, not only from a hook — the injected chat id locates the transcript; the fire names the road it will take, the join names the model it read

THE DEFECT THIS CLOSED. it-6ekf made a subagent's model resolve structurally from the harness's own record (cl-jp4q), and two statements written for the pre-it-6ekf world went on describing it in the present tense at the two seats an operator and an agent actually read. The fire's unstamped line named the dispatching chat's model as what the arc would file under and called it inherited; the join's said the same and gave a reason — "the harness tells a subagent's hooks nothing about its own model" — whose both halves had stopped being true. The join's was the worse of the two: it told the ONE mind that could check the answer a reason to stop checking.

THE TWO SEATS ARE NOT SYMMETRICAL, and that is the whole shape of the cure. The FIRE cannot know: the agent does not exist yet, so there is no agent id and no record to key on. ops::DispatchOutcome.arc_actor is therefore Option<String> — the stamped model or nothing — so the false statement is no longer representable rather than merely no longer printed. What the unstamped line states is the ROAD (the agent's own model, resolved from the harness record at its own first fire) with the override in hand, because the fire is still the one station where --model can be added.

THE JOIN CAN DO BETTER THAN WORDING, and this is the mechanism the repair extracted. coord::agent_model takes the CHAT transcript path, which a hook is handed in its payload and a verb is not — nothing injects it — so before this the record road existed only inside the hook, and q join could do no better than repeat whatever QUARRY_ACTOR that hook had already resolved. coord::locate_chat_transcript closes that: the chat id IS injected (QUARRY_CHAT, cl-z6gc) and the harness names each chat's transcript for it one project directory down, so the file is found by testing <project>/<chat>.jsonl across the project directories under CLAUDE_CONFIG_DIR (or ~/.claude). coord::find_chat_transcript_in takes the root as a parameter so the walk is measurable against a fabricated layout, and coord::agent_actor_here is the verb-side twin of agent_actor. Cost, measured over the 14 real project directories on this machine: about 1.3 ms median for the walk (n=20, an upper bound — taken through PowerShell's own file APIs, where the Rust walk makes the same syscalls without the interpreter). Once per join, and never in a hook: no hook calls this, so the 10-second hook budget is untouched by construction.

THREE ROADS, SAID APART. ops::ArcActorSource is Stamp, Record, Injected, and ops::join returns which one answered, because they read differently to the agent: the stamp is certain and outranks the record; the record is a reading of this very agent; the injected actor is a fallback that may not be the joining agent's model at all. main.rs composes one sentence per road, and the fallback names itself as one rather than dressing up as a reading. Collapsing states is how the false clause survived it-6ekf in the first place — two situations wearing one sentence.

THE JOIN EVENT MOVED WITH IT, which closes the one grain cl-jp4q named. An arc's FIRST fire is the q join shell and there is no earlier entry to fall back on the way a chat has, so agent_model there can fall to the sidecar's coarse spawn alias. arc_actor now resolves stamp, then the record read HERE, then the injected actor — and that read happens after the hook that injected this shell fired, so by the time the join line composes the shell's own turn is on disk: an unstamped arc's first badged act files under the RESOLVED model where the injected value carried the alias.

EVERY MISS DEGRADES, NEVER ERRORS. No chat id, no projects root, no matching project directory, no record for this agent, a path that is not valid UTF-8 — each returns None and the injected actor stands, which is the honest answer and is said to be one. A chat id carrying a path separator is refused before any directory is read: the id arrives from the environment, and joining it as a path component is the one way this walk could reach outside the tree it was pointed at. That guard is measured, not defensive — <project>/../escape.jsonl really is returned without it.

Pinned by a_bare_fire_names_no_model_and_the_join_names_what_it_read in tests/basic.rs: the locator over a fabricated layout (found past a non-matching project, unknown chat, traversal, empty id, missing root), the fire's two lines end to end through the spawned binary, the join's three roads end to end with a fabricated harness home under CLAUDE_CONFIG_DIR, the sidecar window and its close on the re-join, the stamp still outranking a record that disagrees, the join event filing under the record with no stamp anywhere, and the no-record degradation naming itself. Each half probed against the regression it guards: restoring the old fire line fails with the dispatching chat's model in it; restoring the old join sentence fails the fallback assert; dropping the record read at the join fails with claude-fable-5 in the join line; removing the separator guard returns the escape file.

RESIDUE, NAMED AND NOT SETTLED HERE. This adds a THIRD inference to the two th-t842 already holds for the user — the location of the projects root and the per-chat naming inside it, facts the hook payload used to supply — and it takes them from a VERB seat rather than a hook. Filed as its own thread. And cl-dqt4's "STATED AT BOTH SEATS, GATED AT NEITHER" paragraph still describes the pre-it-6ekf pair of answers at both seats; amending a ratified body is not an agent's hand, so this arc's report flags it instead.
