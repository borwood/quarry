---
id: it-ygw7
type: item
title: 'the surfacing atom: one struct, three renderers, no register below the floor'
v: 5
status: done
provenance: assistant
created: 2026-08-11T10:29:15Z
actor: claude
kind: debt
acceptance:
- 'lands `surface-atom`: one atom struct renders every node surface; three renderers (atom_line, atom_ref, atom_unpack) own all registers; surfaces may append, never subtract'
- 'lands `atom-line`: the full atom — id, title, type, kind, status, v, areas, provenance, archived — on one line for every list register, hook enumerations included'
- 'lands `atom-ref`: every binary-composed sentence carries at least "title" [type status] (id); status unconditional; archived rides the status slot'
- 'lands `atom-unpack`: body renders expand ids from the same atom struct, id-anchored, announcing areas only when cited and citing areas differ'
- 'lands `atom-lint`: front.title formatted outside the surface module fails the lint at wrap or CI — checked, never remembered'
- 'lands `carrier-replumb`: homework and query carriers hold atoms, not (id, title) pairs'
- 'lands `find-areas`: find hits name their areas, so foreign-domain matches arrive labeled'
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/main.rs
  at: a37b2a28f726
- rel: about
  to: file:src/render.rs
  at: 806e455a0eb3
- rel: about
  to: file:src/queries.rs
  at: f3faffcbe1d7
- rel: supersedes
  to: it-u7dp
  at: 6
- rel: depends-on
  to: dc-nnf5
  at: 2
---

Yield of the 2026-08-11 surfacing audit (96 front.title sites across 9 files): one shared formatter, line(), private to src/main.rs; roughly 25-30 handrolled registers beside it; no register carries areas, kind, or grounding; id-less registers include cross-session arrivals, lease confirmations, the landed-uncited lint, and most refusals; queries.rs homework carriers bake the starvation into the data layer as (id, title) pairs, so no print-layer fix can retrofit the missing fields. Extracting every register into one surface module is also the first real seam out of the src/main.rs monolith (th-yzmj) and turns the invariant grep-lintable — enforcement by construction, not discipline. Scope ruled by dc-nnf5; acceptance mirrors the ruling. Re-minted from it-u7dp, whose acceptance names were corrupted in flag transit (shell backtick escaping) with no acceptance-remove verb to repair them; that verb gap is filed as its own item.
