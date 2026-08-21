---
id: cl-znh4
type: claim
title: '`counter-voice`: graph silence draws one hook-injected reminder - session acts reset, dense once then light, subagents neither count nor hear'
v: 3
status: ratified
provenance: assistant
created: 2026-08-19T10:32:54Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-19
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/teach.rs
  at: d15f00a82b95
- rel: supports
  to: cl-xrrx
  at: 3
---

The mid-session half of dc-hzrm, landed under it-gwj7. teach::counter_voice counts hook-observed turns - Bash/PowerShell PreToolUse firings by the acting session own chat, the only turn signal the hook can see; pure conversation without tool calls is invisible to it by construction - and injects one reminder line into additionalContext when COUNTER_VOICE_TURNS pass with no logged graph act from the session. Any logged act resets the counter and re-arms the line: one fire per silence stretch, never a per-turn nag, and a filing session never sees it by construction. First encounter renders framings::COUNTER_VOICE_DENSE, later encounters the light phrase; the encounter state rides graph/.counter-voice.json, machine-local working state of the topic-queue species (a .gitignore line for it is owed at the canonical tree). Subagent-marked hook inputs (agent_id present) neither count turns nor receive the line, though their q acts still stamp the session and reset through the log. Fired on silence, never the clock (dc-dty5: the silence itself is the waiter). Pinned by counter_voice_fires_on_silence_resets_on_acts_and_teaches_dense_then_light and counter_voice_rides_the_session_hook_and_skips_subagent_contexts in tests/basic.rs.
