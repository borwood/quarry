---
id: it-4q6t
type: item
title: 'commit sweep: git add graph stages other sessions uncommitted nodes'
v: 9
status: done
provenance: assistant
created: 2026-08-12T21:12:19Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: sweep-shaped staging that would capture another session''s uncommitted node files denies at the shell hook with the asking session''s own git add line in hand; ownership of mixed files derives last-writer-from-the-log; foreign files list with owner last-seen age and flip to an explicit-path adoption offer when stale; retire offers last-rites commit of the retiree''s leftovers; wrap echoes the commit-set; explicit-path staging always passes; q query commit-set is the advertised pull handle'
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
---

Three instances on record: 2026-08-12, git add graph swept thirteen uncommitted ui-cleanup nodes into the dc-ydvb decisions commit. 2026-08-17, the dispatcher's git add -A swept the design session's fresh acceptance-gate nodes (dc-p6z4, it-33bb, edits to it-ds6b, th-wxr9, th-jvwe) minutes after they were written. 2026-08-19, the dispatcher's commit 60f4aca swept the design session's it-f6c2 shaping mid-sweep. Two-plus sessions share one working tree; leases guard file writes, not commit staging; parallel main sessions are the normal shape, so the collision window is standing. The harm in every instance is silent absorption under an unrelated message — the graph log attributes correctly throughout; git history does not.

The settled fix (user-agreed 2026-08-20) — the session hook is the choke point; the log is the ownership oracle (every event carries the session stamp):

1. DENY-AND-TEACH AT THE SHELL HOOK, conditionally: a sweep-shaped staging command (git add graph / -A / . / git commit -a) that would capture another session's uncommitted node files refuses at the PreToolUse hook (the C6 channel), handing the asking session its own git add line. Nothing foreign pending: silence, the sweep is harmless. Detection is string-matching the command — best-effort, wrap backstops, evasion is self-inflicted (write-guard posture).
2. OWNERSHIP DERIVES: a node file's owner is the session with the LAST event on the node since the prior commit. Mixed-authorship files land in exactly one commit-set, annotated as carrying the other session's earlier edits; the other side sees theirs-to-carry. Deterministic — no deferral deadlock.
3. ADOPTION IS EXPLICIT PATHS: the guard blocks the bulk shape, never deliberate adoption. Foreign files list with the owner's last-seen age; stale owners flip the line to an adoption offer — git add <exact paths> plus a commit message naming the origin, which passes the guard by construction.
4. RETIRE OFFERS LAST RITES: q session retire names the retiree's uncommitted node files and offers committing them as their own labeled commit at the moment the session provably ends.
5. WRAP ECHOES the commit-set whenever uncommitted graph changes exist; the shared monthly log file is exempt ledger (internally attributed, never split); q query commit-set is the pull handle both surfaces advertise.
