# Dispatch report — it-gwj7: the counter-voice and the sweep built

First live worktree dispatch (dc-g5x5 in anger). Agent report verbatim; dispatcher's
landing addendum follows.

## Outcomes against the RETURN SPEC

**· lands `wrap-sweep` — MET.** `q wrap` derives the session's deep-touch list from
the event log's per-session acts (`queries::deep_touches`, src/queries.rs), folds acts
per node, ranks by touch depth (ties by recency), renders it under
`framings::spend_down_header`, and asks the spend-down question (`framings::SPEND_DOWN`):
direct authorship taught (`q edit <id> --body-file`, state paragraphs / evidence notes /
dogears by the session's own hand), the do-dspv 2026-08-18 enrichment attitude
compressed into the prompt, and the Goodhart phrase naming volume ("a spend-down is
measured by what the next session no longer re-derives, never by how much was
written"). Once per boundary **by construction**: the sweep reads the same wrap-event
cursor `session_touched` plants, so a second wrap in the same boundary derives an
empty list and renders silence — verified live in a throwaway store (first wrap
rendered the sweep, second rendered nothing) and pinned by
`wrap_sweep_ranks_deep_touches_and_goes_silent_at_the_boundary`.

**· lands `counter-voice` — MET.** `teach::counter_voice` counts hook-observed turns
per acting session; at `COUNTER_VOICE_TURNS` (10) with no logged graph act from the
session, one reminder line rides the session hook's `additionalContext` channel. Any
logged act resets the counter and re-arms the line; a filing session never sees it by
construction. First encounter renders `framings::COUNTER_VOICE_DENSE`, later
encounters the light phrase; encounter state is machine-local working state at
`graph/.counter-voice.json` (the topic-queue species). Pinned end-to-end by
`counter_voice_fires_on_silence_resets_on_acts_and_teaches_dense_then_light` and
`counter_voice_rides_the_session_hook_and_skips_subagent_contexts` (the latter
exercises the real `session_hook_output` path).

**· holds the frame — MET.** Both surfaces are prompts, never gates (nothing refuses,
nothing blocks). No clock anywhere: the counter is turn-based only (the alerts 180s
throttle is pre-existing and untouched). No pressure surface beyond these two — no
wake lines, no wrap additions beyond the sweep, dispatch wake untouched, alert list
untouched.

## Test results (verbatim)

```
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.15s   (tests\basic.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s     (tests\store_lint.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s     (tests\surface_lint.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s     (tests\worktree_proof.rs)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s     (doc-tests)
```

Counts read directly from the raw lines, no filter. Release build finished clean in
the fork's own target\.

## Landing coordinates

- Branch `worktree-agent-a6d21fe349c232de8`, commit `3f8d060`, worktree under
  `.claude/worktrees/`.
- Files: src/teach.rs (counter-voice + hook wiring), src/queries.rs (`deep_touches`),
  src/framings.rs (four composed strings), src/main.rs (wrap sweep render),
  tests/basic.rs (three tests).
- Graph writes under the badge: veins cl-znh4 (`counter-voice`) and cl-amgy
  (`spend-down-sweep`), feature receipt cl-xrrx (`counter-voice-and-sweep`); edges
  cl-znh4/cl-amgy -[supports]-> cl-xrrx -[supports]-> it-gwj7; thread th-tm6f filed
  (q edit has no append; the spend-down vehicle teaches around it); three homework
  affirms cleared, plus the pre-existing it-yxx8 behind reviewed and affirmed.

## Follow-ups the agent flagged as owed

1. `q init --claude` after merge (teach.rs changed) — predicted a no-op beyond hook
   behavior. **Done at landing; confirmed no-op** (SKILL.md byte-identical, hooks
   already wired).
2. `.gitignore` line for `graph/.counter-voice.json` — outside the agent's write-set.
   **Done at landing**, beside its machine-local siblings; `.claude/worktrees/` added
   in the same stroke (harness worktree dirt was untracked noise).
3. Ratify-or-amend (dc-vzvf flag channel) — composed strings: `SPEND_DOWN`,
   `spend_down_header(n)`, `COUNTER_VOICE_DENSE`, `COUNTER_VOICE_LIGHT`, the injected
   wrapper `quarry [counter-voice] — ...`, and the `[depth N]` list line. Interpretive
   calls flagged for the user: turn = hook-observed shell call (the only turn signal
   the hook has); N=10 first setting; one fire per silence stretch, re-armed by the
   next act; graph act = any logged event (pure reads log nothing, don't reset);
   subagent-marked hook inputs neither count nor receive (their q acts still reset
   through the log); unbound chats never see it; depth weights create 5 /
   body,harvest 4 / set,dispatch 3 / edge-work 2 / else 1 (invented ladder, not
   measured); sweep gated on a nonempty list; sweep placed directly after the
   session-touched final-review block.

## Agent reflections (verbatim)

- **"Turn" is the weakest joint.** The ruling says N turns; the hook sees tool calls.
  A session in pure conversation — no shell calls — is invisible to the counter-voice
  entirely: the mechanism can only speak while the session works the shell but not
  the graph. That's arguably the right target (active work going unfiled), but it
  isn't what "N turns" naively promises, and nothing but a new hook event
  (UserPromptSubmit) would make it truer.
- **The depth weights are opinion.** Fully derived, one function, trivially retunable
  — but 5/4/3/2/1 is a ladder I invented, not measured. It survives the acceptance
  line ("ranked by touch depth") but should not be mistaken for settled ground.
- **Wrap now shows the same nodes twice** — the 2026-08-09 final-review list and the
  new ranked sweep sit adjacent. I deliberately didn't fold them (the final-review is
  its own user ruling), but the boundary read is heavier than before; a merge is a
  design call, not mine.
- **Worktree dispatch friction, wanted evidence:** join was flawless — one line,
  badge bound, brief fresh. The store pin's injection cuts the other way for
  sandboxing: every shell inherits `QUARRY_STORE=<canonical>`, and `q init` under a
  pin naming an empty dir refuses — I had to unset the pin and lean on cwd discovery
  to build a throwaway smoke store. A dispatched agent wanting a scratch store must
  know that move. Also, batch `q link` runs put my own just-made edges behind
  mid-batch (each link bumps its source node, so earlier inbound edges to it go
  stale) — three affirms to clean my own three links; correct, but noisy ergonomics.
  The write-gate's first-touch contract reminder and the mint echoes were genuinely
  useful. One pre-existing behind (it-yxx8 -> it-gwj7, from the dispatch flip) I
  reviewed and affirmed as ordinary homework.

## Dispatcher's landing addendum — collection-step evidence for th-mbbb

The first live run of the arc dc-g5x5 built, and the collection order that worked,
recorded here as evidence for th-mbbb (the process ruling stays the user's):

1. **The graph side needed no collection at all** — the agent's 14 badge-stamped
   events (claims, thread, affirms, edge homework) were sitting at the canonical
   store before its report arrived. dc-g5x5 held completely: nothing wrote the fork's
   graph/ copy; observed-vs-leased at harvest matched exactly (5 files, all inside
   the lease).
2. **Collection order used:** harvest (judgment surface) -> judge the diff -> merge
   the fork branch -> rebuild -> q init --claude -> register this report ->
   status=done + release -> commit -> remove worktree and branch.
3. **Fast-forward matters:** the branch based on main's tip, so the merge was a
   fast-forward and the merged blobs are byte-identical to what the agent's vein
   sources hashed — the claims' blob stamps stay true through collection. A true
   merge (diverged main) would re-write blobs and drift every vein sourced in the
   fork; ordering vein-sourcing dispatches to fast-forward, or affirming post-merge,
   is part of the open ruling.
4. **Hand-carried workarounds held, and are already filed:** q unreachable from the
   fork (no target\ checkout, PATH fix landed this session for future sessions) and
   the spawn line's "in {root}" misdirection (it-rmqy, ready). The spawn line was
   overridden in the dispatch prompt; the agent stayed in its fork throughout.
5. **New seam surfaced by the agent:** the injected store pin makes scratch/throwaway
   stores inside a badged context awkward (unset-the-pin is the move, nowhere
   taught). Belongs to th-mbbb's question 3 alongside the init-from-fork seam this
   dispatch sidestepped.
