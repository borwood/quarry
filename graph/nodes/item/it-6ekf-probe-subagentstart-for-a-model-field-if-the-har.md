---
id: it-6ekf
type: item
title: 'the subagent model is already on disk: a sidecar keyed by the injected agent id names it'
v: 4
status: sketch
provenance: assistant
created: 2026-08-21T12:36:25Z
actor: claude-fable-5
kind: slice
acceptance:
- 'a dispatched agent''s graph writes file under its own model with nothing for the dispatcher to remember: the model is resolved from the harness record keyed by the injected agent id, --model survives only as an override, and the undocumented-layout dependency is stated where it is read'
witness:
- line: 'a dispatched agent''s graph writes file under its own model with nothing for the dispatcher to remember: the model is resolved from the harness record keyed by the injected agent id, --model survives only as an override, and the undocumented-layout dependency is stated where it is read'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
aliases:
- probe-subagentstart-for-a-model-field-if-the-har
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

The one road that could make dispatched attribution structural instead of a flag the dispatcher must remember (cl-dqt4's named residue, and the tension with dc-zbxj's identity-is-structural-never-discipline).

WHAT IS KNOWN. A PreToolUse hook firing inside a subagent carries no model of any kind — probed live from a subagent seat, 2026-08-21: session_id, transcript_path, cwd, prompt_id, permission_mode, agent_id, agent_type, effort, hook_event_name, tool_name, tool_input, tool_use_id, and nothing else. Only SessionStart is documented to carry model, and a subagent never fires SessionStart.

THE ROAD IS SHORTER THAN THIS ITEM WAS FILED FOR — verified from the dispatcher seat at the it-j4tx landing, 2026-08-21, by reading the real bytes rather than probing an event. A subagent's own transcript and a metadata sidecar already exist on disk, under the parent chat's directory:

  <projects>/<chat>/subagents/agent-<agent-id>.jsonl
  <projects>/<chat>/subagents/agent-<agent-id>.meta.json

The sidecar for this session's own arc reads, in full:
  {"agentType":"general-purpose","description":"Fix stale chat-actor row","toolUseId":"toolu_01VAgkNS7B9Fi2hAWyXfK53W","spawnDepth":1,"model":"opus"}

So the model is on disk, keyed by the agent id — and that id is exactly what the session hook already injects as QUARRY_AGENT. Nothing needs to arrive in a payload. The .jsonl beside it carries the RESOLVED id (78 of 78 assistant entries read claude-opus-5 for that arc) where the sidecar carries the dispatcher's alias as typed (opus), so both spellings are available and the resolved one is the better source.

WHAT THIS CHANGES. cl-cv92 already reads a transcript at hook time and measured the cost (404-517 microseconds against a ten-second budget), so the machinery and the performance evidence both exist; this is the same move one directory over, keyed on an id already in hand. Attribution would stop depending on the dispatcher remembering --model, which is what dc-zbxj asks for and what th-e5ez frames as the user's fork: with this, --model becomes an override rather than the only road.

WHAT IS STILL UNKNOWN, and why this is not yet a bug. The layout is undocumented and harness-internal — the same class of dependency cl-cv92 already took on knowingly, but taken twice is a real exposure, and one worth stating plainly wherever it is read. cl-cv92's own sidechain filter would need relaxing for the subagent file, since every entry there carries isSidechain true. And SubagentStart's payload is still unread; it may carry the model directly, which would be cheaper than any file read. That probe still wants running, and it still needs a .claude/settings.json edit, which is the user's own harness configuration and no agent's to change unasked.

Neighbourhood: cl-dqt4 for the flag this would demote to an override, cl-cv92 for the transcript-reading machinery and its measurements, th-e5ez for the user's fork, and th-t2fx for the nodes already mis-filed that no verb can re-attribute.
