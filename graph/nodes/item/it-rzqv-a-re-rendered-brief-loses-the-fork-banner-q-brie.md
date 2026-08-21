---
id: it-rzqv
type: item
title: 'a re-rendered brief loses the fork banner: q brief orients no one, and it is the command the agent is told to re-run'
v: 1
status: sketch
provenance: assistant
created: 2026-08-21T08:54:45Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Noticed from the agent seat while landing it-rmqy's second arc. The where-you-stand banner (cl-kr7f) is composed in ops::join and prepended to the rendered brief, so a fork agent is oriented exactly once — at the join. Every later re-render of the same brief loses it: q brief <item> calls render::brief directly, and that is the command the graph itself keeps advertising to a working agent (the write-guard teaching line and the PreToolUse dispatch reminder both end with "q brief <item> re-renders the full brief"). An agent that re-reads its brief mid-arc — the common case after a context loss, which is precisely when orientation matters most — reads a brief that says nothing about the fork it is standing in.

Two shapes, both cheap:
- ops::fork_banner is already pub; one call in main.rs's Brief arm (or in render::brief) prepends it there too.
- Or the banner stays a join-time-only greeting by design, and this item closes as won't-fix with the reason recorded.

The placement argument that put the banner in ops::join is real and unchanged either way: the brief's second line promises everything below it is derived from the graph at render time, and where the process stands is not graph. That argues for the main.rs call site over render::brief, not against carrying it at all.

Left unbuilt deliberately: it-rmqy's acceptance names the join and the spawn line only, and the first arc's report offered this to the dispatcher as a judgment rather than taking it. Filing it so the call outlives the report.
