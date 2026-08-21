---
id: it-6ekf
type: item
title: 'probe SubagentStart for a model field: if the harness names a subagent''s model, per-agent attribution stops being a flag'
v: 1
status: sketch
provenance: assistant
created: 2026-08-21T12:36:25Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

The one road that could make dispatched attribution structural instead of a flag the dispatcher must remember (cl-dqt4's named residue, and the tension with dc-zbxj's identity-is-structural-never-discipline).

WHAT IS KNOWN. A PreToolUse hook firing inside a subagent carries no model of any kind -- probed live from a subagent seat, 2026-08-21: session_id, transcript_path, cwd, prompt_id, permission_mode, agent_id, agent_type, effort, hook_event_name, tool_name, tool_input, tool_use_id, and nothing else. Only SessionStart is documented to carry `model`, and a subagent never fires SessionStart.

WHAT IS NOT KNOWN. The harness has a SubagentStart event that DOES fire for a subagent and does carry agent_type and agent_id. Its documented input schema names no model -- but that schema is itself flagged as needing verification upstream (anthropics/claude-code issue 19170), so absence from the docs is not absence from the payload. Nobody has looked at the real bytes.

THE PROBE. Wire a SubagentStart hook that dumps its raw stdin, spawn one subagent on a model different from the parent's, read the dump. This arc could not run it: it requires editing .claude/settings.json, which sat outside the arc's write-set and is the user's own harness configuration.

IF THE MODEL IS THERE. A SubagentStart arm records agent_id -> model exactly as HookCmd::Orient records chat_id -> model today, coord::badge_model reads the per-agent row ahead of the badge stamp, and --model becomes an override rather than the only road. Attribution then costs the dispatcher nothing and cannot be forgotten, which is what dc-zbxj asked for.

IF IT IS NOT. The badge stamp stands as built and the discipline residue is real; the thread on whether q dispatch should REQUIRE --model is where that goes.
