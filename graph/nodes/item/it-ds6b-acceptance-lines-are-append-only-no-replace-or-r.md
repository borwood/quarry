---
id: it-ds6b
type: item
title: 'acceptance lines are append-only: no replace or remove verb'
v: 12
status: done
provenance: assistant
created: 2026-08-11T10:29:40Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: q set removes or replaces acceptance lines via acceptance-= matched exact-or-unique-substring with refusal listing candidates on ambiguity, the log and echo carry the full resolved line removed, stripping a readied item''s last line loudly demotes it to shaped in the verb''s own output, non-design-seat mutations ride the witness review channel, and the item bumps - the contract is content'
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/ops.rs
  at: dcf1ef41f019
- rel: depends-on
  to: dc-p6z4
  at: 2
- rel: depends-on
  to: dc-mpg8
  at: 3
- rel: about
  to: file:src/queries.rs
  at: a444d032fe31
- rel: about
  to: file:src/model.rs
  at: 3c1ae26933b8
- rel: about
  to: file:src/main.rs
  at: dbb05e52e902
---

q set supports acceptance+= only; a corrupted or stale acceptance line cannot be repaired or retired through any verb. Forcing incident 2026-08-11: shell-mangled backtick names forced re-minting it-u7dp whole as it-ygw7 — the only repair for one bad line was dropping the item.

The settled fix (user-agreed 2026-08-20):

1. REMOVAL BY LINE, acceptance-= : q set <item> "acceptance-=<text>" identifies the victim line and deletes it. Replace is both fields in one q set call — "acceptance-=<old>" "acceptance+=<new>" — one act, one log event, one bump. No new verb; q set's field list already carries it.
2. MATCHING FORGIVING, RECORD FAITHFUL: the -= value matches the exact line, or any substring matching exactly one line; zero or multiple matches refuse, listing candidates — never a silent no-op. The confirmation echoes and the log stores the full resolved line actually removed, never what was typed (the mint-echo pattern: a wrong-but-real match reads wrong in the echo). Strict-verbatim input would rebuild the forcing incident's trap — long backtick-laden lines round-tripping shell quoting — on the removal side.
3. THE DEMOTION REQUIREMENT (inherited from th-wxr9, user-ruled 2026-08-17, a requirement not a suggestion): an edit that strips a readied item's last acceptance line loudly demotes ready to shaped in the verb's own output — the ready-implies-acceptance invariant held at every mutation point: flip refuses (q ready), fire backstops (q reserve), the brief trips, and these verbs demote on strip.
4. THE ITEM BUMPS: unlike q unlink, this is content, not bookkeeping — the contract changed; citers' stamps go behind accordingly. Logged with who and why, in the unlink spirit.
5. SEAT RULES FOLLOW THE PEN (dc-p6z4, dc-mpg8): authoring-by-subtraction is authoring. From a non-design seat a replacement line witness-marks like any authored line and a removal rides the same review channel until the user ratifies; design seats mutate freely. Existing witness display already degrades gracefully — a removed marked line shows "no longer in acceptance, mark stands as record".
