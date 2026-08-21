---
id: th-t842
type: thread
title: 'the second undocumented harness dependency is now load-bearing: accept it as a watch with a named trigger, or hold attribution to what the payload documents'
v: 4
status: queued
provenance: assistant
created: 2026-08-21T13:46:33Z
actor: claude-opus-5
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: it-6ekf
  at: 6
---

THE CALL, and cl-cv92 already named it the user's in its own closing words: reading a subagent's own transcript "leans harder on an undocumented layout, so the fork belongs to the user". it-6ekf was fired to build exactly that, and cl-jp4q landed it — so the build is done and this thread is about what the graph now accepts, not about whether to write the code.

WHAT IS NOW LEANED ON. Two harness-internal facts, neither documented anywhere the harness commits to:
  1. cl-cv92's, already standing: a chat transcript is JSONL whose assistant entries carry message.model, and the hook payload's transcript_path points at it.
  2. cl-jp4q's, new: beside that file sits <chat>/subagents/agent-<id>.jsonl and agent-<id>.meta.json, keyed by the agent id the hook injects; every entry in the former carries isSidechain:true; the latter's "model" key is the spawn alias.

WHY IT IS NOT SIMPLY A BUG WAITING. Every read is best-effort and every miss degrades to the previous answer — a layout change makes attribution go back to being SessionStart-grained and chat-inherited, which is where it was before it-j4tx and it-xcvb. Nothing errors, no shell fails, no verb refuses. That is the shape CLAUDE.md reserves the `watch` kind for: an accepted compromise, which wants a NAMED TRIGGER rather than a vague intention to notice.

THE FORK.
  (a) Accept it: file a watch item with a trigger the machine can actually meet — e.g. "QUARRY_ACTOR resolves to the dispatching chat's model on a dispatch known to have been spawned on another", or a periodic re-run of the 268-record survey cl-jp4q measured, which would go silent the moment the layout moved.
  (b) Bound it: keep the chat-transcript road (cl-cv92) and give up the subagent road, returning --model to being the only structural answer and re-opening th-e5ez's gate question in earnest.
  (c) Ask for the documented road instead: SubagentStart's payload is still unread on this machine, and it may carry the model directly — cheaper than any file read and a documented channel if it does. That probe needs a .claude/settings.json edit, which is the user's own harness configuration and no agent's to make unasked. it-6ekf's own brief named this as still-unknown and it stayed unknown: this arc took the on-disk road because it needed no configuration change.

WHAT THIS ARC DID PROVISIONALLY: built it, and stated the exposure at both seats that read it (the coord doc comments and the guide's ENVIRONMENT section) rather than leaving it to be discovered. No watch item filed — the trigger is the user's to name.
