---
id: th-h54p
type: thread
title: 'the harness layout is now inferred from a VERB seat too: q join locates the projects root and the per-chat transcript name, a third fact under th-t842''s fork'
v: 4
status: queued
provenance: assistant
created: 2026-08-21T14:15:54Z
actor: claude-opus-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

THE CALL, and it is th-t842's fork widened rather than a new one. th-t842 holds two harness-internal facts for the user's ruling: cl-cv92's (a chat transcript is JSONL whose assistant entries carry message.model, at the path the hook payload names) and cl-jp4q's (beside that file sits <chat>/subagents/agent-<id>.jsonl and agent-<id>.meta.json, keyed by the injected agent id). it-xwpw adds a THIRD, and moves the seat.

WHAT IS NEW. Both standing facts are reachable only because a HOOK is handed transcript_path in its payload. A verb is not handed it, and nothing injects it. For q join to NAME the model it resolved instead of repeating whatever the hook had already put in QUARRY_ACTOR — which is what it-xwpw's acceptance line asks for — it has to find the chat transcript itself, from the chat id it IS injected with. That means inferring two more things the payload used to supply: that the harness's project directories live under $CLAUDE_CONFIG_DIR/projects (or ~/.claude/projects), and that each chat's transcript is named <chat-id>.jsonl inside one of them. coord::locate_chat_transcript and coord::find_chat_transcript_in are where that inference lives, and cl-644e is the vein.

WHY IT IS A DIFFERENT KIND OF EXPOSURE, not simply one more of the same:
  1. THE SEAT MOVED. The two standing facts are read inside hook processes, where a miss degrades an injection nobody watches. This one is read inside a VERB, and its miss changes a printed statement an agent reads and acts on: the join falls to the Injected arm and says the harness's own record could not answer for this seat. Honest, no error, nothing fails — but visible, and read by the one mind that was told to check it.
  2. THE ROOT IS GUESSED, NOT GIVEN. transcript_path is a value the harness hands over. ~/.claude/projects is a convention this repo now assumes. A relocated config dir is covered (CLAUDE_CONFIG_DIR is honoured); a harness that moves where projects live is not, and there is no payload to fall back on at this seat.

THE FORK ITSELF IS UNCHANGED IN SHAPE — accept with a named trigger, bound the dependency, or ask for the documented road (SubagentStart's payload, still unprobed on this machine, which would retire all three facts at the join as well as at the hook). What this thread adds is that option (b), "bound it", now costs more than it did when th-t842 was filed: giving up the subagent road would take the join's honest statement with it, not only the hook's injection, and the join would go back to repeating a value it cannot account for.

WHAT THIS ARC DID PROVISIONALLY, taking the least-committal path consistent with the standing rulings: built it on the it-6ekf posture rather than inventing a new one — every miss degrades to the injected actor and the fallback names itself as a fallback; a chat id carrying a path separator is refused before any directory is read (measured: <project>/../escape.jsonl really is returned without the guard); the exposure is stated in the coord doc comments where it is read. No watch item filed — the trigger is the user's to name, and th-t842 holds the pen.
