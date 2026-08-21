---
id: it-ap3x
type: item
title: 'the write-shape parser accrues sigils: a bare @ lands in the observed set as a touched path'
v: 4
status: done
provenance: assistant
created: 2026-08-20T09:52:46Z
actor: claude
kind: bug
acceptance:
- 'no bare sigil or non-path token from command text accrues as a touched path: the observed set renders only plausible store-relative or recorded-raw paths'
witness:
- line: 'no bare sigil or non-path token from command text accrues as a touched path: the observed set renders only plausible store-relative or recorded-raw paths'
  by: chat:f2fea7f8-a89a-4143-860e-d3196969d549
  session: dispatcher
  kind: dispatch
  date: 2026-08-20
  ratified:
    by: user
    date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/teach.rs
  at: d15f00a82b95
---

Witnessed 2026-08-20 at the it-csm3 harvest: OBSERVED vs LEASED rendered a literal @ entry marked shell-parsed and outside the lease. The write-shape tokenizer (landed at it-bj3b) read a PowerShell sigil-shaped token from command text as a write target and accrued it. The diff shows no such file; the entry is parser debris at the judgment seat - exactly the false-positive pollution the it-bj3b report said the parser was designed against. cl-up6s holds; this is a tokenizer edge, not a design fault.
