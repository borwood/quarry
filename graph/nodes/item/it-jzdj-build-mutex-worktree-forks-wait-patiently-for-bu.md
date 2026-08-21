---
id: it-jzdj
type: item
title: 'build mutex: worktree forks wait patiently for build time'
v: 1
status: sketch
provenance: user
created: 2026-08-21T07:42:20Z
actor: claude
kind: slice
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
---

The user's word at the serial run's start (2026-08-20): 'we'll be building a mutex so worktrees wait patiently for build time. that's for sure.' This machine builds one at a time (do-799m); the serial run honored it by dispatching serially - eleven arcs, one builder at a time - but parallel worktree dispatch stays blocked on exactly this: two forks compiling concurrently have no coordination point. The mutex makes the one-build rule construction instead of discipline, and unlocks the parallel dispatch shape th-mbbb's ruling will want.
