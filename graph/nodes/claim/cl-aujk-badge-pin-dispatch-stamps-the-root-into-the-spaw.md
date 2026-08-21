---
id: cl-aujk
type: claim
title: '`badge-pin`: dispatch stamps the root into the spawn line, join plants the identity pin, the session hook injects it, harvest clears it'
v: 3
status: ratified
provenance: assistant
created: 2026-08-18T08:51:35Z
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
  to: file:src/ops.rs
  at: c4450abbbbd8
- rel: supports
  to: cl-nzjs
  at: 3
---

The pin lifecycle rides the badge (dc-g5x5). q dispatch stamps --store <root> into the one-line spawn prompt beside the join token; q join consumes it — the only verb carrying an explicit pin, because the badge and its locale bind together — and records identity to {root, item} in the machine-local, user-level pin file (QUARRY_HOME overrides its home; user-level because a hook process must read it from ANY cwd, and a fork checkout carries no machine-local graph state — gitignored files are never checked out). The session hook injects QUARRY_STORE into badged shells beside SESSION/ACTOR/CHAT/AGENT (dc-zbxj, cl-z6gc), launcher env winning, and only ever from a recorded pin — never from discovery, which would pin every shell to its own cwd. coord::clear_dispatch prunes the pins with the held entry and the acting associations, so the pin lives and dies with the badge: harvest, land, and the steal take-over all clear it. A dead pin (root no longer a store) refuses loudly on the env road and skips silently on the identity road.
