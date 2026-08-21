---
id: cl-8qn3
type: claim
title: '`retire-scope`: q session retire refuses only when the retiree is implicated - own session under a live badge, or a retiree mid-dispatch toward its q harvest'
v: 2
status: ratified
provenance: assistant
created: 2026-08-20T10:19:36Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-20
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 24df8ce05680
---

coord::retire_refusal is the one guard point for q session retire (it-e6wq), replacing the blanket boundary capture at that arm - the fix user-agreed 2026-08-19 scoping the refusal to the RETIREE. (1) Retiring the chat's own session (the injected QUARRY_SESSION) while this context is mid-dispatch routes through boundary_refusal verbatim - the full capture enumerating every held dispatch with its q harvest command; that IS closing out with harvest owed. (2) A retiree with a dispatch of its own in flight - any held entry whose session field names the retiree, or whose holder is recorded as the session key - refuses pointing at that dispatch's q harvest: retire releases the retiree's leases, ripping the zone out from under a working agent. (3) A third session with no live dispatch retires clean while unrelated badges fly - this chat's own held badges and env badges included - and its idle leases release with last rites, retire's standing semantics. Wrap and session resume keep the blanket capture of cl-ahzk; retire alone is retiree-scoped, which amends cl-ahzk's retire clause and the retire word of dc-qyr5's boundary sentence per the it-e6wq user agreement. Pinned end to end through the spawned binary by session_retire_refuses_only_when_the_retiree_is_implicated in tests/basic.rs.
