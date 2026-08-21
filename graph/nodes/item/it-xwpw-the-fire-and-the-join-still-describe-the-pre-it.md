---
id: it-xwpw
type: item
title: 'the fire and the join still describe the pre-it-6ekf world: two attribution statements now read as false'
v: 3
status: sketch
provenance: assistant
created: 2026-08-21T13:45:36Z
actor: claude-opus-5
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: it-6ekf
  at: 6
- rel: about
  to: ar-xa38
  at: 1
---

THE DEFECT. it-6ekf made a subagent's model resolve structurally from the harness's own record (cl-jp4q), so an UNSTAMPED dispatch no longer files under the dispatching chat's model. Two statements written for the old world still say it does, in the present tense, at the two seats an operator and an agent actually read.

REPRO, both from a fire with no --model:
  1. q dispatch <item> (no --model) prints, from src/main.rs (~line 2196):
     "attribution: no --model given, so this arc's graph writes file under <chat actor> — this chat's own model, inherited. Right only if the spawn inherits it too; on any other model every node the agent mints files under a model that did not write it..."
     Every clause after the em dash is now false: the agent's shells resolve its OWN model from its first hook fire.
  2. q join <token> then prints, from src/main.rs (~line 2231):
     "Those acts file under <chat actor>, inherited from the dispatching chat because the harness tells a subagent's hooks nothing about its own model..."
     Both halves are false — the harness does tell, one directory over, and the acts do not file under the chat.

Also stale beside them: the doc comments on ops::DispatchOut.arc_actor and ops::JoinOut.arc_actor, which both say "the dispatching chat's own actor inherited" as the no-stamp answer.

WHY IT WAS NOT FIXED IN THE ARC THAT CAUSED IT. The it-6ekf lease covered src/coord.rs, src/teach.rs and tests/** — src/main.rs and src/ops.rs sat outside it, and an agent may not extend its own lease.

THE SHAPE OF THE FIX, and it is not only wording. The FIRE genuinely cannot know: the agent does not exist yet, so there is no agent id and no record — its line should say the model will be resolved from the harness record at the agent's own first fire, and that --model overrides it. The JOIN can do better than wording: it runs INSIDE the agent with QUARRY_AGENT injected, so coord::agent_model can resolve the true model there and the join can state the real answer rather than a prediction. That would also close the one grain cl-jp4q names — a first fire that lands before the agent's first turn reaches disk and falls back to the coarse sidecar alias — since by the time the join line is composed the arc has a turn on disk.
