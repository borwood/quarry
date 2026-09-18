# quarry

A standalone work-graph RAG tool for AI-native development.

quarry holds the knowledge a project accumulates — decisions, claims, open
questions, work items, prose artifacts — as typed nodes with typed,
**version-stamped** edges. Data lives as plain files inside the host project's
repo (`graph/`), written only through the tool's verbs, and consumed as
derived views (session briefs, dispatch briefs, handoffs, a browsable UI) that
are never hand-maintained.

The design premise, earned the hard way on a prior project: every
process-documentation failure — stale cross-references, one-directional
pointers, contradictions between documents, re-derived dead ideas, silently
overwritten decisions — is a graph invariant being enforced by prose and
attention. Discipline-based invariants decay. quarry turns each one into
**an edge, a query, or a denied write**.

Status: dogfooding + continuous improvement phase. See [docs/DESIGN.md](docs/DESIGN.md).
