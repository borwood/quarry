---
id: it-nnrs
type: item
title: 'affirm --to takes no key: a slug or title silently scopes to nothing where every other verb resolves it'
v: 2
status: sketch
provenance: assistant
created: 2026-08-21T12:08:14Z
actor: claude-fable-5
kind: debt
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Named as residue at the it-awhz landing, 2026-08-21, and left unfixed there because that arc was scoped to the message.

Every other verb in this CLI resolves a node key the way q find does — id, title, or a distinguishing fragment. q affirm --to does not: it string-matches the scope against edge targets as written, so only a bare id (or a file: ref, the affordance it-awhz documented) selects anything. A slug or a title silently scopes to nothing.

Before it-awhz that silence was invisible — the zero printed the unscoped all-clear, so a mistyped scope and a clean node read identically. cl-2kxr closed that: a scope naming nothing now says so and reports the node's reach beyond it, so the failure is honest. What remains is only the asymmetry — a verb that refuses the input shape its siblings accept, in a surface whose own homework lines hand out the id form and therefore never trip it.

Filed as debt, not a bug, on that ground: nothing is silently wrong any more, and the machine-composed recipes all pass ids. The cost falls on a hand-typed scope, where the reader now gets a truthful 'no ref toward that' rather than a false all-clear — a wrong answer to the question they meant, but a legible one.

The cure is the ordinary resolver at the scope, with one thing to settle: a fragment that matches several edge targets wants a refusal listing them, not a silent pick, since scoping is the whole point of the flag.

Neighbourhood: it-awhz for the surface this leans on, cl-2kxr for the three zeros it now tells apart.
