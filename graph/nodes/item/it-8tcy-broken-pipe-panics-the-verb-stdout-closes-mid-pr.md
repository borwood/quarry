---
id: it-8tcy
type: item
title: 'broken pipe panics the verb: stdout closes mid-print and homework is lost'
v: 1
status: sketch
provenance: assistant
created: 2026-08-16T02:44:35Z
actor: claude
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Observed 2026-08-15: piping a verb through 'head -1' closed stdout early and q panicked (os error 232, failed printing to stdout) after the write landed but mid-homework print - the edit succeeded, the homework enumeration was lost. Two defects in one: the panic itself (broken pipe should exit quietly), and the loss surface - homework prints once at act time, so a swallowed print is gone (kin to it-pafx: surfaces should self-certify what happened). Fix shape: ignore SIGPIPE-equivalent on Windows stdout writes or handle the io error gracefully.
