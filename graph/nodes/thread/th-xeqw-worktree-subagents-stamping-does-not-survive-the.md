---
id: th-xeqw
type: thread
title: 'worktree subagents: stamping does not survive the worktree boundary'
v: 1
status: queued
provenance: user
created: 2026-08-14T00:54:08Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
---

Dogeared 2026-08-13 mid-session, deliberately not expanded in the filing session: the user notes the stamping system does not work with subagents running in git worktrees - this slipped through the cracks. The user's normal working pattern is subagents in worktrees, from the deepcraft repo whose postmortem quarry is founded on (do-aurk). Open for discussion when pulled: what exactly breaks (badge association, write-hook observation, blob stamps taken against a different working tree), and what the fix wants to be. Nothing here is settled; the thread exists so the gap is owned data rather than memory.
