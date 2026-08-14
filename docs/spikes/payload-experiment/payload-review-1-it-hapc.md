# Payload review — it-hapc dispatch brief (candid, first read)

Written immediately after `q join gey3etbyxb`, before starting the work.

## What it contains

- **THE WORK** paragraph: one dense paragraph carrying the whole spec — derive live
  sessions with coverage (kind + purview fit + last_seen heartbeat) at `q dispatch`
  from a non-dispatch session; any live → leave the item ready and print who covers
  it; none live → offer the wake choice (enumerate dispatch-kind sessions, one
  charter-certain candidate → offer its launcher, several → user picks); fire solo
  stays legitimate; surface never gate.
- **READ-FIRST**: four cited dependencies (it-u8uf per-chat badge, dc-ydvb session
  kinds ruling, it-skpa kind-as-registry-data, dc-crea the pull-not-send ruling) with
  full body text inline.
- **Area dump for `cli`**: six open threads (all marked NOT yours), nine in-force
  decisions with full bodies, ~27 spine claims, 15 registered docs.
- **WRITE-SET**: src/**, tests/** leased.
- **RETURN SPEC**: no acceptance recorded — flagged with an instruction to fix the
  graph first.
- **PROTOCOL**: machine build rules (one cargo at a time, read test lines raw).
- **ACTOR RULES**: build-from-spine, C3, cite, don't flip done.

## What is genuinely relevant

- THE WORK paragraph is excellent — it is the spec, and it is current (reshaped
  2026-08-14 under dc-crea, i.e. today). The pull-vs-send distinction and the
  "no choosing among live dispatchers" rule are exactly the design calls an
  implementer would otherwise get wrong.
- dc-crea's full body (appears twice — see noise) is the authoritative ruling; the
  READ-FIRST rendering of it is the single most useful block after THE WORK.
- it-skpa / cl-3rx9 tell me where kind lives (registry entries, `coord::parse_kind`
  the one validation point) — that is the exact integration point.
- cl-br3p (kind-shaped wake, `coord::wake_shape`) and cl-s98g (multi-held state)
  tell me which coord machinery already exists to build on.
- The protocol block (one build at a time, raw test lines) is short and load-bearing
  on this machine.

## What reads as noise

- The spine shelf is a firehose: ~27 claims, most irrelevant to this item
  (q-unlink, unpack click-target, find-areas, claim-title-override, mention-index,
  atom-lint...). Maybe five are load-bearing for this work. No relevance ordering —
  alphabetical-ish by id, so the reader triages by hand.
- The 15-doc shelf is nearly pure noise here: fourteen dispatch reports listed with
  no signal about which (if any) I should read. I can guess (kind-shaped wake,
  multi-held, per-chat badge) but the brief doesn't rank them.
- dc-crea's body appears verbatim twice — once in READ-FIRST, once in the area's
  decisions shelf. Same for large parts of dc-ydvb's content echoed inside it-u8uf's
  READ-FIRST body. Dedup would cut a third of the payload.
- Six open threads all marked "NOT yours to settle" — fair as guardrails, but six
  full atom lines of them is mostly weight. (The graph knows this: th-uacu, "the
  brief is a dump", is literally one of the listed threads.)

## What I wish it told me that it didn't

- **Acceptance criteria.** The RETURN SPEC itself flags it: no acceptance recorded,
  "outcomes cannot be judged." The brief even prints the fix command — but a brief
  that knows its own acceptance is missing could refuse at dispatch time rather than
  hand the gap to the agent. I will have to derive acceptance from THE WORK prose
  and either set it or flag it.
- **Where the code is.** No file map. The spine claims name `coord::parse_kind`,
  `coord::wake_shape`, `src/surface.rs` in passing, but the brief never says
  "dispatch lives in src/X.rs; the registry is machine-local at path Y; last_seen
  is written by Z." First hour of the work is rediscovering that.
- **What `last_seen` currently is.** THE WORK leans on a "last_seen heartbeat" —
  the brief nowhere says whether the registry already records last_seen, what
  updates it, or what liveness threshold is expected. If it does not exist yet,
  that is a silent sub-project inside the item.
- **Purview fit semantics.** "purview fit" is load-bearing in the spec but the brief
  never defines a session's purview or how fit is judged (area overlap? glob
  overlap?). Presumably it exists in the registry — again, code archaeology.
- **The launcher.** "dispatcher-session.cmd today" — no path, no note on whether it
  is registered per session or a repo-level convention.
- **What the current fire-solo output looks like**, so the new surface can be
  written as a diff against it rather than reverse-engineered.

## Overall

The top third (THE WORK + READ-FIRST + write-set + protocol) is a genuinely good
brief — current, opinionated, with the design rationale attached. The bottom two
thirds is an area dump whose selection principle is "everything in cli", which
buries the five relevant spines among twenty-two bystanders and repeats whole
decision bodies. And the two things an implementer needs most beyond the ruling —
acceptance criteria and a map from the nouns in the spec to files/state on disk —
are exactly the two things missing. th-uacu (queued) already names this.
