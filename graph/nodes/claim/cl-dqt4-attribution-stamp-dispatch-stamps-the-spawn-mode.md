---
id: cl-dqt4
type: claim
title: '`attribution-stamp`: dispatch stamps the spawn model into the badge; the session hook spends it for the joined agent, keyed on the agent id alone'
v: 10
status: ratified
provenance: assistant
created: 2026-08-21T12:35:08Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 24df8ce05680
- rel: source
  to: file:src/teach.rs
  at: d15f00a82b95
- rel: source
  to: file:src/ops.rs
  at: 7785f1965da3
- rel: source
  to: file:src/main.rs
  at: 75651d05f793
- rel: source
  to: file:tests/basic.rs
  at: 75f4c2e53c44
- rel: supports
  to: it-xcvb
  at: 5
- rel: about
  to: ar-xa38
  at: 1
---

`attribution-stamp`: dispatch stamps the spawn model into the badge; the session hook spends it for the joined agent, keyed on the agent id alone

THE HOLE, MEASURED. coord::record_chat_actor writes graph/.chat-actors.json keyed by CHAT id from the SessionStart hook input's `model` field, and the session hook injects that value as QUARRY_ACTOR. A subagent is not a chat and never fires SessionStart, so a dispatched agent inherited its dispatcher's row. Read at the it-xcvb landing: eighteen entries, every one claude-fable-5, no subagent row at all, while two Opus arcs did that day's code and graph work. Every node those arcs minted -- cl-kr7f, it-x4bb, it-rzqv and their affirms -- carries a model that did not write it.

NOTHING ARRIVES AT THE AGENT'S SEAT. Probed live from inside a subagent, 2026-08-21: a PreToolUse firing there carries session_id, transcript_path, cwd, prompt_id, permission_mode, agent_id, agent_type, effort, hook_event_name and the tool fields, and no model of any kind. The docs agree -- only SessionStart carries `model`, and not guaranteed even there; SubagentStart's documented input names agent_type and agent_id and no model. That much still holds. What this paragraph originally concluded from it -- that the per-agent cure could not be written at all -- was wrong, and cl-jp4q corrected it the same day: nothing arrives, but the answer is already on disk beside the transcript the payload does name.

THE DISPATCHER IS THE ONE PARTY THAT KNOWS, and the badge is where it already writes. q dispatch --model <model> stamps DispatchState.model at the fire (ops::dispatch), re-stamped on every fire by construction -- a re-dispatch or a steal builds a fresh entry, so an arc handed to a different model carries the model it was handed to -- and logged on the dispatch event besides, the durable seat the badge is not (the badge dies at harvest).

SPENT AT THE HOOK, KEYED ON THE AGENT ALONE. coord::badge_model is the one derivation point, pure over the DispatchMap: an agent:<id> acting row through live_acting_badge, then that badge's stamp. coord::badge_actor is the store-reading half, already through safe_actor so a display-name model can never derive USER provenance. teach::session_hook_output prefers it over the chat-actor map and falls through whenever nothing is stamped, so an inheriting spawn keeps the answer that is right for it. The agent key is deliberate: every other identity is a real chat that fired SessionStart and already has its own model recorded, and reading a chat-keyed row would spread the badge's model onto the dispatching chat's own shells wherever parent and subagent are indistinguishable (record_acting's no-agent-id fallback) -- mis-attribution in the expensive direction, bought for nothing. Residue is never a binding: live_acting_badge already refuses a row pointing at a cleared dispatch, so harvest frees the actor with the badge.

THE JOIN EVENT IS THE WHOLE OF THE PRE-BIND WINDOW. The hook firing that injected QUARRY_ACTOR into the joining shell ran before the bind existed, so that one shell still carries the dispatcher's model. ops::join therefore stamps the join event from the badge directly (agent-keyed, matching badge_model), and the arc's first badged act already files right; every later shell resolves it at the hook.

STATED AT BOTH SEATS, GATED AT NEITHER. The fire states the arc's attribution above the spawn prompt every time, because that is the one station that can still change it before the agent exists; the join states it to the agent, riding the BIND line rather than a line of its own (the it-rmqy invariant: the where-you-stand banner must be the very next thing a fork join says), because the joining agent is the one mind that knows its own model. What each seat SAYS was rewritten at it-xwpw once the stamp stopped being the only road -- see the amendment below.

AMENDED AT THE it-6ekf AND it-xwpw LANDINGS (2026-08-21, the dispatcher's hand). Two paragraphs of this body described a world that lasted about three hours, and both are corrected above rather than left to mislead.

The residue this vein named as DISCIPLINE is closed. It read: a dispatcher that omits --model while spawning on another model reproduces the defect silently, and nothing in the harness can see the mismatch -- queued against dc-zbxj's identity-is-structural-never-discipline. cl-jp4q closed it by finding the answer on disk: beside the chat transcript sits <chat>/subagents/agent-<id>.jsonl and agent-<id>.meta.json, keyed by the very id the hook already injects as QUARRY_AGENT. An unstamped arc now resolves its OWN model, so --model is an override rather than the only road, and identity-is-structural holds for attribution too.

The two-answers framing at the fire went with it. There is no longer an "inherited answer" to state: ops::DispatchOutcome.arc_actor became Option, the fallback to the dispatching chat's actor is unrepresentable rather than merely unprinted, and an unstamped fire names the ROAD the agent will resolve on instead of predicting a model. The join no longer predicts at all -- it runs inside the agent with the id in hand, locates the chat transcript from the injected chat id (cl-644e), and names the model it actually read, saying which of stamp, record, or injection answered.

Two things this vein's own instrument got wrong, worth recording because they were ratified: one assertion pinned the pre-fix behaviour as correct ("the fire states the inherited answer verbatim", expecting the dispatching chat's model from a bare fire), and the instrument does not catch a badge/record precedence inversion at all -- cl-jp4q's test is the only thing holding that order. Neither was a mis-assay when made; the world moved under them the same day, which is itself the argument for reading a vein's instruments when its ground shifts.

Pinned by a_dispatched_arc_files_under_the_model_it_was_spawned_on and the_session_hook_injects_the_badge_model_over_the_inherited_chat_actor in tests/basic.rs: the defect measured on the derivation and again end to end through the spawned binary (the subagent's shell and its dispatcher's handed the same actor), then the stamp, the join event, the log, the agent-key boundary, the harvest release, and the safe_actor guard. Each half probed against the regression it guards -- dropping the hook preference fails the end-to-end assert with claude-fable-5 still in the prefix; restoring Store::actor() on the join event fails the first-act assert; widening badge_model to chat keys fails the chat-row assert.
