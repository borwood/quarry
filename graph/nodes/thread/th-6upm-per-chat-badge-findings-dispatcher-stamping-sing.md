---
id: th-6upm
type: thread
title: 'per-chat badge findings: dispatcher stamping, single-badge chats, and the five holes'
v: 3
status: resolved
provenance: user
created: 2026-08-13T08:29:59Z
actor: claude
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Opened 2026-08-13 by the user as the collective triage thread for the do-75ut findings - the pattern th-dy25 makes policy, applied to its motivating instance. To be pieced apart into items; nothing here is settled by collection.

The defect (user-named, the sharpest): the dispatching chat holds the badge under its own chat key, so the dispatcher's UNRELATED mid-flight acts stamp into the dispatch trace. The entire point of dispatch stamping is that the dispatched work stamps as that work unit - under dc-ydvb the dispatcher is definitionally not working on it. The dispatcher's harvest-adjacent acts are discoverable as related and can be auto-associated without riding the badge. Direction to examine: split "this chat is mid-dispatch" (which the boundary guard and the dispatch refusal genuinely need from held entries) from "this act belongs to the dispatch" (stamping) - stamping resolution should follow the WORK (env badge, learned acting association), never the holding chat. it-9p3v was closed done on the per-chat landing; this thread carries what that close left open.

The single-badge chat (surfaced by the user's question, verified in code): held state is one entry per chat key, and a second dispatch of a different item from the same chat refuses ("parallel dispatch belongs to parallel chats"). But the ruled normal shape (dc-ydvb: "are these parallelizable items? fire them all off") is ONE chat firing several agents - a steward or a hot decisions session wants several badges in flight. With dispatcher stamping removed per the defect above, multi-badge held state loses its resolution ambiguity: acting chats resolve their own badges; the holder resolves none.

The five holes from do-75ut, with the user's reactions to piece apart:
1. Ordering-dependent observation: the chat-to-badge association is learned at the agent's first badged q act (note_acting_chat fires on log_event when QUARRY_DISPATCH and QUARRY_CHAT are both present); file writes before that act resolve no badge. User direction: gate the first FILE write on a prior q act under the badge - or find a better association channel not yet considered (candidates to examine: the hook injecting QUARRY_DISPATCH once an association exists; declaring the agent chat at dispatch time so the association is constructed, not learned).
2. Manual export: the payload instructs the agent to set QUARRY_DISPATCH in each shell it opens; nothing propagates it automatically. User question: manual in what sense - can the transport be structural (hook-carried) instead of instructional?
3. An agent chat with no badged act yet can run wrap - the guard cannot tell it is an agent. User: understood and bad.
4. Two hook-uncovered shells sharing one session name collide on the session fallback key. User: understood and bad.
5. The same item dispatched from two chats of one session double-holds with first-found drift cursors. User direction: prevent dispatching an already-dispatched item outright, with an explicit override (the steal pattern: loud, reasoned, logged).

State 2026-08-13, first topic RULED (dc-zbxj, hashed out in this thread's conversation): join-as-fetch settles the defect, hole 1, hole 2, and hole 3 by construction - dispatch mints a single-use token, the spawn prompt is one line, q join binds the hook-provided agent identity to the badge and renders the brief fresh; hooks inject identity only (QUARRY_AGENT; probed: subagent shells are otherwise indistinguishable from the parent chat); the join gate denies unassociated file writes into a dispatched lease with a teaching line; stamping follows the work, never the holding chat. Build shaped as it-vkxh; landing it also closes it-pdnq and hole 4 shrinks to out-of-hook-coverage shells only. STILL OPEN on this thread, one topic each: multi-badge held state, and hole 5 (double-dispatch refusal with a loud override); triage of any remainder after those.
