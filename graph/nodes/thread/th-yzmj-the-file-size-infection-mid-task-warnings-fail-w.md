---
id: th-yzmj
type: thread
title: 'the file-size infection: mid-task warnings fail; what defense actually works'
v: 4
status: queued
provenance: user
created: 2026-08-11T06:28:36Z
actor: claude
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: file:src/main.rs
  at: 2d2daff8ff48
---

file-size was a major infection deepcraft had; we instated hook warnings that didn't really work — agents always dismissed extraction mid-task as out of scope. file-size infects us here: an agent can never read the full file but must grep through it and possibly miss important context.

State 2026-08-11, recorded mid-session in case the thread outlives it. The 2000-line threshold argument died in conversation (2000 reads whole); the thread reframed to first principles. Read-side: for most work an agent needs the map and the contracts, not internals — the residue is debugging (where contract and internals have diverged) and boundary-moving refactors. The user's blue-sky supplement: a derived summarizer — skeleton from parse (always fresh, portable across codebase opinions), contracts from stamped claims, absences visible — decoupling reading cost from file layout; main-leaf branching derived, not imposed. Write-side survives the reframe: granularity still governs stamp sharpness, lease grain, and diff blast — a claim stamped on a monolith drifts on every unrelated edit. Contracts cannot be derived: selection (promised vs incidental), normativity (what counts as wrong), and commitment (binds the future) are authored acts — derivation drafts, ratification binds, so the contract layer is claim-shaped (cl-hd56 landed this session as the claim verb's own bone). The pressure half of any defense is th-jvwe's question verbatim; discovery can ride the observe/accrue/wrap machinery, and nothing here settles pressure. The surfacing audit (it-ygw7; scope ruled by dc-nnf5) found the first real extraction seam out of src/main.rs: the surface module. User, closing the audit: 'I should have spent more time laying out project structure in the first session, and that's on me. This is a fatal pattern and is part of what prompted the start of this project.'

State 2026-08-13, from the contracts conversation (th-nc7d): the contract species settled in shape — kind=contract as the first explicit claim kind; asserted is the draft, ratified is the binding (the status already in the schema); source: file blob-stamped, about: file. Symbol anchoring, first slice: the backticked symbol NAME in the title (feeding the lexicon), blob stamp carrying drift — and a user note for the summarizer's shaping: a dedicated symbol FIELD may follow, enabling two directions at once — the contract node programmatically resolving current line numbers for its symbol, and the summarizer presenting the contract body at the right section of its file summary. Also landing in this orbit: ratification authority (who binds a drafted contract — user, session judgment, or protocol-declared; the contract register protocol dogeared on th-qv27 carries the answer); processual creation moments (harvest sweeping a landing for new public boundaries is the natural candidate — the choke point that asks vein-or-no-vein can ask contract-or-no-contract); and the write-time file-grain watermark (it-wtre), the closest-to-source delivery beside the summarizer's read-time surface. The summarizer consumes both species: contracts at symbol grain and veins (dc-wrnc renames spines) at mechanism grain.
