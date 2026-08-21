---
id: cl-wjdb
type: claim
title: '`structural-attribution`: a dispatched arc files under the model that wrote it with nothing for the dispatcher to remember'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T13:45:02Z
actor: claude-opus-5
kind: feature
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/teach.rs
  at: '001106469205'
- rel: supports
  to: it-6ekf
  at: 6
- rel: about
  to: ar-xa38
  at: 1
---

`structural-attribution`: a dispatched arc files under the model that wrote it with nothing for the dispatcher to remember

The capability it-6ekf landed, and the closing of the residue cl-dqt4 named in its own last paragraph. Before this, a dispatched agent's graph writes filed under the model its dispatcher typed into q dispatch --model, or — if the dispatcher forgot — under the dispatching chat's own model, silently and with nothing in the harness able to see the mismatch. Now the session hook resolves a subagent's model from the harness's own record of that subagent, keyed by the agent id it already injects (cl-jp4q), so the correct answer arrives whether or not anyone said anything.

WHAT AN OPERATOR SEES. q dispatch --model still works and still wins where it is given: it is now an override for the case where the dispatcher knows better or the harness layout is absent, not the mechanism attribution depends on. A fire with no --model no longer silently mis-files: the agent's own shells carry its own model from its first hook fire onward. The dispatching chat's own shells are untouched throughout — the lookup is keyed on the agent id, which the harness gives only to a subagent — and a spawn that genuinely inherits its chat's model still resolves to exactly that, now via the agent's own record rather than by assumption.

WHAT IT COSTS AND WHAT IT LEANS ON. One tail read per subagent fire, in place of the chat read it replaces; measured medians across all four roads sit within 11.4-12.0 ms of whole-hook cost against a 10-second budget (cl-jp4q carries the method). It leans on an undocumented, harness-internal directory layout — the second such dependency in this machine after cl-cv92's — which is stated in the code that reads it and in the guide's ENVIRONMENT section rather than left to be discovered. Every failure to read degrades to the previous answer; none of them errors.

WHAT IS NOT CLOSED. th-e5ez asks the user whether q dispatch should refuse without --model; this landing is the evidence its own last paragraph asked for, and the case for a gate is weaker now, but settling stays the user's. Two statements at the dispatcher's seats still describe the old world in the present tense — the fire's no-model attribution line and the join's inherited-model line, both in src/main.rs, both outside this arc's lease — and both now read as false; filed as a defect at landing. th-t2fx's gap is untouched: nothing re-attributes a node already filed under a wrong model.
