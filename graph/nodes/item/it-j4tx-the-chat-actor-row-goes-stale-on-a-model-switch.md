---
id: it-j4tx
type: item
title: 'the chat-actor row goes stale on a model switch: SessionStart writes it once and the session files under a model that stopped writing it'
v: 1
status: sketch
provenance: assistant
created: 2026-08-21T12:36:08Z
actor: claude-fable-5
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

The chat-actor row is written ONCE, at SessionStart, and never revisited. coord::record_chat_actor is called from the HookCmd::Orient arm alone, keyed by chat id from the SessionStart input's `model` field; the session hook then injects that value as QUARRY_ACTOR for every shell of that chat, forever. Nothing re-reads it. A model switch after the session starts -- the user's /model, or a resume onto a different model -- leaves the row stale, and the whole rest of the session files under a model that stopped writing it.

MEASURED LIVE, 2026-08-21, on the dispatching chat itself. graph/.chat-actors.json records chat b8214cb4-bf6e-42fa-9ceb-317d5660ca2d as claude-fable-5 (all eighteen entries in the map read claude-fable-5). Its own transcript at ~/.claude/projects/B--repos-borwood-quarry/<chat>.jsonl carries 400 assistant messages, and every one of them is "model":"claude-opus-5"; isSidechain is false throughout, so this is the top-level chat's own record of what produced its turns. The chat is running Opus 5 and filing everything under Fable 5. cl-2kxr, minted by that chat at 12:03:22Z the same day, carries actor: claude-fable-5 on its front matter.

WHY IT MATTERS BEYOND ITS OWN SESSION. This is the same family as it-xcvb (a dispatched subagent inheriting its dispatcher's row) and it compounds with it: the value a dispatched arc inherits is not even the dispatcher's real model, it is the dispatcher's SessionStart-recorded model. The house commit convention names the model that did the work and the graph is the only record of it; attribution is also the ground for reading the corpus later (which model produced which claim, how a model's veins held up at assay).

THE CANDIDATE CURE, UNPROBED. The PreToolUse hook input carries no model (measured -- see cl-dqt4), but it does carry `transcript_path`, and the transcript's last assistant message names the model currently producing turns. Reading it backwards from the end at hook time would let the row refresh itself. That format is undocumented and harness-internal, and the cost per hook fire needs measuring before anyone leans on it. The alternative is to accept SessionStart-grained attribution and say so where it is read.
