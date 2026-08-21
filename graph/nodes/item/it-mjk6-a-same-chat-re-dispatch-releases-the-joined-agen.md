---
id: it-mjk6
type: item
title: 'a same-chat re-dispatch releases the joined agent and says nothing: the fire announces only the steal'
v: 1
status: sketch
provenance: assistant
created: 2026-08-21T14:40:30Z
actor: claude-opus-5
kind: bug
acceptance:
- 'the fire states the release whenever it happens: a dispatch that replaces a live arc names the agent it just cut loose, in both arms, so the dispatcher learns at the one seat that can act on it'
witness:
- line: 'the fire states the release whenever it happens: a dispatch that replaces a live arc names the agent it just cut loose, in both arms, so the dispatcher learns at the one seat that can act on it'
  by: agent:ab3c81b3dc8f06162
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Noticed 2026-08-21 landing it-jsu5 (cl-gy6q, `replacement-clear`). Both replacement
arms now release the agent they replace — the cross-chat steal and the same-chat
re-dispatch alike — but only the steal SAYS so. main.rs prints a loud
"⚠ STOLEN from <holder> … the old agent's associations cleared" line for the steal,
while a same-chat re-dispatch prints "[re-dispatch: lease kept]" and nothing about the
agent whose badge, pin and attention just died.

The gap matters most where the flow is used: the same-chat re-dispatch is the
documented RECOVERY flow, so the replaced agent is often still running. After the fire
its writes into the leased zone deny (cl-2pc9, no badge resolves) and its acts stamp
nothing — correct, and invisible to the dispatcher who caused it. The steal's own line
is the precedent for what the re-dispatch owes.

src/main.rs was outside it-jsu5's write-set, and ops::DispatchOutcome carries no field
for it: the shape is probably a `replaced: Option<String>` (the previously joined
identity, already in hand as `live.joined` at the commit point in ops::dispatch)
printed the way `stolen_from` is.
