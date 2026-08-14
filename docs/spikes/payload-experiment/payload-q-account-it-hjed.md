# Honest q-account — it-hjed work phase

Written before the stage-two exploration, from what I can actually ground. Epistemic
note first, because the honesty demand deserves it: at the time of writing, my context
retains the tail of the work phase verbatim (from the claim mint onward); the earlier
commands are grounded in the graph log (`graph/log/2026-08.jsonl`, lines 842-846 —
read directly as a file for this account, not via q) and in my own landing report,
not in quotable transcript. Where a command is reconstructed rather than quoted, I say so.
The log records writes and named events only; reads (find, guide, query) leave no log
line, so the read list below is transcript + memory, and I state its confidence.

## Every q command of the work phase, in order

1. `q join <token>` — certain (log: `join` op 06:43:21 binding `agent:a7948945a4a5222b1`,
   `brief` op 06:43:30). The entry point; rendered the brief I saved to
   `payload-copy-it-hjed.md`. A graph read by nature, but it IS the payload — not
   consultation beyond it.
2. — code phase: no q commands. Geography (which module owns find, where
   `contains_word` lives) came from Grep/Read over `src/`, not from the graph.
   Tests and build were cargo. I stopped the running `q serve` (pid 30840) with a
   process kill, not a q verb.
3. `q find cli` — near-certain, exact flags not quotable (my report: "ran the binary
   against this repo's graph after rebuild (output in transcript)"). Live verification
   of RETURN outcome 1. I believe I ran find only with the queries needed for
   verification; I cannot rule out one or two additional probe queries during
   eyeballing, and I won't pretend to a precision I don't have.
4. `q claim --about file:... --source file:src/queries.rs --title "`find-tiers`: ..."`
   — certain in substance (log: create + source link 06:50:14, `dispatch:"it-hjed"`
   stamped), exact flag order reconstructed. Minted cl-ybth. Its mint surface echoed
   relatedness suggestions (the "mentions 'every'" irony in my reflections).
5. `q guide` (piped through a Select-String for "claim") — certain, verbatim in
   transcript. **This is the one genuine graph consultation beyond the payload.**
   Purpose: find which rel is legal claim→claim before linking; my first instinct,
   `builds-on`, turned out illegal (decision→decision, doc→claim/decision only), and
   the matrix line sent me to `supports`.
6. `q link cl-89b5 supports cl-ybth --note "..."` — certain, verbatim (log 06:50:49).
7. `q serve` relaunched detached via Start-Process — certain, verbatim. Process
   management, not a read.
8. `q find zzqxv` — certain, verbatim. No-match message verification.

Not run at any point: `q open`, `q query` (queue/ready/shaping), `q view`, any read of
node files under `graph/` during the work phase. The READ-FIRST shelf's "q open <id>
for any neighborhood" invitation went unused.

## Did I consult the graph beyond the payload?

**Essentially no — one exception.** The design phase ran entirely from THE WORK
paragraph: it carried the tiers, the predicate spec, the divergence rationale, the
declined unification, and the atom_line constraint, so there was nothing I needed to
open. The gaps the payload did have (code geography, label wording) were code-shaped
gaps, and I closed them by reading `src/`, not the graph — the graph was never going
to contain the line number of `contains_word`. The exception is `q guide` at
citation time: the payload's ACTOR RULES order me to cite what I build on but never
say which edges are legal between which node types, so the payload manufactured its
own consultation. That read was necessary, small, and late — but it was real, so
"I never looked outside the payload" would be false by one command.

Why so little consultation? Two honest reasons. First, sufficiency: the settled
design was genuinely complete for this small item, and the READ-FIRST shelf visibly
scaled with the area's history rather than the item's needs, which taught me early
that opening more of it had poor expected value. Second — less flattering — economy:
q open on 25 spine claims was never going to happen under any time pressure, so the
payload's breadth converted into skimming, not reading. If something load-bearing had
been buried at shelf position 19, I would have missed it.

## A cost I did not know I was paying

The stage-two instructions revealed that q reads advance session watermarks and
heartbeats belonging to other chats. Nothing in the payload, CLAUDE.md, or the join
output warned me of read side-effects; my `q find` verifications and `q guide` were
run in innocence of it. The evidence was actually on screen — the `q link` confirmation
opened with "since your last read of 'cli' [area active] (ar-c7f5)", which is a
read-watermark surface announcing itself — and I did not register the implication.
If dispatched agents' q reads pollute per-session read state, the brief should say so
(and say which verbs are safe); an agent cannot honor an invariant it has never been
told exists.
