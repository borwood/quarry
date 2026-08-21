---
id: cl-cv92
type: claim
title: '`live-actor`: the injected actor is re-derived from the transcript at every fire; the badge still outranks it'
v: 6
status: ratified
provenance: assistant
created: 2026-08-21T13:11:00Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 24df8ce05680
- rel: source
  to: file:src/teach.rs
  at: d15f00a82b95
- rel: source
  to: file:tests/basic.rs
  at: 4716c2fa0694
- rel: supports
  to: it-j4tx
  at: 4
---

`live-actor`: the injected actor is re-derived from the transcript at every fire; the badge still outranks it

THE DEFECT, MEASURED. coord::record_chat_actor was called from ONE place -- the HookCmd::Orient arm, at SessionStart, keyed by chat id off the hook input's `model` field -- and coord::chat_actor had exactly one production reader, teach::session_hook_output, which read it back unquestioned. Nothing between them ever re-derived. So a /model switch mid-session, or a resume onto a different model, left the row naming a model that had stopped writing the session, and every node the rest of that session minted filed under it. Measured on the dispatching chat itself, 2026-08-21: graph/.chat-actors.json said claude-fable-5 for chat b8214cb4 while all 400 assistant entries in that chat's own transcript said claude-opus-5, and all eighteen rows in the map read claude-fable-5.

THE TRANSCRIPT IS THE ONE CHANNEL THAT ANSWERS PER FIRE. The PreToolUse payload carries no model (cl-dqt4) but it does carry transcript_path, and the last assistant entry there names the model producing turns right now. coord::refreshed_chat_actor is the one derivation point: derive from the transcript, fall back to the recorded row, and rewrite the row only when the value moved, so a settled session's fires are reads. teach::session_hook_output calls it in the slot chat_actor used to hold, so the resolution order is unchanged in shape -- env, badge, chat -- and only the chat half got honest.

READ FROM THE END, SIZED BY MEASUREMENT. coord::transcript_model seeks to len minus min(len, TRANSCRIPT_TAIL_BYTES) and hands the slice to coord::last_assistant_model, the pure core. The window is 512 KiB because across the 23 real transcripts on this machine the last assistant entry sat at most 32,820 bytes from EOF (median about 5 KB) and the largest single line anywhere was 144,652 bytes; half a megabyte clears their sum three times over. Cost, measured in the real hook process against the real 2.37 MB transcript: 404-517 microseconds for the read and scan, against a whole-hook median of 10.0 ms post-fix versus 10.5 ms pre-fix (n=20 each, the delta inside run-to-run noise) and a 10-second hook budget. The cost does not grow with the session, which is the property that made this road takeable at all.

EVERY JUDGMENT IS A FILTER, NEVER AN ASSUMPTION, because the format is undocumented and harness-internal. An entry counts only if type is assistant, isSidechain is not true, and message.model is a real name; `<synthetic>` is the measured counter-example (present in 4 of the 23 transcripts) and any other angle-bracketed placeholder falls with it. Anything unrecognised is skipped and the walk simply continues to the entry before it. A truncated tail drops its first line, which is a fragment by construction.

THE SIDECHAIN FILTER IS LOAD-BEARING, not defensive. Measured 2026-08-21: a subagent's own transcript entries all carry isSidechain true -- all 78 assistant entries of this arc's own -- and older transcripts on this machine interleaved sidechain entries into the parent file. Without the filter a subagent's model becomes the chat's, which is mis-attribution in the expensive direction, the same one badge_model refuses from the other side.

THE BADGE STILL OUTRANKS IT, and the reason is measured rather than assumed. A PreToolUse firing inside a subagent carries the PARENT CHAT's transcript_path, not the subagent's own -- probed live from a subagent seat, 2026-08-21, the whole payload dumped through a temporary build and reverted. The subagent's own transcript does exist, one directory down at <projects>/<chat>/subagents/agent-<agent_id>.jsonl, and it is not what the harness hands the hook. Two consequences. First, the value derived here is the CHAT's model whoever fires, so storing it under the chat key can never smear a subagent's model onto its dispatcher: the row stays chat-keyed by construction rather than by discipline. Second, cl-dqt4's badge stamp keeps owning a joined agent's model -- teach::session_hook_output still asks badge_actor first and falls through to the refreshed chat row, so an inheriting spawn now inherits the parent's LIVE model instead of its stale one.

THE RESIDUE IS NAMED WHERE THE ACTOR IS EXPLAINED. Where the transcript cannot answer -- no transcript_path in the payload, an unreadable file, no assistant entry inside the window -- the recorded row stands and attribution keeps its SessionStart grain. The guide's ENVIRONMENT section now says that in those words rather than leaving an agent to infer it. Two smaller grains, both stated and neither closed here: the current turn's assistant entry is usually flushed before PreToolUse fires but not always (measured both ways across two consecutive fires), so a switch can be honoured one shell late and self-corrects at the next fire; and nothing re-attributes a node already filed under a stale row, which is th-t2fx's gap and outlives this item.

NOT BUILT, AND THE CALL IS ALREADY FILED. The subagent transcript found here would give a subagent's real model structurally, which is what it-6ekf probes for and what would turn q dispatch --model into an override rather than the only road (th-e5ez). Reading it needs the isSidechain filter relaxed for that one caller and leans harder on an undocumented layout, so the fork belongs to the user.

Pinned by the_chat_actor_row_refreshes_from_the_transcript_at_every_fire in tests/basic.rs: the pure core on the switch shape, each filter on its measured counter-example, the truncation rule pinned with a fragment that PARSES (a half-line usually parses to nothing and would be skipped anyway), the defect measured on the derivation itself, all three cannot-answer roads keeping the recorded row, a transcript twice the window read from its end, the end-to-end injection through the spawned binary with and without a transcript in the payload, the badge outranking it, and the guide's grain statement. Each half probed against the regression it guards: restoring chat_actor at the hook reproduces the incident verbatim (QUARRY_ACTOR='claude-fable-5' where opus is right); dropping the sidechain filter, the synthetic filter, the truncation drop, the write-back, or the tail seek each fail their own assert; and inverting the badge-then-chat order fails BOTH this instrument and cl-dqt4's the_session_hook_injects_the_badge_model_over_the_inherited_chat_actor.
