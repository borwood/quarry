---
id: cl-erm8
type: claim
title: '`store-resolver`: one function owns the graph locale — pin first, discovery the fallback, the work root rides along'
v: 3
status: ratified
provenance: assistant
created: 2026-08-18T08:51:18Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-18
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/store.rs
  at: 44d71b72d0f5
- rel: supports
  to: cl-nzjs
  at: 3
---

store::resolve_store is THE resolver (dc-g5x5): every verb obtains its store through Store::resolve and every hook through resolve_store with its own input identity and cwd in hand. Precedence: the explicit pin (q join --store, the spawn line stamp), the env pin (QUARRY_STORE — launcher env wins; the session hook injects it for badged shells), the acting identity recorded pin (agent, chat, then session key), the future configured-location slot, then cwd walk-up discovery — now a private function, the fallback arm. The work root rides along: the repo enclosing cwd (a worktree fork in a worktree dispatch), where file existence and blob stamps read — Store::blob hashes the file the agent actually touched against the store-relative path — while every graph write lands at root. The floor is lint-held in both halves (store::lint_locale_sources; tests/store_lint.rs in CI, q wrap in-repo): outside src/store.rs no source line may carry the discovery call or the pin reads.
