---
id: th-e5ez
type: thread
title: should q dispatch require --model? the attribution stamp is discipline, and dc-zbxj ruled identity structural
v: 1
status: open
provenance: assistant
created: 2026-08-21T12:36:43Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
---

THE CALL. dc-zbxj is a user ruling and its second half reads "identity is structural, never discipline": hooks do identity injection only, the per-shell export died, nothing depends on an agent or a dispatcher remembering to say who they are. The cure this arc landed (cl-dqt4) puts ATTRIBUTION on the discipline road it just took identity off: q dispatch --model <model> is a flag the dispatcher must remember on every fire, and omitting it while spawning on another model reproduces the original defect silently.

WHY IT WAS BUILT THAT WAY ANYWAY. The harness leaves no structural road. A PreToolUse firing inside a subagent carries no model of any kind (measured, cl-dqt4), and SessionStart -- the one event that does -- never fires for a subagent. The dispatcher is the only party in the system that knows which model it spawned on. The item's own shaping preferred the badge stamp for exactly this reason, so the flag is what the design seat asked for; what was NOT settled there is whether the flag should be optional.

THE FORK. Should q dispatch REFUSE without --model?

  FOR: it converts silence into a refusal at the one station that can fix it -- the cl-74qt / dc-p6z4 pattern this machine already uses at the fire (no item fires without a stated contract; no lease takes a malformed glob). Silence is the failure mode the whole item exists to kill.

  AGAINST: an inheriting spawn (a fork, an agent type with no model override) genuinely runs the dispatching chat's model, and today's inheritance is CORRECT for it -- so a refusal would demand a ritual for the common case where nothing is wrong. It also changes the dispatcher's own workflow, and dc-zbxj's one-line hand-off was shaped around bulk dispatch being N cheap fires.

WHAT WAS BUILT PROVISIONALLY, pending this call: no gate. The fire STATES which of the two answers the dispatcher got, above the spawn prompt, every time -- the stamped model, or the inherited one with the re-fire command in hand. The join states it again to the agent, which is the one mind told its own model and can report a mismatch. Two statements, no refusal.

RELATED, AND IT MAY DISSOLVE THE CALL ENTIRELY: it-6ekf probes SubagentStart for a model field. If the harness names a subagent's model there, the per-agent row can be recorded exactly as SessionStart records the chat's, --model becomes an override rather than the only road, and dc-zbxj gets what it asked for with no discipline at all.
