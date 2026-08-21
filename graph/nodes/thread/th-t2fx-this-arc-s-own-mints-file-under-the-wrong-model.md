---
id: th-t2fx
type: thread
title: this arc's own mints file under the wrong model, and nothing can re-attribute a node after the fact
v: 1
status: open
provenance: assistant
created: 2026-08-21T12:37:05Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
---

THE CALL, met on this arc's own seat. it-xcvb was fired before --model existed, so this arc's badge carries no stamp and every node it minted -- cl-dqt4, it-j4tx, it-6ekf, th-e5ez, this thread, and the report doc -- is stamped actor: claude-fable-5. The agent that wrote them is claude-opus-5[1m]. The fix landed one fire too late for its own record, and it compounds with it-j4tx: claude-fable-5 is not even the dispatching chat's live model, it is that chat's stale SessionStart row.

WHAT WOULD HAVE MADE THEM RIGHT, and why it was not done. The session hook injects QUARRY_ACTOR as a command prefix, so a later `export QUARRY_ACTOR=claude-opus-5` in the same command wins, and every mint would have carried the true model. That was not done: CLAUDE.md says in the user's own words "QUARRY_ACTOR is auto-injected by the session hook -- don't set it by hand. Export it manually only when working from outside hook coverage", and C3 says a fork that would overturn a user ruling goes to a thread rather than into the work. The least-committal path consistent with the standing rulings was to mint under the injected value and surface it here.

THE CALL ITSELF, for the user: (a) leave these nodes misattributed as the last instance of the defect they document, (b) correct them by hand at harvest, or (c) rule that an agent may hand-export QUARRY_ACTOR when it knows the injected value is false -- which is the discipline road dc-zbxj rules against, in the one case where the agent's knowledge is better than the machine's.

THE GAP UNDERNEATH IT, which outlives this arc: nothing can re-attribute a node after the fact. q set carries no actor=, and the front-matter actor plus every logged event are written from Store::actor() at act time. Any of the three answers above except (a) needs a road that does not exist. Worth deciding whether one should -- a correction verb is also what an it-j4tx cure would want for the sessions already filed stale.
