---
id: cl-aruk
type: claim
title: '`consume-readings`: settling verbs sweep readings whose last live consumer settled; the sweep archives and says what it hid'
v: 3
status: asserted
provenance: assistant
created: 2026-08-17T08:59:01Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: 3d01fde92bd6
- rel: supports
  to: cl-9z95
  at: 3
---

ops::consume_readings walks readings in blast's leaning vocabulary (inbound depends-on/source/builds-on plus the reading's own supports targets); when every consumer is settled for its type - items done/dropped, threads resolved, decisions in-force or superseded, claims refuted/superseded, docs always - the reading archives with a logged consumption cause and the calling surface says what it hid. set (status/kind flips), archive, rule, and refute run the sweep; a consumerless reading never sweeps; q archive admits readings at any ladder status because a reading settled when it landed (dc-6gn9). First live firing: cl-eb8j archived at its own re-kind.
