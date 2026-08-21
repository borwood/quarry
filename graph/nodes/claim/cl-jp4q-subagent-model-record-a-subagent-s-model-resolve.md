---
id: cl-jp4q
type: claim
title: '`subagent-model-record`: a subagent''s model resolves from the harness''s own record keyed by the injected agent id; --model becomes an override'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T13:44:30Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 6cf732b21cbf
- rel: supports
  to: it-6ekf
  at: 6
- rel: about
  to: ar-xa38
  at: 1
---

`subagent-model-record`: a subagent's model resolves from the harness's own record keyed by the injected agent id; --model becomes an override

THE DISCIPLINE THIS REMOVES. cl-dqt4 put ATTRIBUTION on the discipline road it had just taken IDENTITY off: q dispatch --model is a flag the dispatcher must remember on every fire, and omitting it while spawning on another model reproduced the original defect silently — against dc-zbxj's "identity is structural, never discipline", which is the fork th-e5ez holds for the user. cl-dqt4 was right that nothing ARRIVES at a subagent's seat: a PreToolUse firing there carries no model of any kind. The answer is on disk instead.

THE LAYOUT, AND IT IS UNDOCUMENTED AND HARNESS-INTERNAL. Beside the chat transcript the hook is handed sits a directory named for the chat, holding one pair of files per agent id: <projects>/<chat>/subagents/agent-<id>.jsonl (that agent's own turns) and agent-<id>.meta.json (its spawn record). The id is exactly what the session hook already injects as QUARRY_AGENT (cl-z6gc), so nothing has to arrive in a payload for this to be reachable. coord::subagent_records derives both paths from transcript_path, stripping ".jsonl" BY NAME rather than by with_extension, which would amputate whatever follows the last dot of any name that is not a .jsonl. This is the second dependency of the class cl-cv92 took on knowingly; taken twice it is a real exposure, and it is stated at both seats that read it — the coord doc comments and the guide's ENVIRONMENT section.

THE TRANSCRIPT LEADS, THE SIDECAR CATCHES, both halves measured over the 268 subagent records on this machine. The transcript names the RESOLVED model, so an arc's attribution reads in the same spelling the chat road produces, and it sees models the spawn call never named: of the 88 sidecars carrying no model key at all, 9 ran on a model their parent chat was not running — a claude-code-guide arc on haiku under a fable chat, four depth-2 Explore arcs on opus under fable chats, four arcs whose chat changed model after the spawn. A sidecar's silence is therefore not evidence of inheritance, and gating on it would have missed one unstamped subagent in ten. The sidecar carries only the ALIAS the dispatcher typed, and that alias is ambiguous across time — `opus` resolved to claude-opus-5 in 128 arcs and to claude-opus-4-8 in 43 — so no table maps it to a resolved id and it can only ever be the fallback. Its one job is the window before the agent's first turn reaches disk: the transcript is written incrementally and live (measured on this arc's own file mid-session), but an arc's FIRST fire is the q join shell, and there is no earlier entry to fall back on the way a chat has. A family-correct alias for that one shell beats the dispatcher's model, which is wrong outright.

THE SIDECHAIN FILTER INVERTS, and it is the one judgment that depends on whose file this is. cl-cv92 made the filter load-bearing over a CHAT's transcript; over a subagent's own it is fatal. Measured across the 268: every one of the 261 holding a readable assistant entry carries isSidechain:true on ALL of them, with not one non-sidechain entry anywhere among them. coord::scan_last_assistant_model therefore takes it as a parameter and wears two faces — last_assistant_model (a chat's file, skips) and last_agent_model (an agent's own, admits) — with every other filter unchanged on both roads: <synthetic> still skipped, a truncated tail still dropping its fragment first line, anything unrecognised still skipped rather than guessed at.

THE STAMP STILL OUTRANKS IT, which is what makes --model an OVERRIDE rather than dead weight. teach::session_hook_output resolves env, then badge_actor, then agent_actor, then the refreshed chat row. badge_model is agent-keyed and every subagent has a harness record, so putting the record first would make the flag unreachable in practice; the dispatcher's explicit word wins, and it is also the only road left where this layout does not exist at all.

NOTHING IS RECORDED. cl-dqt4 could not write a per-agent row because nothing at the agent's seat knew the answer; now something does, and a row still buys nothing — it would be empty in the one window where the read is hard, and it would need a lifecycle nobody owns. Derived at every fire like refreshed_chat_actor, tail-read so the cost does not grow with the arc. Measured in the real hook process against the real files, n=20 each: whole-hook medians of 12.0 ms on the chat road (a 2.48 MB transcript), 11.6 ms on the agent road (a 0.79 MB one), 11.4 ms for a record-miss that falls through to the chat, and 11.4 ms with no transcript at all. The agent road is if anything the cheaper of the two reads, and every delta sits inside run-to-run spread against a 10-second hook budget.

EVERY MISS DEGRADES, NEVER ERRORS: no transcript path to locate the directory from, no such agent recorded, neither file readable, or a future harness handing a subagent its own path instead of its chat's — each returns None and the chat row answers, which is the right answer for a spawn that genuinely inherits.

Pinned by a_subagents_model_resolves_from_the_harness_record_keyed_by_its_agent_id in tests/basic.rs: the two faces of the scan over the same bytes, the layout derivation with its suffixless counter-example, the sidecar's three shapes, the transcript-over-sidecar precedence, all four degradations, and end to end through the spawned binary — an UNSTAMPED subagent filing under its own model while its dispatcher's own shell keeps the chat's, the stamp overriding it, a freed badge falling to the agent's model rather than the dispatcher's, and an unknown agent inheriting. Each half probed against the regression it guards: removing the hook call site fails the unstamped assert with the chat's model still in the prefix; restoring the hard sidechain filter fails the native-mark assert; dropping the sidecar fallback fails the pre-first-turn assert; with_extension in place of the by-name strip fails the suffixless assert; and inverting the badge/record order fails the override assert — cl-dqt4's own instrument does NOT catch that inversion, since it runs with no harness record on disk.

CONFIRMED LIVE ON REAL HARNESS DATA, 2026-08-21: the real hook binary, handed this repo's own chat transcript and agent ab979fc19cd654d43 — a claude-code-guide arc at spawnDepth 2 whose sidecar carries no model key at all — injects QUARRY_ACTOR=claude-haiku-4-5-20251001, where the chat road says claude-opus-5.
