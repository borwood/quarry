---
id: it-8k3p
type: item
title: a backticked vein name is silently eaten by the house shell, and a title fix never re-slugs its file
v: 4
status: sketch
provenance: assistant
created: 2026-08-21T14:43:21Z
actor: claude-opus-5
kind: bug
acceptance:
- 'a vein name reaches the graph as it was written or the mint refuses: a title whose backticked register segment is malformed never lands silently, and a corrected title either carries its filename with it or says plainly that it will not'
witness:
- line: 'a vein name reaches the graph as it was written or the mint refuses: a title whose backticked register segment is malformed never lands silently, and a corrected title either carries its filename with it or says plainly that it will not'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

Witnessed at the it-jsu5 landing, 2026-08-21, and standing on disk right now: graph/nodes/claim/cl-gy6q-eplacement-clear-a-fire-that-displaces-a-live-ar.md. The claim's title reads replacement-clear; its filename lost the r.

TWO FAULTS THAT COMPOUND, and the second is what makes the first permanent.

FIRST, the house shell eats the house style. dc-qvtz makes a backticked name the vein register's form, and PowerShell is this machine's shell. Inside a double-quoted PowerShell argument the backtick is the escape character, so a title written as backtick-r-e-p-l-a-c-e-m-e-n-t loses its opening backtick and its r to a carriage-return escape before q ever sees the string. Nothing warns: q receives a well-formed title that is simply not the one that was typed. Every backticked name whose first letter is one of PowerShell's escape letters is exposed the same way — r, n, t, a, b, f, v, 0, e — which is a large share of the alphabet a vein name is likely to start with, and the agent that hit this had followed the register faithfully.

SECOND, q set title= repairs the title and never the filename. The node's slug is derived at mint and never re-derived, so a corrected title leaves the wrong slug on disk forever. That is defensible on its own terms — the id is the immutable anchor and the render unpacks ids to current titles, so nothing downstream reads the slug — but it means a mint-time typo cannot be undone at all, and this class of typo is silent, so it is discovered late or never.

THE DAMAGE IS BOUNDED, MEASURED AT FILING. Every claim file in the store was checked, comparing the first word of its title against the first word of its slug: cl-gy6q is the only mismatch. So this is a first occurrence and not an accumulated backlog — which is worth stating precisely because the filing instinct was the opposite, and the sweep was cheap. It is a bug on sight under dc-ygzz regardless of count; the measurement bounds the repair, not the urgency. Anyone re-running that sweep later should note it compares first words only and would miss a corruption further into a title.

WHY IT IS A DEFECT AND NOT A TYPING LESSON. The corruption is invisible at the moment it happens, the register that demands the backtick is ratified house style, and the shell that eats it is the one every session here runs. It has already produced one wrong artifact, and it will recur on any vein whose name starts with an escape letter.

THE FORK, for the build. On the eating: q could refuse or warn on a claim title that carries a backticked segment with an unbalanced or missing opening backtick — the register's own shape is checkable, and construction already holds the shape elsewhere (a backticked register name in an acceptance line refuses from a witness seat). On the slug: either re-derive the filename when a title changes, which makes the repair complete and would fix cl-gy6q in passing, or state plainly in the verb's help that it will not, so a corrected title is known to leave its file behind. Doing only the second half leaves the silent corruption; doing only the first leaves today's artifact wrong.

Note the safer authoring road already exists and is undocumented as such: passing a body or title through a here-string or a file, which is how several arcs on 2026-08-21 got their vein bodies in intact. Whatever the build settles, that road is worth naming where titles are taught.

Neighbourhood: cl-gy6q is the damaged artifact, dc-qvtz the register that demands the backtick.
