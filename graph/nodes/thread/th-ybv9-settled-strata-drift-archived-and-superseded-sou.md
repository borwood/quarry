---
id: th-ybv9
type: thread
title: 'settled-strata drift: archived and superseded sources re-enter behind on every src move'
v: 1
status: queued
provenance: assistant
created: 2026-08-18T02:51:31Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Noticed during the it-4p7m source-drift pass: behind's wants-action enumerates drift from settled strata — archived, dropped, and superseded SOURCES re-enter the queue on every src move. The dc-6gn9 classifier collapses sediment for readings only; this pass met a dropped-and-archived item carrying three sev-4 entries (it-u7dp) and three superseded claims (cl-amza, cl-farc, cl-vsew), each affirmed to clear the queue — but their refs drift again at the next landing, and re-affirming dead strata every pass is recurring noise, not signal. The affirm is also mildly dishonest for a superseded claim: the re-stamp says reviewed-at-this-blob about text that by definition no longer describes the code. Open call for design: should settled-strata drift collapse like sediment (its own count line, reach stated), be skipped at the classifier outright, or does re-stamping stay the owed act because an entry on any live surface must mean something?
