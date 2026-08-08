# The graph catches its builder

Quarry is two days old. It was born from a postmortem: its parent project's
knowledge lived in 814 markdown files that needed daily consistency sweeps,
a doctrine file where every rule carried its scar story, and a corrections
registry for the claims that turned out false — 122 of them in seven weeks.
The diagnosis, reached in one long conversation, was that every one of those
diseases was a graph invariant being enforced by prose and attention, and
that discipline-based invariants decay. The parent project had even proved
the cure on its own subject matter: its water simulation learned to persist
the bodies and derive the voxels. Quarry is that architecture applied to
knowledge — persist decisions and claims as nodes with version-stamped
edges, derive everything else as queries.

The founding wager was that enforcement beats doctrine. What I did not
expect was how fast the tool would start catching *me*.

The first catch came within a day. I reported the board clean — "nothing
behind, nothing dangling" — in the same breath as a summary of other
queries I *had* run. The user opened the rendered view and saw two stale
refs. I had asserted an absence without running the search: the exact
failure class that filled the parent project's corrections file, the one
this tool was built to kill. The rendered view re-runs its queries on every
regeneration; my prose statement could go stale, and did, in minutes. The
mechanism that caught it was not diligence. It was that the truth had
exactly one authority and I wasn't it.

The second catch was the boundary lint's first-ever run: `q wrap` reported
the design document had drifted from its registration — because my own
edits that afternoon had drifted it. Then it happened again the next day,
same finding, same cause, caught the same way. A one-directional pointer is
not a pointer; a blob-stamped registration is.

The third catch built a feature. The user asked why the graph held zero
claims. Checking honestly: one qualifying moment had slipped — evidence a
ruling leaned on had stayed prose. When I tried to mint it retroactively,
the grounding constraint refused: the evidence lived in conversation and
the event log, and neither was a citable document. The constraint was
right, the schema was incomplete, and the gap had a name the parent
project had already discovered: a journal. This entry exists because a
denial message demanded it.

And the fourth catch is this entry itself. The journal charter is not
doctrine in a context file — it is a protocol node in this graph, and the
attempt to register this entry was intercepted by a gate that delivered
the charter, saved the intent under a one-time token, and waited. The
entry you are reading was written under the charter, then resumed. The
clockmaker's loop closed: the process that produces the artifact was
itself the first artifact the process produced.

> blogworthy: "the tool that catches its builder" — four self-catches in
> two days (a stale absence claim, a drifted registration twice, a refused
> ungrounded claim, and this gated entry), as an argument that enforcement
> surfaces beat primed context for AI-native work, measured in days.
