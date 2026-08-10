---
id: do-799m
type: doc
title: 'resource rules: this machine''s build serialization'
v: 1
status: registered
provenance: assistant
created: 2026-08-10T06:49:35Z
actor: claude
kind: protocol
edges:
- rel: about
  to: ar-xa38
  at: 1
on: brief
tier: inline
---

Build rules for this machine — violating these has hung this box:
· cargo: ~/.cargo/bin is on the User PATH; if bare `cargo` fails in your harness, use the full path ("$HOME/.cargo/bin/cargo" bash · "$env:USERPROFILE\.cargo\bin\cargo.exe" pwsh).
· TMP/TEMP→B:\tmp and jobs=4 are owned by ~/.cargo/config.toml — do not override them; C: is space-constrained and a full drive fails the linker with LNK1108.
· ONE build at a time, never concurrent cargo processes — this machine has hung under parallel heavy builds.
· Verify tests by READING the raw `test result:` line and its counts, never through a grep filter — a filter once silently swallowed a pass line ("0 passed" is a substring of "20 passed").
· This repo's hooks invoke target/release/q.exe: while cargo rebuilds it the exe is being replaced, so a transient hook error mid-build is the build, not the graph — run q verbs outside build windows.
