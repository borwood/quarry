---
id: it-xcvb
type: item
title: 'dispatched work is attributed to the dispatcher''s model: a subagent''s actor is inherited, never recorded'
v: 3
status: sketch
provenance: assistant
created: 2026-08-21T09:02:19Z
actor: claude-fable-5
kind: bug
acceptance:
- 'a node''s recorded actor names the model that actually wrote it: work done by a dispatched subagent whose model differs from its dispatcher''s is never filed under the dispatcher''s model, and the commit convention can be honoured from the graph alone'
witness:
- line: 'a node''s recorded actor names the model that actually wrote it: work done by a dispatched subagent whose model differs from its dispatcher''s is never filed under the dispatcher''s model, and the commit convention can be honoured from the graph alone'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

Witnessed from the dispatcher seat, 2026-08-21, across both it-rmqy arcs. Both arcs were spawned on Opus 5 by explicit user instruction; every graph node they minted — cl-kr7f, it-x4bb, it-rzqv, and their affirms — is recorded 'assistant · claude-fable-5', the dispatching chat's model.

The mechanism: coord::record_chat_actor writes graph/.chat-actors.json keyed by CHAT id at SessionStart, and the PreToolUse hook injects that value as QUARRY_ACTOR. A subagent is not a chat and never fires SessionStart, so it inherits its parent chat's entry. The map read at landing carries eighteen entries, every one claude-fable-5, with no subagent row at all — despite two Opus agents doing the day's code and graph work. QUARRY_AGENT is injected for subagents (dc-zbxj), so a per-subagent IDENTITY already reaches q; only the MODEL is missing.

Why it is a defect and not cosmetic. The house commit convention names the model that did the work, and the graph is the only record of that — it cannot currently support the convention for any dispatched arc. Attribution is also the ground for reading the corpus later (which model produced which claim, how a model's veins held up at assay), and every dispatched landing silently files under the wrong name. The dispatcher's model choice is a deliberate act; the record erases it.

The open question this cannot answer from the dispatcher seat: whether the harness exposes a subagent's model to a hook at all. If it does, the cure is a per-agent row keyed on QUARRY_AGENT, read before the chat row. If it does not, the fork is real and belongs to design — dispatch could stamp the model it spawned with into the badge (the dispatcher knows it; it is an argument it passes), or the actor could stay chat-grained with the limitation documented and the commit convention amended to match. Prefer the badge stamp: the dispatcher already mints the badge and already knows the model, and it needs nothing from the harness.
