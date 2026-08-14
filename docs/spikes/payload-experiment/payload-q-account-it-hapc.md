# Honest q-account — it-hapc work phase

Written from my own transcript, before any further graph exploration.

## Every q command run, in order

1. `q join gey3etbyxb` — the join; printed the brief (the payload under review).
2. `q set it-hapc 'acceptance+=...' x4` — WRITE. Set the four acceptance lines the
   RETURN spec said were missing (the brief itself printed the fix command).
3. `q query ready` — READ. During live verification, to find a safe probe item
   (it returned "nothing dispatchable", which sent me to the shaping items I
   already knew from the payload — I picked it-xzp7 from the payload's area
   shelf, not from a query).
4. `q dispatch it-xzp7 --files "src/**"` — probe #1 (wake offer). Non-mutating by
   the feature's own design; run to verify the surface I had just built.
5. `q dispatch it-xzp7 --files "src/**"` — probe #2 (leave offer), after stamping
   the dispatcher heartbeat. Also non-mutating; heartbeat file restored after.
6. `q dispatch --help` — READ (help text, to confirm the new flag rendered).
7. `q claim --help` — READ (help text, to get the exact claim syntax).
8. `q claim '...' --title '...' --about ar-c7f5 --source file:src/coord.rs` —
   WRITE. Minted spine claim cl-9v7c after tests passed.

Not q commands, but honesty demands them here:
- I read graph FILES directly, all machine-state, none of them nodes:
  `graph/sessions.json` (the registry — I needed the on-disk shape to code
  against), `graph/.sessions-live.json` (heartbeat shape + who was live),
  `graph/.dispatch.json` (held entries, to pick a safe probe item).
- I WROTE `graph/.sessions-live.json` directly twice during verification
  (stamped the dispatcher fresh, then restored the original bytes). Machine-local
  dot-file, not a node or the log — but it is a direct write under graph/ that
  did not go through a q verb.
- Grep results incidentally crossed graph/ node files and the event log while I
  was searching for "launcher" and "dispatcher-session" — I read the matched
  lines (a handful of titles and log entries), not the files.

## Did I consult the graph beyond the payload?

**Effectively no.** I never ran `q open`, `q find`, or deliberately read a node
file during the work phase. Zero graph reads for design intent. The two
near-exceptions are honest but narrow: the machine-state dot-files (which are
coordination plumbing, and reading them WAS the work — the feature is built on
exactly those files), and stray grep lines.

## Why not

The payload's THE WORK paragraph plus dc-crea's unpacked body was a complete
behavioral spec — every design question I hit (leave vs wake, no ranking among
live dispatchers, solo always legitimate, surface never gate) was answered in
text I already had. All my open questions were CODE questions — where the
registry lives, what last_seen is, how dispatch flows, what the launcher
convention is — and the graph does not hold code; src/ does. So I went to src/.

The one place this bit me, mildly: I built the launcher offer (`wake_command`)
by reverse-engineering `q session set --launcher` in main.rs, when the graph
holds it-3pab ("the dispatch-session launcher"), a DONE item about exactly that
convention — I only learned its id when my own claim's touches listed it after
the work was finished. Same shape with dc-wngq: I leaned on "the dispatcher is
all-areas" for the vacuous-fit call, and my only source for that was a
mention-unpack INSIDE a spine claim's body, not anything the payload surfaced
deliberately. Both calls turned out right, but they were made on second-hand
fragments when first-hand nodes existed one `q open` away — and it did not occur
to me to look, because the payload felt complete. That is worth something for
the experiment: a payload that reads complete suppresses graph consultation,
for better (focus) and worse (near-misses like these).
