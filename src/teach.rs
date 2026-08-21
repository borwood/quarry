//! The teaching surfaces: the embedded judgment-layer guide (`q guide`, and
//! the generated Claude skill), the C6 guard hook, and the installer that
//! distributes them into a host repo. One authority per fact: syntax lives in
//! --help, constraints live in error messages, judgment lives here, state
//! lives in the graph.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::Node;

pub const GUIDE: &str = r#"QUARRY — THE JUDGMENT LAYER
(mechanics live in `q --help` and `q <verb> --help`; this is when and why)

WHAT THIS IS
A work graph: decisions, claims, threads, items, docs, areas — typed nodes
with version-stamped edges, stored under graph/ in this repo. Files there
are never edited by hand (a hook denies it); every write goes through a q
verb, which bumps versions, stamps refs, and enforces the constraints.

SESSION SHAPE
Open with the three orientation queries: queue (threads awaiting the user,
answerable now), ready (items dispatchable now), shaping (upcoming work and
its blockers). Before designing anything, `q open` the areas and items it
touches — the neighborhood brief IS the context payload, and it shows what
changed behind every stale ref. Close a session by reviewing behind and
affirming ONLY what you actually re-read: affirm is a recorded act of
review, never a way to silence a marker. `q wrap` runs the whole boundary
lint — owed threads, stale refs, in-flight work, unfiled nodes, unrecorded
rulings, uncommitted graph changes. THE HOUSE RULE ON NAMES SPLITS: node
BODIES cite other nodes by bare id (dc-wwnk style — an id is an immutable
anchor where a title is a mutable label; render unpacks every id to its
current title, and mentioned-by backlinks derive from the same scan), while
conversation with the user keeps TITLES — ids belong in commands and node
bodies, never in prose to the user.

PROVENANCE HONESTY
Your analyses and proposals are assistant-provenance — the default. Mark
user provenance or ratification only for what the user actually said or
approved, in their words. The tool refuses an assistant decision that
settles or supersedes a user-provenance node (C3). That refusal is not an
obstacle: it is the queue telling you the call belongs to the user.

CLAIMS — EXTRACT, NEVER MINT WHILE WRITING
Prose stays prose: journals, spike reports, and design notes are docs,
written normally and registered whole. A claim node exists only when
something depends on a statement or kills it — extract it at that moment,
with its subjects and its GROUNDING: a source doc, a source file (the code
it was read off, blob-stamped), a method (how it was measured), or user
provenance. No free-floating assistant assertions (C2). A journal entry
written later can be attached as a source after the fact — never stop
mid-flow to manufacture one. If you are not building on it or refuting it,
it is not a claim yet.

THREADS — ANYTHING THAT NEEDS THE USER
A thread is a strand needing user input: a pick between options, a
discussion, a topic that needs a spike before it can be answered. Queue it
rather than asking ad hoc; give it depends-on edges to any intermediate
work it spawns, and it will surface only when answerable. Record the user's
ruling with the rule verb; the thread resolves and its dependents unblock.
OWNERSHIP IS DATA: the queue is the COMPLETE list of what the user owes.
If the user owes a call and no thread exists, mint one — never track a
user obligation in prose or memory. The hard edge is C3: agents cannot
settle what the queue holds.

ITEMS — SKETCH EARLY, DERIVE BLOCKAGE
Upcoming work enters as sketch the moment it is anticipated, with its
expected relationships as ordinary edges, and is refined as threads
resolve. Ready is a stored intent; whether anything still blocks an item is
always derived — never write "blocked" anywhere.

INTENT AND LINEAGE — NAMES ARE THE JOIN
An item IS intent: acceptance lines name the capabilities they intend in
the vein register (lands `name`: what it provides), and vein claim
titles carry the names that exist — same vocabulary, so the delta derives:
q query intent-delta reports intended-but-unlanded and landed-but-
unintended per shared area, both directions, never a sweep. Specs and
decisions record what they stand on with builds-on (builder → built-upon):
informational lineage, no status coupling — a refuted target never flips
its builders — but full staleness, so a killed capability puts the specs
standing on it behind, and blast walks reverse builds-on to enumerate
them. PRDs and tech specs are ordinary doc nodes (kind=prd, kind=spec)
registered whole; alternatives live in the body until something depends on
one (extraction-on-citation).

THE WITNESS PEN — PLEAS ARE THREADS
Acceptance authoring belongs to design (dc-p6z4); a session whose
registered kind is not design holds the WITNESS PEN (dc-mpg8):
transcription only — an item filed from that seat carries a witnessed
defect's negation as its contract, at filing, never at fire. Construction
holds the shape: a backticked register name in the line refuses (naming is
design's), authoring after the gate refused the same item refuses by
sequence, and the authoring badge can neither join nor solo-build the item
it authored — author is never executor. Every witness-authored line is
marked at authoring and rides the design wake's review channel until the
user ratifies or amends it (q witness; the user's word lands as
q witness <item> --ratify --by user). For anything with creative wiggle
room — work subsumable in planned systems, lines implying design intent,
repairs whose right shape is arguable — the channel is the PLEA: put the
evidence on the thread it informs, or open one (q new thread "<the plea>"
--about <area>). Unique context from your seat is leveraged there, never
self-authorized.

REFS AND STALENESS
Cite nodes and files with edges; stamps are automatic. behind is
information, not noise: severity 1 means something you cite was refuted or
superseded — read it before building further. When a claim falls, blast
enumerates everything leaning on it. That list IS the correction; there is
no sweep. Mutating verbs print the HOMEWORK an action creates — citers put
behind, work unblocked, threads made answerable — with the command that
addresses each. Do the homework (or queue it) before moving on. A bare id
in a body is a MENTION, not an edge: it unpacks at render and derives a
mentioned-by backlink, but it carries no stamp and blast and behind never
traverse it — a mention references; an edge leans. When something genuinely
stands on a mentioned node, record the real edge (q link) — extraction-on-
citation applied to references. A mislinked edge retires as a logged act
(q unlink): the edge leaves the frontmatter, the log keeps who and why,
and neither node bumps — bookkeeping, not content. Mint and edit echo
every resolved id's title beside it (read the echo: a wrong-but-real id
reads wrong there) and ask about id-shapes resolving to nothing; wrap
lints danglers.

ATTENTION SURFACES — DELIVERY AT CHOKE POINTS
Structure guarantees delivery and recorded acknowledgment, not reading;
these surfaces put the load-bearing material in channels the work cannot
avoid. Minting prints a TOUCHES line — candidates your new node's text
relates to (and prior mentions of a concept that just earned its node):
review and judge, link only what genuinely relates, never treat the list
as complete. The first write into an area you have not read this session
GATES with the area's derived read-first (a prior q open of the area
passes silently — open first and you never see the gate); after that,
foreign drift in an area prints inline when your own verb touches it.
DISPATCH IS A FETCH (dc-zbxj): q dispatch <item> runs one act — brief
logged, lease, in-flight, and a single-use join token in a ONE-LINE
spawn prompt. The agent runs q join <token>, which binds its identity
to the badge and renders the brief fresh from the graph — anything the
hand-off should carry belongs IN the graph, so the derived brief
carries it (q brief then q reserve remains the solo path; C8 makes any
lease follow a same-session brief; never hand-compose dispatch
context). The write hook holds a joined agent to the leased set,
DENIES unjoined writes into a dispatched zone (join first — the
teaching line names the fix), and OBSERVES everyone else: leaseless
code writes are never denied — they accrue to a machine-local
touched-set, nudge once at threshold with the items they resemble, and
surface at wrap. Verbs stamp the badge on the JOINED identity's events
(q query dispatch <item> replays what a dispatch wrote); the
dispatcher's own acts never stamp — stamping follows the work. THE
RETURN IS A REPORT, NOT A LANDING: an agent's "done" is a stop signal
— the dispatcher judges at q harvest <item> (observed-vs-leased,
report registration as a doc, then status=done and release by the
dispatcher's own hand). A landed capability registers
its vein: a claim titled name-first (`name`: what it provides), source
the code it was read off — the area's claims are its `veins` —
load-bearing structure — and building starts from them. The user's
single-thread conversation order lives in q queue (push/pop/front/drop)
— it is working state; what the user owes stays the graph's queued
threads.

PROJECT PROTOCOL — HOUSE RULES AS CONTENT
Quarry is an engine; this project's house rules are content, living IN the
graph. A protocol entry is a doc node, kind=protocol: its body is the
instruction; its fields pick the trigger (on=<verb>, node_type=<type>,
node_kind=<kind>) and the delivery tier. tier=inline rides the verb's
confirmation. tier=gate intercepts the FIRST matching attempt per session:
read the delivered context, do the work under it, then run q resume
<token> (your args are remembered). Author protocol like any doc:
  q new doc "journal charter" --kind protocol --body-file charter.md \
    --field on=new --field node_kind=journal --field tier=gate --about <area>
It versions, attaches, and goes stale like everything else. Engine-native
gates also exist: a steal demands --reason. Gates are for rare,
consequential, or authoring-shaped acts — never for frequent verbs.

ARCHIVING — SETTLED LEAVES LEAVE THE DEFAULT VIEW, NEVER THE GRAPH
Archived is a flag: ids resolve, edges hold, blast and behind always see
everything. Only settled statuses archive (status decides, never age);
parents with live children refuse — a parent indexes its archived
offspring, so aging leaves are found through the thing they were part of.
Docs and journals never archive (they are the record); areas retire by
status. Every surface that hides archived content says how many it hid and
how to reach them — nothing hides silently. Superseded decisions are
routine archive candidates; wrap lists what qualifies.

SESSIONS AND PARALLEL WORK
A main session's identity is its PURVIEW — a named set of areas in the
committed registry. The q CLI is agent-operated: the user never authors a
session by hand. SESSIONS PERSIST BY DEFAULT — defining one makes it
re-enterable forever, via the optional launcher script or by the user
saying "you're <name>" and the agent adopting; ephemeral (this chat only)
is the marked odd case, asked about at creation. WHEN THE USER ASKS FOR A
NEW SESSION ("start a q session covering X"): ask whether it should
persist across chats or is ephemeral; mint or verify the areas; register
the purview (session set — heed its overlap warning; distinct concerns
are the point, and deliberate overlap wants --shared leases there); offer
the launcher as a convenience, not a default; then make THIS chat the
session: q session adopt <name> (binds on the next shell call, env
injected automatically). WHEN A CHAT STARTS UNBOUND in a repo that has
sessions and the user launches into substantive work: before the first
graph write or dispatch, ask — adopt an existing session, or define a new
one (persistent or ephemeral)? The orient hook reminds you of exactly
this. AT THE WRAP OF AN EPHEMERAL SESSION, retiring it is the default
close act — do it and say so (q session retire); if its purview proved
durable during the session, instead offer converting it to a durable
session and let the user choose. WAKING AS A SESSION: run q session resume —
the derived handoff: holdings, your session's recent acts, arrivals from
other sessions, what the user is owed. Nobody writes a close block; the
wake brief is derived from the log and cannot be stale. Sessions start LEASELESS: browsing, design, and graph writes
never need a lease. Reserve AT DISPATCH — when an agent is about to touch
files — attaching the lease to the item it serves. Exclusive is the
default; a shared lease marks a co-write zone where presence-awareness
replaces mutual exclusion. Release explicitly when the arc lands (wrap
nags); a steal is always loud and logged. Cross-session requests need no
machinery: file an item into the other purview's areas with a depends-on
from your blocked item — their orientation surfaces it, and homework
reports to both sides when it lands.

EDGE MATRIX (names only)
about (anything → area or file) · part-of (hierarchy) · depends-on
(item/thread → item/thread/decision) · settles (decision → thread) ·
supports (claim/doc → decision/item/claim) · refutes (claim/doc →
claim/decision) · supersedes (same type) · source (claim → doc) ·
builds-on (decision → decision, doc → claim/decision — lineage, never
status)

ENVIRONMENT
QUARRY_ACTOR (your model identity) and QUARRY_SESSION are auto-injected by
the session hook for chats running in this repo — you should never set
them by hand. Set QUARRY_ACTOR manually only when operating outside hook
coverage (e.g. from a parent directory). If unset entirely, provenance
safely derives as assistant; user provenance is always explicit.
The dispatch badge is bound at q join, never exported by hand: the
session hook injects QUARRY_AGENT (in subagents) and QUARRY_CHAT
alongside SESSION/ACTOR, and q resolves badges from the machine-local
association map the join wrote — identity is structural, never
discipline (dc-zbxj). QUARRY_DISPATCH env survives ONLY as the
out-of-hook-coverage override (e.g. a parent-directory session).
Multi-held dispatch (dc-qyr5): a chat holds any number of live
dispatches; an item belongs to one chat (same-chat re-dispatch is free
with a fresh token; from another chat, q dispatch --steal --reason
takes the dispatch whole, loud and logged). Boundary verbs refuse
while a chat holds ANY live dispatch, enumerating each with its
q harvest command; a chat with no badge keeps its boundary verbs while
other chats' dispatches fly. Retire alone is scoped to the RETIREE
(it-e6wq): the chat's own session under a live badge refuses as above;
a retiree with a dispatch of its own in flight refuses toward that
dispatch's q harvest; a third session with no live dispatch retires
clean while unrelated badges fly. Harvest and release clear badge,
token, and associations per item.
The badge pins the store (dc-g5x5): one resolver owns the graph locale
for every verb and hook — the pin first (the spawn line's
q join --store, then QUARRY_STORE injected per badged shell, launcher
env winning), cwd discovery the fallback. A worktree dispatch works in
its fork while every q act, stamp, and observed write lands at the
canonical graph; blob stamps hash the file actually touched against
the store-relative path, and the fork's own graph/ copy is never
written. The pin lives and dies with the badge — harvest clears it.
"#;

const SKILL_FRONT: &str = "---\nname: quarry\ndescription: The work graph in this repo's graph/ directory — decisions, claims, threads, items, docs. Use at session start to get oriented (q query queue / ready / shaping), before design work (q open the relevant nodes), when recording a user ruling, extracting a claim, queueing a thread for the user, or closing a session (review behind, affirm what you re-read). All graph writes go through q verbs, never file edits.\n---\n\n";

/// The session hook (PreToolUse on Bash|PowerShell), doing two jobs in one
/// output envelope:
/// 1. Identity injection — if this chat's session_id is bound (or an
///    adopt-request is pending, consumed and bound here), rewrite the shell
///    command to carry QUARRY_SESSION via updatedInput. Launcher env wins:
///    when the process already carries QUARRY_SESSION, no injection.
/// 2. Cross-session alerts — when the session is known by either path,
///    throttled additionalContext deltas: filed into your purview, your
///    lease stolen, your work unblocked. Silence is the default state.
pub fn session_hook_output(store: &crate::store::Store, input: &str) -> Option<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(input).ok()?;
    let chat_id = v.get("session_id").and_then(|x| x.as_str());
    let agent_id = hook_agent_id(&v);
    let tool = v.get("tool_name").and_then(|x| x.as_str()).unwrap_or("");
    if let Some(cid) = chat_id {
        if let Some(req) = crate::coord::take_adopt_request(store) {
            let _ = crate::coord::bind_chat(store, cid, &req);
        }
    }
    let env_sess = std::env::var("QUARRY_SESSION").ok().filter(|s| !s.trim().is_empty());
    let env_actor = std::env::var("QUARRY_ACTOR").ok().filter(|s| !s.trim().is_empty());
    let bound = chat_id.and_then(|cid| crate::coord::chat_binding(store, cid));
    let session = env_sess.clone().or_else(|| bound.clone());
    // Actor injection applies to ANY chat (bound or not): the model recorded
    // at SessionStart, safety-prefixed so provenance derivation stays honest;
    // "claude" as the fallback when the harness gave no model. Env wins.
    //
    // The BADGE's model outranks the chat's for a joined subagent (it-xcvb).
    // The chat-actor map is keyed by CHAT and written at SessionStart, which
    // a subagent never fires, so a dispatched agent inherited its
    // dispatcher's model and every node it minted filed under a model that
    // did not write it — the house commit convention names the model that
    // did the work, and the graph was the only record of it. The dispatcher
    // stamps what it spawned with into the badge at the fire; this is where
    // that stamp is spent. Falls through to the chat model whenever nothing
    // is stamped, so an inheriting spawn keeps the answer that is right for
    // it.
    let inject_actor = if env_actor.is_none() {
        Some(
            crate::coord::badge_actor(store, agent_id.as_deref()).unwrap_or_else(|| {
                crate::coord::safe_actor(
                    &chat_id
                        .and_then(|cid| crate::coord::chat_actor(store, cid))
                        .unwrap_or_else(|| "claude".into()),
                )
            }),
        )
    } else {
        None
    };
    let inject_sess = if env_sess.is_none() { bound.clone() } else { None };
    // Chat identity injection applies to ANY chat with a session_id: per-chat
    // machine-local state (the dispatch badge) resolves by chat id inside q
    // processes, and the hook is the only place that knows it. Env wins.
    let env_chat = std::env::var("QUARRY_CHAT").ok().filter(|s| !s.trim().is_empty());
    let inject_chat = if env_chat.is_none() { chat_id.map(String::from) } else { None };
    // Agent identity injection (dc-zbxj): hooks run in a subagent carry an
    // agent id — the subagent's ONLY distinguishing mark (its session_id and
    // env match the parent chat's). q join binds by it; badge resolution
    // reads it first. Env wins here too.
    let env_agent = std::env::var("QUARRY_AGENT").ok().filter(|s| !s.trim().is_empty());
    let inject_agent = if env_agent.is_none() { agent_id.clone() } else { None };
    // The store pin rides the identity injection channel (dc-g5x5): a joined
    // identity's shells carry QUARRY_STORE so every q act lands at the
    // pinned graph, wherever cwd sits (a worktree fork). Launcher env wins,
    // like its siblings; injected only from a recorded pin — never from
    // discovery, which would pin every shell to its own cwd.
    let inject_store = if crate::store::env_pin().is_none() {
        let keys: Vec<String> = [
            agent_id.as_ref().map(|a| format!("agent:{}", a)),
            chat_id.map(|c| format!("chat:{}", c)),
        ]
        .into_iter()
        .flatten()
        .collect();
        crate::store::pinned_root(&keys).map(|r| r.display().to_string())
    } else {
        None
    };
    let mut updated_input: Option<serde_json::Map<String, serde_json::Value>> = None;
    if matches!(tool, "Bash" | "PowerShell")
        && (inject_sess.is_some()
            || inject_chat.is_some()
            || inject_agent.is_some()
            || inject_store.is_some()
            || (inject_actor.is_some() && (chat_id.is_some() || agent_id.is_some())))
    {
        if let Some(ti) = v.get("tool_input").and_then(|x| x.as_object()) {
            if let Some(cmd) = ti.get("command").and_then(|c| c.as_str()) {
                let mut prefix = String::new();
                if let Some(qs) = &inject_sess {
                    prefix += &match tool {
                        "Bash" => format!("export QUARRY_SESSION='{}'; ", qs),
                        _ => format!("$env:QUARRY_SESSION='{}'; ", qs),
                    };
                }
                if let Some(qa) = &inject_actor {
                    prefix += &match tool {
                        "Bash" => format!("export QUARRY_ACTOR='{}'; ", qa),
                        _ => format!("$env:QUARRY_ACTOR='{}'; ", qa),
                    };
                }
                if let Some(qc) = &inject_chat {
                    prefix += &match tool {
                        "Bash" => format!("export QUARRY_CHAT='{}'; ", qc),
                        _ => format!("$env:QUARRY_CHAT='{}'; ", qc),
                    };
                }
                if let Some(qg) = &inject_agent {
                    prefix += &match tool {
                        "Bash" => format!("export QUARRY_AGENT='{}'; ", qg),
                        _ => format!("$env:QUARRY_AGENT='{}'; ", qg),
                    };
                }
                if let Some(qr) = &inject_store {
                    prefix += &match tool {
                        "Bash" => format!("export QUARRY_STORE='{}'; ", qr),
                        _ => format!("$env:QUARRY_STORE='{}'; ", qr),
                    };
                }
                let mut u = ti.clone();
                u.insert("command".into(), serde_json::json!(format!("{}{}", prefix, cmd)));
                updated_input = Some(u);
            }
        }
    }
    let alert = session.as_deref().and_then(|s| alerts(store, s));
    // The counter-voice (dc-hzrm): a subagent's shells carry the parent
    // chat's session, but its context is not the conversation the reminder
    // speaks for — an agent-marked hook input neither counts a turn nor
    // receives the line. Subagent q acts still stamp the session and reset
    // the counter through the log: acts always speak, turns are the
    // parent's own.
    let voice = if agent_id.is_none() {
        session.as_deref().and_then(|s| counter_voice(store, s))
    } else {
        None
    };
    let context = match (alert, voice) {
        (Some(a), Some(v)) => Some(format!("{}\n{}", a, v)),
        (a, v) => a.or(v),
    };
    if updated_input.is_none() && context.is_none() {
        return None;
    }
    let mut hso = serde_json::Map::new();
    hso.insert("hookEventName".into(), serde_json::json!("PreToolUse"));
    if let Some(u) = updated_input {
        hso.insert("updatedInput".into(), serde_json::Value::Object(u));
    }
    if let Some(a) = context {
        hso.insert("additionalContext".into(), serde_json::json!(a));
    }
    Some(serde_json::json!({ "hookSpecificOutput": serde_json::Value::Object(hso) }))
}

#[derive(Serialize, Deserialize, Clone)]
struct AlertCursor {
    /// Log position (legacy timestamp cursors convert on first read).
    cursor: crate::coord::Cursor,
    /// Wall-clock throttle stamp — stays a timestamp; it gates check
    /// frequency, not log position.
    checked: String,
}

fn cursors_path(store: &crate::store::Store) -> PathBuf {
    store.root.join("graph").join(".alert-cursors.json")
}

/// Throttled alert check: at most every 180s per session; first call only
/// plants the cursor (never dumps history); cursor advances on every check.
fn alerts(store: &crate::store::Store, session: &str) -> Option<String> {
    use std::collections::BTreeMap;
    let now = crate::store::Store::now();
    let mut map: BTreeMap<String, AlertCursor> = fs::read_to_string(cursors_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let write = |m: &BTreeMap<String, AlertCursor>| {
        if let Ok(s) = serde_json::to_string_pretty(m) {
            let _ = fs::write(cursors_path(store), s + "\n");
        }
    };
    let Some(cur) = map.get(session).cloned() else {
        // First check plants the cursor at the log's current end — history
        // is never dumped.
        let end = store.read_log().map(|l| l.len()).unwrap_or(0);
        map.insert(
            session.into(),
            AlertCursor { cursor: crate::coord::Cursor::Index(end as u64), checked: now },
        );
        write(&map);
        return None;
    };
    {
        use time::format_description::well_known::Rfc3339;
        if let Ok(t) = time::OffsetDateTime::parse(&cur.checked, &Rfc3339) {
            if (time::OffsetDateTime::now_utc() - t).whole_seconds() < 180 {
                return None;
            }
        }
    }
    let all = store.load_all().ok()?;
    let reg = crate::coord::load_sessions(store);
    let log = store.read_log().ok()?;
    let lines = if let Some(p) = reg.get(session) {
        let ids: Vec<&str> = p.areas.iter().map(|s| s.as_str()).collect();
        alerts_between(&all, &log, session, &ids, crate::coord::cursor_index(&cur.cursor, &log))
    } else {
        vec![]
    };
    map.insert(
        session.into(),
        AlertCursor { cursor: crate::coord::Cursor::Index(log.len() as u64), checked: now },
    );
    write(&map);
    if lines.is_empty() {
        None
    } else {
        Some(format!(
            "quarry [session {}] — since your last check:\n{}\n(details: q session resume)",
            session,
            lines.iter().map(|l| format!("  · {}", l)).collect::<Vec<_>>().join("\n")
        ))
    }
}

/// The pure alert computation — the ratified closed list, nothing else:
/// (a) filed into your purview by another session, (b) your lease stolen
/// (with the logged reason), (c) your work unblocked by another session's
/// landing. Citation staleness stays pull-only by ruling.
pub fn alerts_between(
    all: &[Node],
    log: &[serde_json::Value],
    session: &str,
    area_ids: &[&str],
    from: usize,
) -> Vec<String> {
    use crate::model::Node as N;
    let ref_of = |id: &str| {
        all.iter()
            .find(|n| n.front.id == id)
            .map(|n| crate::surface::atom_ref(&crate::surface::atom(all, n)))
            .unwrap_or_else(|| format!("({})", id))
    };
    let mut out: Vec<String> = Vec::new();
    for ev in log.iter().skip(from) {
        let ev_sess = ev.get("session").and_then(|x| x.as_str());
        let op = ev.get("op").and_then(|x| x.as_str()).unwrap_or("");
        let node_id = ev.get("node").and_then(|x| x.as_str()).unwrap_or("");
        let node: Option<&N> = all.iter().find(|n| n.front.id == node_id);
        match op {
            "create" => {
                if ev_sess.map_or(false, |s| s != session) {
                    if let Some(n) = node {
                        if crate::coord::in_purview(n, area_ids)
                            && !matches!(n.front.status.as_str(), "done" | "dropped" | "resolved" | "superseded")
                        {
                            out.push(format!(
                                "new from {}: {} — q open {}",
                                ev_sess.unwrap_or("?"),
                                crate::surface::atom_line(&crate::surface::atom(all, n)),
                                n.front.id
                            ));
                        }
                    }
                }
            }
            "steal" => {
                if ev.get("from_session").and_then(|x| x.as_str()) == Some(session) {
                    let victim = ev.get("from_item").and_then(|x| x.as_str()).unwrap_or("?");
                    let reason = ev
                        .get("reason")
                        .and_then(|x| x.as_str())
                        .unwrap_or("no reason recorded");
                    out.push(format!(
                        "your lease on {} was taken by {}: {}",
                        ref_of(victim),
                        ev_sess.unwrap_or("?"),
                        reason
                    ));
                }
            }
            "set" => {
                let landed = ev
                    .get("fields")
                    .and_then(|f| f.as_array())
                    .map_or(false, |fs| {
                        fs.iter().any(|x| {
                            matches!(x.as_str(), Some("status=done") | Some("status=resolved"))
                        })
                    })
                    || (ev.get("field").and_then(|x| x.as_str()) == Some("status")
                        && matches!(ev.get("to").and_then(|x| x.as_str()), Some("done") | Some("resolved")));
                if landed && ev_sess.map_or(false, |s| s != session) {
                    for d in all.iter().filter(|d| {
                        d.front.edges.iter().any(|e| e.rel == "depends-on" && e.to == node_id)
                            && crate::coord::in_purview(d, area_ids)
                            && crate::queries::live_blockers(all, d).is_empty()
                    }) {
                        out.push(format!(
                            "unblocked: {} — {} landed {}",
                            crate::surface::atom_line(&crate::surface::atom(all, d)),
                            ev_sess.unwrap_or("?"),
                            ref_of(node_id)
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    out.dedup();
    if out.len() > 6 {
        let extra = out.len() - 6;
        out.truncate(6);
        out.push(format!("…and {} more (q session resume)", extra));
    }
    out
}

// ── the counter-voice (dc-hzrm) ────────────────────────────────────────────
//
// Every ambient surface speaks the graph's frame; this is the one line that
// speaks for the prose channel — anything floated in conversation that has
// no node. Fired on graph silence, never the clock: N hook-observed turns
// since the acting session's last graph act draw one reminder line into
// injected context; any graph act resets the counter; a session that is
// filing never sees it. First encounter renders the dense teaching,
// subsequent encounters a light phrase — the encounter state is
// machine-local working state (the topic-queue species), never graph data.

/// The silence threshold, in turns. A turn is operationally a session-hook
/// firing (a Bash/PowerShell tool call by the acting session's own chat) —
/// the only turn signal quarry can observe; conversation without tool calls
/// is invisible to the hook by construction. Deliberately a first setting:
/// habituation is the mechanism class's known failure mode (dc-hzrm queues
/// its defense separately), so the threshold leans quiet.
pub const COUNTER_VOICE_TURNS: u64 = 10;

#[derive(Serialize, Deserialize, Clone, Default)]
struct VoiceState {
    /// Hook-observed turns since the session's last graph act.
    turns: u64,
    /// Log length at last check — events past it are the unseen tail.
    seen: u64,
    /// The reminder already fired for this silence stretch: one line per
    /// stretch, never a nag per turn — the next graph act re-arms it.
    fired: bool,
    /// This session has met the dense teaching (machine-local encounter
    /// state): later encounters render the light phrase.
    encountered: bool,
}

fn voice_path(store: &crate::store::Store) -> PathBuf {
    store.root.join("graph").join(".counter-voice.json")
}

/// One counter-voice check for the acting session: counts the turn, resets
/// on any logged graph act from the session since the last check, and
/// returns the reminder line exactly when the silence threshold is crossed.
/// First sight of a session plants the cursor at the log's end — history
/// never counts as silence. A graph act is any logged event stamped with
/// the session (writes, briefs, wraps); pure reads log nothing and reset
/// nothing — reading is not filing.
pub fn counter_voice(store: &crate::store::Store, session: &str) -> Option<String> {
    use std::collections::BTreeMap;
    let log = store.read_log().ok()?;
    let mut map: BTreeMap<String, VoiceState> = fs::read_to_string(voice_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let mut st = map.get(session).cloned().unwrap_or_else(|| VoiceState {
        seen: log.len() as u64,
        ..VoiceState::default()
    });
    let seen = (st.seen as usize).min(log.len());
    let acted = log[seen..]
        .iter()
        .any(|ev| ev.get("session").and_then(|v| v.as_str()) == Some(session));
    if acted {
        st.turns = 0;
        st.fired = false;
    } else {
        st.turns += 1;
    }
    st.seen = log.len() as u64;
    let mut out = None;
    if st.turns >= COUNTER_VOICE_TURNS && !st.fired {
        st.fired = true;
        let text = if st.encountered {
            crate::framings::COUNTER_VOICE_LIGHT
        } else {
            crate::framings::COUNTER_VOICE_DENSE
        };
        st.encountered = true;
        out = Some(format!("quarry [counter-voice] — {}", text));
    }
    map.insert(session.into(), st);
    if let Ok(s) = serde_json::to_string_pretty(&map) {
        let _ = fs::write(voice_path(store), s + "\n");
    }
    out
}

/// The agent id a hook input carries when the harness runs the hook inside
/// a subagent (the harness fact dc-zbxj stands on). The key shape is read
/// DEFENSIVELY — several plausible spellings — because the field's presence
/// is established but its exact name may vary across harness versions;
/// whichever matches, the value injects as QUARRY_AGENT and keys the
/// association map.
pub fn hook_agent_id(v: &serde_json::Value) -> Option<String> {
    for k in [
        "agent_id",
        "agentId",
        "agent_session_id",
        "agentSessionId",
        "subagent_id",
        "subagentId",
    ] {
        if let Some(s) = v.get(k).and_then(|x| x.as_str()) {
            if !s.trim().is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// The file path a Write/Edit/NotebookEdit hook input targets, if any.
pub fn write_target(input: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(input).ok()?;
    let tool = v.get("tool_name")?.as_str()?;
    if !matches!(tool, "Write" | "Edit" | "NotebookEdit") {
        return None;
    }
    let ti = v.get("tool_input")?;
    ti.get("file_path")
        .or_else(|| ti.get("notebook_path"))?
        .as_str()
        .map(String::from)
}

/// The lease layer of the write guard (the dispatch chain's last link).
pub enum LeaseCheck {
    Deny(String),
    Warn(String),
    Allow,
}

/// Check a repo-relative write path against the live leases. Under a badge,
/// the BADGE'S lease is the contract: inside its globs allows, outside
/// denies. With no badge, the JOIN GATE (dc-zbxj) guards every zone a live
/// dispatch leases: a write there from the holder session or an unbound
/// context denies with the teaching line — under join-as-fetch the
/// dispatcher does not work the leased zone, and the unjoined agent's fix
/// is q join. A foreign EXCLUSIVE lease still denies for everyone (C7).
/// Solo leases (reserve without dispatch) keep the holder-session allow and
/// the scope-creep warning. The contract is REPO-RELATIVE: a path outside
/// the host repo (absolute — scratchpads, temp files) is never scope creep,
/// never contract material, and always allowed here. `dispatched` lists the
/// item ids some live dispatch holds.
pub fn lease_check(
    leases: &[crate::coord::Lease],
    session: Option<&str>,
    badge: Option<&str>,
    dispatched: &[String],
    rel_path: &str,
) -> LeaseCheck {
    if leases.is_empty() || rel_path.starts_with("graph/") {
        return LeaseCheck::Allow;
    }
    if rel_path.starts_with('/') || rel_path.contains(':') {
        return LeaseCheck::Allow; // outside the host repo — not this graph's concern
    }
    if let Some(item) = badge {
        // The badge's own lease is the contract — session identity does not
        // enter it (a joined agent usually has none of its own). A badge
        // with no lease is a research dispatch: every code write is outside.
        let covered = leases.iter().any(|l| {
            l.item == item && l.globs.iter().any(|g| crate::coord::globs_overlap(g, rel_path))
        });
        if covered {
            return LeaseCheck::Allow;
        }
        return LeaseCheck::Deny(format!(
            "dispatch write outside the leased write-set: {} is not covered by the lease for {} — the brief's write-set is the contract; ask the dispatcher to extend the lease.",
            rel_path, item
        ));
    }
    // The join gate (dc-zbxj), C8's move applied to the hand-off: a zone a
    // live dispatch leases belongs to the JOINED agent. The holder session
    // (the dispatcher — its chores live outside the zone) and the unbound
    // context (an agent that skipped its join) both deny and teach; a
    // genuinely foreign session falls through to C7 below, which names the
    // holder instead.
    if let Some(l) = leases.iter().find(|l| {
        dispatched.contains(&l.item)
            && session.map_or(true, |s| l.session == s)
            && l.globs.iter().any(|g| crate::coord::globs_overlap(g, rel_path))
    }) {
        return LeaseCheck::Deny(format!(
            "this zone belongs to a dispatch: {} is inside the write-set leased for \"{}\" ({}). Join first — run the `q join <token>` line from your spawn prompt; it binds your identity to the badge and renders the brief (no spawn prompt? ask the dispatcher). The dispatcher's own chores live outside the leased zone.",
            rel_path, l.item_title, l.item
        ));
    }
    let foreign_exclusive = leases.iter().find(|l| {
        session.map_or(true, |s| l.session != s)
            && !l.shared
            && l.globs.iter().any(|g| crate::coord::globs_overlap(g, rel_path))
    });
    if let Some(f) = foreign_exclusive {
        return LeaseCheck::Deny(format!(
            "C7: {} is inside session {}'s exclusive lease ({:?} for \"{}\") — coordinate with the holder, mark a co-write zone with --shared leases, or steal loudly (q reserve --steal).",
            rel_path, f.session, f.globs, f.item_title
        ));
    }
    // Solo holdings only from here down: a dispatched lease is the agent's
    // zone, not the dispatcher's own hands — the dispatcher's outside writes
    // are ordinary leaseless observation (dc-cc76), never scope creep.
    let own_solo = |l: &&crate::coord::Lease| {
        session.map_or(false, |s| l.session == s) && !dispatched.contains(&l.item)
    };
    let covered_own = leases.iter().filter(own_solo).any(|l| {
        l.globs.iter().any(|g| crate::coord::globs_overlap(g, rel_path))
    });
    let holds_any = leases.iter().any(|l| own_solo(&l));
    if holds_any && !covered_own {
        return LeaseCheck::Warn(format!(
            "quarry: this write ({}) lands outside every lease your session holds — scope creep, or a lease wanting extension?",
            rel_path
        ));
    }
    LeaseCheck::Allow
}

/// The observation layer riding the write guard, AFTER allow/warn — it never
/// denies (a blocked write breeds junk leases; prompts, not gates). Accrues
/// the touched path (O(1) append, no graph load), echoes the dispatch
/// contract on the first badged write, raises a badge-keyed drift notice off
/// the dispatch cursor (throttled — never a per-write log scan), and fires
/// the leaseless threshold nudge once per session with a derived item match.
/// Returns additionalContext lines for the hook envelope.
pub fn observe_write(
    store: &crate::store::Store,
    leases: &[crate::coord::Lease],
    session: Option<&str>,
    badge: Option<&str>,
    rel_path: &str,
) -> Vec<String> {
    use crate::coord;
    if rel_path.starts_with("graph/") {
        return vec![]; // graph state is not code; verbs carry their own record
    }
    if rel_path.starts_with('/') || rel_path.contains(':') {
        // A path that never resolved store-relative. Leaseless it is simply
        // outside this graph's arc (scratchpads, temp files) and stays
        // unrecorded. Under a badge it is ACCOUNTING (it-bj3b): the raw path
        // is recorded, marked unresolved, and harvest renders it — a badged
        // write's resolution failure is visible bookkeeping, never a
        // silently dropped fact.
        if badge.is_some() {
            let key = coord::touch_key(badge, session);
            if !coord::touches_for(store, &key).iter().any(|t| t.path == rel_path) {
                coord::accrue_touch_ext(store, &key, rel_path, None, true);
            }
        }
        return vec![];
    }
    let mut out: Vec<String> = Vec::new();
    let key = coord::touch_key(badge, session);
    let prior = coord::touched_for(store, &key);
    let is_new = !prior.iter().any(|p| p == rel_path);
    if is_new {
        coord::accrue_touch(store, &key, rel_path);
    }
    if let Some(b) = badge {
        // First badged write: echo the contract captured at dispatch time —
        // the write path reads one small state file, never the graph. The
        // echo names the user-owned-calls rule (it-f6c2): the call itself
        // is semantic and trips no hook, so the guaranteed in-flight
        // channel says the landing once, where the first write lands.
        if prior.is_empty() {
            if let Some(d) = coord::dispatch_for_item(store, b) {
                out.push(format!(
                    "first write under dispatch {} — the contract: item \"{}\"; write-set {:?} (outside writes deny); RETURN: {} acceptance line(s), accepted by outcome. {} Report and stop — landing belongs to the dispatcher. (q brief {} re-renders the full brief.)",
                    b, d.item_title, d.globs, d.acceptance.len(), crate::framings::USER_OWNED_ECHO, b
                ));
            }
        }
        // Badge-keyed drift notice: has the dispatched item moved since the
        // brief? Cursor-incremental from the dispatch state, throttled.
        if let Some(mut d) = coord::dispatch_for_item(store, b) {
            let stale = {
                use time::format_description::well_known::Rfc3339;
                time::OffsetDateTime::parse(&d.checked, &Rfc3339)
                    .map(|t| (time::OffsetDateTime::now_utc() - t).whole_seconds() >= 120)
                    .unwrap_or(true)
            };
            if stale {
                if let Ok(log) = store.read_log() {
                    let drifted: Vec<String> = log
                        .iter()
                        .skip(d.cursor as usize)
                        .filter(|ev| {
                            ev.get("node").and_then(|v| v.as_str()) == Some(b)
                                && matches!(
                                    ev.get("op").and_then(|v| v.as_str()),
                                    Some("set") | Some("body") | Some("link")
                                )
                        })
                        .map(|ev| {
                            format!(
                                "[{}] by {}",
                                ev.get("op").and_then(|v| v.as_str()).unwrap_or("?"),
                                ev.get("actor").and_then(|v| v.as_str()).unwrap_or("?")
                            )
                        })
                        .collect();
                    if !drifted.is_empty() {
                        out.push(format!(
                            "dispatch drift: \"{}\" ({}) changed since your brief ({}) — the contract may have moved; re-read: q open {}",
                            d.item_title, b, drifted.join(", "), b
                        ));
                    }
                    d.cursor = log.len() as u64;
                    d.checked = crate::store::Store::now();
                    let _ = coord::save_dispatch(store, &d);
                }
            }
        }
        return out;
    }
    // Leaseless: sessions holding leases already get the scope-creep warn;
    // the threshold nudge is for the genuinely leaseless arc taking shape.
    let holds_any = session.map_or(false, |s| leases.iter().any(|l| l.session == s));
    if !holds_any && is_new && prior.len() + 1 == coord::LEASELESS_NUDGE_THRESHOLD {
        let mut touched = prior.clone();
        touched.push(rel_path.to_string());
        // The match loads the graph — once, at the threshold crossing, never
        // on the steady write path.
        let matched = store
            .load_all()
            .map(|all| {
                crate::queries::items_matching_files(&all, &touched)
                    .into_iter()
                    .map(|n| crate::surface::atom_line(&crate::surface::atom(&all, n)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let hint = if matched.is_empty() {
            "no item's write-set or file refs cover these — if the arc is real, sketch it: q new item \"...\" --about <area>".to_string()
        } else {
            format!(
                "this resembles {} — take it up: q brief <item>, then q reserve <item> --files <globs> (or q dispatch <item>)",
                matched.join(", ")
            )
        };
        out.push(format!(
            "quarry: {} source files touched this session with no lease ({}) — an arc is forming. A lease is an arc declaration, not permission (leaseless writes never deny), but a declared arc gets vein extraction and presence. {}",
            coord::LEASELESS_NUDGE_THRESHOLD,
            touched.join(", "),
            hint
        ));
    }
    out
}

/// Quote-aware shell tokenization for the write-shape parser: whitespace
/// splits words outside quotes; command separators (`;`, `|`, `&`, parens)
/// and input redirects (`<`, heredoc openers) push None — a boundary; runs
/// of `>` split into their own token even glued to a neighbor (`2>file`),
/// so the scan can pair each redirect with the token that follows it.
///
/// An unquoted newline is a command separator too (it-dprv), unless the line
/// continues — a bash `\` or a PowerShell backtick standing as its own word
/// before it, which joins the two lines instead. A quoted newline is not a
/// separator at all: prose inside an argument stays one token.
///
/// Two multi-line literals are consumed WHOLE, each as one opaque token,
/// because both carry PROSE through a parser that would otherwise read it as
/// command text.
///
/// A PowerShell here-string (`@'` … newline `'@`, or the `@"` form) —
/// it-ap3x. This is the repo's own documented way to hand a multi-line commit
/// message to git: an apostrophe in it ("main.rs's arm") would otherwise open
/// a quote the tokenizer never closes, desyncing every token after it — the
/// closing `'@` re-opens the quote instead, and the command's real tail
/// (`2>&1 | tail -20`) arrives as one bogus word behind a `>` from an
/// unrelated `<…>` in the prose. Swallowing the literal keeps the tail
/// tokenizing correctly.
///
/// A bash heredoc (`<<EOF`, `<<'EOF'`, `<<-EOF`) — it-dt68, the same class one
/// channel over. The `<<` pushes a boundary, so nothing desyncs, but every
/// line of the body used to tokenize as ordinary words: a message line reading
/// `touch foo` minted `foo` as a perfectly plausible touched path, and the
/// judgment seat cannot tell such a mint from a real write. The delimiter is
/// tracked from the opener and the body is consumed at the newline that ends
/// the opener line (bash's own ordering — a redirect after the delimiter,
/// `cat <<EOF > out.txt`, still belongs to the command) through its
/// line-initial terminator, or to end of input when it has none. The token
/// carries the `<<` it opened with, so it is opaque by construction.
fn shell_tokens(command: &str) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    // Heredocs opened on the line being tokenized, in the order their bodies
    // arrive: (opener text as written, delimiter, `<<-` tab-stripping form).
    let mut pending: Vec<(String, String, bool)> = Vec::new();
    let flush = |cur: &mut String, out: &mut Vec<Option<String>>| {
        if !cur.is_empty() {
            out.push(Some(std::mem::take(cur)));
        }
    };
    let boundary = |out: &mut Vec<Option<String>>| {
        if out.last() != Some(&None) {
            out.push(None);
        }
    };
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                cur.push(c);
            }
            continue;
        }
        match c {
            // Here-string opener at the start of a word: swallow the literal
            // through its line-initial terminator (or to end of input when it
            // has none) as a single token. Opaque by construction — it always
            // carries the quote character it opened with, which no write
            // target does.
            '@' if cur.is_empty() && matches!(chars.peek(), Some('\'') | Some('"')) => {
                let q = *chars.peek().expect("peeked");
                chars.next();
                let mut body = String::from("@");
                body.push(q);
                let mut after_newline = false;
                while let Some(b) = chars.next() {
                    if after_newline && b == q && chars.peek() == Some(&'@') {
                        chars.next();
                        body.push(b);
                        body.push('@');
                        break;
                    }
                    after_newline = b == '\n';
                    body.push(b);
                }
                out.push(Some(body));
            }
            '\'' | '"' => quote = Some(c),
            // The newline that ends an opener line: every heredoc opened on it
            // now takes its body, in order, each as one opaque token followed
            // by a boundary — so a real command after the terminator is read
            // as its own command, not as an argument of the prose.
            '\n' if !pending.is_empty() => {
                flush(&mut cur, &mut out);
                for (opener, delim, strip) in std::mem::take(&mut pending) {
                    let mut body = opener;
                    body.push('\n');
                    loop {
                        let mut line = String::new();
                        let mut saw_newline = false;
                        while let Some(&b) = chars.peek() {
                            chars.next();
                            if b == '\n' {
                                saw_newline = true;
                                break;
                            }
                            line.push(b);
                        }
                        // Trailing whitespace (a CRLF's `\r` included) never
                        // makes a terminator a body line; leading whitespace
                        // only under the `<<-` form.
                        let t = line.trim_end();
                        let done = if strip { t.trim_start() == delim } else { t == delim };
                        body.push_str(&line);
                        if saw_newline {
                            body.push('\n');
                        }
                        if done || !saw_newline {
                            break;
                        }
                    }
                    out.push(Some(body));
                    boundary(&mut out);
                }
            }
            // A CRLF's `\r` belongs to the newline behind it, never to the
            // word in front: skipping it here leaves the continuation check
            // below reading the real last character of the line.
            '\r' if chars.peek() == Some(&'\n') => {}
            // A LINE CONTINUATION, so the two lines are one command: bash's
            // trailing `\` and PowerShell's trailing backtick, each STANDING
            // AS ITS OWN WORD — the idiomatic form in both shells. It vanishes
            // exactly as bash removes a backslash-newline pair, so the tail
            // joins the command in front of it instead of starting a new one.
            // The word rule is what keeps the exception from minting: `\` also
            // ends a Windows directory path, and both shells share this
            // parser. Measured, a rule reading any trailing `\` turns a
            // `Copy-Item a.rs B:\dest\` line followed by a `touch src/b.rs`
            // line into the single target ["B:\desttouch"] — a plausible path
            // nobody wrote, the expensive direction — and it buys nothing for
            // the glued form it would cover: a `cp a.rs\` line followed by
            // `src/b.rs` parses to [] either way, since bash's own joining
            // makes that destination "a.rssrc/b.rs".
            '\n' if cur == "\\" || cur == "`" => cur.clear(),
            // AN UNQUOTED NEWLINE ENDS THE COMMAND BEFORE IT (it-dprv). It
            // used to be ordinary whitespace, so a multi-line command was ONE
            // command and only its first word was ever read as a command word
            // — minting line two's `touch` as a plausible touched path, and
            // consuming line two's real `cp` as an argument nothing rescanned.
            '\n' => {
                flush(&mut cur, &mut out);
                boundary(&mut out);
            }
            c if c.is_whitespace() => flush(&mut cur, &mut out),
            ';' | '|' | '&' | '(' | ')' => {
                flush(&mut cur, &mut out);
                boundary(&mut out);
            }
            // Input redirects. A run of exactly two is a heredoc opener: read
            // its delimiter here (the body waits for the newline). One `<` is
            // a plain input redirect and three is a bash here-string, whose
            // word is a single token already — both are boundaries alone.
            '<' => {
                let mut run = 1;
                while chars.peek() == Some(&'<') {
                    chars.next();
                    run += 1;
                }
                flush(&mut cur, &mut out);
                if run == 2 {
                    let mut opener = String::from("<<");
                    let mut strip = false;
                    if chars.peek() == Some(&'-') {
                        chars.next();
                        opener.push('-');
                        strip = true;
                    }
                    while matches!(chars.peek(), Some(' ') | Some('\t')) {
                        opener.push(chars.next().expect("peeked"));
                    }
                    let mut delim = String::new();
                    match chars.peek().copied() {
                        // `<<'EOF'` / `<<"EOF"`: the quotes belong to the
                        // opener, the delimiter is what they hold.
                        Some(q @ ('\'' | '"')) => {
                            chars.next();
                            opener.push(q);
                            while let Some(b) = chars.next() {
                                opener.push(b);
                                if b == q {
                                    break;
                                }
                                delim.push(b);
                            }
                        }
                        // `<<EOF`: to the first whitespace or metacharacter.
                        _ => {
                            while let Some(&b) = chars.peek() {
                                if b.is_whitespace()
                                    || matches!(b, ';' | '|' | '&' | '(' | ')' | '<' | '>')
                                {
                                    break;
                                }
                                chars.next();
                                opener.push(b);
                                delim.push(b);
                            }
                        }
                    }
                    if !delim.is_empty() {
                        pending.push((opener, delim, strip));
                    }
                }
                boundary(&mut out);
            }
            '>' => {
                flush(&mut cur, &mut out);
                let mut r = String::from(">");
                while chars.peek() == Some(&'>') {
                    chars.next();
                    r.push('>');
                }
                out.push(Some(r));
            }
            c => cur.push(c),
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// The command word, normalized: path and .exe stripped, lowercased —
/// `/usr/bin/tee`, `TEE.EXE`, and `tee` all read as "tee".
fn cmd_word(tok: &str) -> String {
    let t = tok.replace('\\', "/").to_lowercase();
    let base = t.rsplit('/').next().unwrap_or(&t);
    base.trim_end_matches(".exe").to_string()
}

/// A parsed target the accrual should never chase: streams, devices,
/// variables and substitutions the parse cannot resolve — and, since
/// it-ap3x, anything that cannot be a filename at all.
///
/// The parse is a guess made over text the tokenizer may have read wrong
/// (prose in a quoted argument, an unterminated heredoc body), and a guess
/// that lands in the observed set is debris at the judgment seat. Two shapes
/// are refused outright, both impossible for a real write target: a token
/// carrying a control character or a shell metacharacter the tokenizer would
/// have split on had it been reading a command (`<>|;&`) or a quote it would
/// have consumed — the tell that the token is a fragment of something else —
/// and a sigil-only token with no name character in it at all (`@`, `--`,
/// `{}`). Dropping is the cheap direction here: the shell channel is
/// best-effort by construction and the sight boundary states its residue,
/// while a false positive misreads as a real write nobody made.
fn opaque_target(t: &str) -> bool {
    t.is_empty()
        || t == "-"
        || t.contains('$')
        || t.contains('%')
        || t.contains('`')
        || t.contains('*')
        || t.contains('?')
        || t.chars().any(|c| {
            c.is_control() || matches!(c, '<' | '>' | '|' | ';' | '&' | '\'' | '"')
        })
        || !t.chars().any(char::is_alphanumeric)
        || matches!(t.to_lowercase().as_str(), "/dev/null" | "nul" | "null")
}

/// Best-effort write-target extraction from a shell command (it-bj3b #4):
/// the common write shapes — redirects (`>`/`>>`, glued forms included),
/// `tee`, `cp`/`mv` targets, `touch`, `git checkout -- <paths>` and
/// `git restore <paths>`, and the PowerShell content writers (Set-Content,
/// Add-Content, Out-File, Copy-Item/Move-Item destinations). String-matching
/// in the commit-sweep guard's posture: parsing narrows the blind zone, the
/// sight-boundary statement covers what it cannot see (opaque scripts,
/// `git apply`, generated files). Returns raw targets in order, deduped.
pub fn write_shapes(command: &str) -> Vec<String> {
    let tokens = shell_tokens(command);
    let mut out: Vec<String> = Vec::new();
    let push = |t: &str, out: &mut Vec<String>| {
        let t = t.trim_start_matches("./");
        if !opaque_target(t) && !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    };
    let mut i = 0;
    while i < tokens.len() {
        let Some(tok) = tokens[i].as_deref() else {
            i += 1;
            continue;
        };
        // A redirect anywhere: the next token (if any, and not a boundary)
        // is its target.
        if tok == ">" || tok == ">>" {
            if let Some(Some(t)) = tokens.get(i + 1) {
                push(t, &mut out);
            }
            i += 2;
            continue;
        }
        // A command word: scan its own args — stopping at the first
        // redirect (whose target belongs to the redirect, re-scanned below)
        // or boundary.
        let word = cmd_word(tok);
        let args: Vec<&str> = {
            let mut a = Vec::new();
            let mut j = i + 1;
            while let Some(Some(t)) = tokens.get(j) {
                if t == ">" || t == ">>" {
                    break;
                }
                a.push(t.as_str());
                j += 1;
            }
            a
        };
        let non_flag = |xs: &[&str]| -> Vec<String> {
            xs.iter()
                .filter(|x| !x.starts_with('-') && **x != ">" && **x != ">>")
                .map(|x| x.to_string())
                .collect()
        };
        match word.as_str() {
            "tee" => {
                for t in non_flag(&args) {
                    push(&t, &mut out);
                }
            }
            "cp" | "mv" => {
                // `-t <dir>` names the target ahead; otherwise the last
                // non-flag arg is the destination.
                if let Some(p) = args.iter().position(|x| *x == "-t" || *x == "--target-directory")
                {
                    if let Some(t) = args.get(p + 1) {
                        push(t, &mut out);
                    }
                } else {
                    let plain = non_flag(&args);
                    if plain.len() >= 2 {
                        push(plain.last().unwrap(), &mut out);
                    }
                }
            }
            "touch" => {
                for t in non_flag(&args) {
                    push(&t, &mut out);
                }
            }
            "git" => {
                let sub = args.first().map(|s| s.to_lowercase()).unwrap_or_default();
                if sub == "checkout" {
                    if let Some(p) = args.iter().position(|x| *x == "--") {
                        for t in &args[p + 1..] {
                            if *t != ">" && *t != ">>" {
                                push(t, &mut out);
                            }
                        }
                    }
                } else if sub == "restore" {
                    let mut k = 1;
                    while k < args.len() {
                        let a = args[k];
                        if a == "--source" {
                            k += 2;
                            continue;
                        }
                        if !a.starts_with('-') && a != ">" && a != ">>" {
                            push(a, &mut out);
                        }
                        k += 1;
                    }
                }
            }
            "set-content" | "add-content" | "out-file" | "copy-item" | "move-item" => {
                let path_flag = |f: &str| {
                    matches!(f, "-path" | "-filepath" | "-literalpath")
                        || (matches!(word.as_str(), "copy-item" | "move-item")
                            && f == "-destination")
                };
                // Flags whose VALUE is not a path — skip it so it never
                // reads as the positional target.
                let value_flag = |f: &str| {
                    matches!(f, "-value" | "-encoding" | "-inputobject" | "-stream" | "-filter")
                };
                let mut positional: Vec<String> = Vec::new();
                let mut flagged: Option<String> = None;
                let mut k = 0;
                while k < args.len() {
                    let a = args[k];
                    let f = a.to_lowercase();
                    if path_flag(&f) {
                        if let Some(t) = args.get(k + 1) {
                            if flagged.is_none() || f == "-destination" {
                                flagged = Some(t.to_string());
                            }
                        }
                        k += 2;
                        continue;
                    }
                    if value_flag(&f) {
                        k += 2;
                        continue;
                    }
                    if a.starts_with('-') || a == ">" || a == ">>" {
                        k += 1;
                        continue;
                    }
                    positional.push(a.to_string());
                    k += 1;
                }
                match word.as_str() {
                    // Copy-Item/Move-Item: the DESTINATION is the write —
                    // second positional when no flag named it.
                    "copy-item" | "move-item" => {
                        if let Some(t) =
                            flagged.or_else(|| positional.get(1).cloned())
                        {
                            push(&t, &mut out);
                        }
                    }
                    // The content writers: first positional is the path.
                    _ => {
                        if let Some(t) = flagged.or_else(|| positional.first().cloned()) {
                            push(&t, &mut out);
                        }
                    }
                }
            }
            _ => {}
        }
        // Jump to the boundary this command's args ran to; redirects inside
        // the args were NOT consumed here — re-scan them individually.
        let mut j = i + 1;
        while let Some(Some(t)) = tokens.get(j) {
            if t == ">" || t == ">>" {
                if let Some(Some(target)) = tokens.get(j + 1) {
                    push(target, &mut out);
                }
            }
            j += 1;
        }
        i = j;
    }
    out
}

/// The shell half of the observation layer (it-bj3b #4), riding the
/// Bash|PowerShell PreToolUse hook: parse the command for common write
/// shapes and accrue every target that RESOLVES store-relative, marked
/// `via: shell` so harvest renders the channel honestly. Observation only —
/// never a denial, never a context line: parsing is best-effort, and a
/// false-positive parse must cost nothing. Targets that resolve nowhere are
/// dropped here (a parse is a guess; only the tool-write channel records
/// unresolved paths, because there the write is a certainty).
pub fn observe_shell(
    store: &crate::store::Store,
    badge: Option<&str>,
    session: Option<&str>,
    command: &str,
    cwd: Option<&Path>,
) {
    use crate::coord;
    let targets = write_shapes(command);
    if targets.is_empty() {
        return;
    }
    let key = coord::touch_key(badge, session);
    let mut prior: Vec<String> =
        coord::touches_for(store, &key).into_iter().map(|t| t.path).collect();
    let base = cwd.map(Path::to_path_buf).unwrap_or_else(|| store.work_root.clone());
    for t in targets {
        let abs_shaped =
            t.starts_with('/') || t.starts_with('\\') || t.get(1..2) == Some(":");
        let abs = if abs_shaped {
            t.clone()
        } else {
            base.join(&t).to_string_lossy().to_string()
        };
        let Some(rel) = store.relative(&abs) else { continue };
        if rel.starts_with("graph/") || rel.split('/').any(|c| c == "." || c == "..") {
            continue;
        }
        if prior.iter().any(|p| *p == rel) {
            continue;
        }
        coord::accrue_touch_ext(store, &key, &rel, Some("shell"), false);
        prior.push(rel);
    }
}

/// C6, as a PreToolUse hook. Returns Some(denial) if the tool call should be
/// blocked, None to allow. Input is the hook's stdin JSON.
pub fn guard(input: &str) -> Option<String> {
    let path = write_target(input)?;
    let p = path.replace('\\', "/").to_lowercase();
    if p.contains("graph/nodes/") || p.contains("graph/log/") {
        Some(
            "C6: files under graph/ are the work graph and are written only through the q verbs — \
             a verb bumps the node's version, stamps every ref, and enforces the constraints; a \
             hand edit silently bypasses all three. Use q new / q set / q edit / q link / q rule / \
             q claim / q refute / q affirm instead (reading is unrestricted; `q open <node>` \
             renders the brief). Run `q guide` for the judgment layer."
                .into(),
        )
    } else {
        None
    }
}

/// What a sweep-shaped staging command would capture: repo-relative prefix
/// coverage ("" = the whole tree), each cover flagged tracked-only when the
/// shape stages tracked files only (`git commit -a` cannot capture an
/// untracked node file).
pub struct SweepShape {
    /// The offending token, for the refusal to name.
    pub shape: String,
    pub covers: Vec<(String, bool)>,
}

impl SweepShape {
    pub fn captures(&self, path: &str, tracked: bool) -> bool {
        let p = path.to_lowercase();
        self.covers.iter().any(|(prefix, tracked_only)| {
            (!tracked_only || tracked)
                && (prefix.is_empty() || p == *prefix || p.starts_with(&format!("{}/", prefix)))
        })
    }
}

/// Bulk-staging detection for the commit-sweep guard (it-4q6t): string-
/// matching the shell command, best-effort by design — wrap backstops, and
/// evasion is self-inflicted (the write-guard posture). `cwd_rel` is the
/// command's cwd relative to the repo root ("" = the root); it scopes `.`
/// and bare relative dirs. Explicit file paths never match: adoption and
/// the taught own-line pass by construction.
pub fn sweep_shape(command: &str, cwd_rel: &str) -> Option<SweepShape> {
    let cwd_rel = cwd_rel.trim_matches('/').to_lowercase();
    let scoped = |p: &str| -> String {
        if cwd_rel.is_empty() {
            p.to_string()
        } else if p.is_empty() {
            cwd_rel.clone()
        } else {
            format!("{}/{}", cwd_rel, p)
        }
    };
    // A graph DIRECTORY is bulk; a named file is explicit. The six type
    // dirs live under graph/nodes; a glob into graph is bulk too.
    let bulk_graph_dir = |p: &str| -> bool {
        p == "graph"
            || p == "graph/nodes"
            || p == "graph/log"
            || (p.starts_with("graph/nodes/") && !p[12..].contains('.') && !p[12..].contains('/'))
            || (p.starts_with("graph/") && p.contains('*'))
    };
    // Tokenize: whitespace splits words; shell separators (even glued to a
    // token, `graph;git`) reset the command boundary.
    let seps: &[char] = &['\n', ';', '|', '&', '(', ')'];
    let cleaned: String =
        command.chars().map(|c| if seps.contains(&c) { '\u{1}' } else { c }).collect();
    let mut tokens: Vec<Option<String>> = Vec::new(); // None = a separator
    for raw in cleaned.split_whitespace() {
        for (i, piece) in raw.split('\u{1}').enumerate() {
            if i > 0 {
                tokens.push(None);
            }
            let t = piece.trim_matches(|c| c == '"' || c == '\'');
            if !t.is_empty() {
                tokens.push(Some(t.to_string()));
            }
        }
    }
    #[derive(PartialEq)]
    enum St {
        Idle,
        Git,
        Add,
        Commit,
    }
    let mut st = St::Idle;
    let mut covers: Vec<(String, bool)> = Vec::new();
    let mut shape: Option<String> = None;
    for tok in &tokens {
        let Some(tok) = tok.as_deref() else {
            st = St::Idle;
            continue;
        };
        match st {
            St::Idle => {
                if tok == "git" || tok.to_lowercase().ends_with("git.exe") {
                    st = St::Git;
                }
            }
            St::Git => {
                if tok.starts_with('-') {
                    // global flags (-C/-c consume a value we cannot see
                    // apart — best-effort: rare in agent hands)
                } else if tok == "add" || tok == "stage" {
                    st = St::Add;
                } else if tok == "commit" {
                    st = St::Commit;
                } else {
                    st = St::Idle;
                }
            }
            St::Add => match tok {
                "--" => {}
                "-A" | "--all" | "-u" | "--update" | "--no-ignore-removal" | ":/" | ":/."
                | ":(top)" => {
                    shape.get_or_insert_with(|| format!("git add {}", tok));
                    covers.push((String::new(), false));
                }
                t if t.starts_with('-') => {}
                "." | "./" | "*" => {
                    shape.get_or_insert_with(|| format!("git add {}", tok));
                    covers.push((scoped(""), false));
                }
                t => {
                    let p = t
                        .replace('\\', "/")
                        .trim_start_matches("./")
                        .trim_end_matches('/')
                        .to_lowercase();
                    let p = if p.starts_with("graph") { p } else { scoped(&p) };
                    if bulk_graph_dir(&p) {
                        shape.get_or_insert_with(|| format!("git add {}", t));
                        covers.push((p, false));
                    }
                }
            },
            St::Commit => {
                if tok == "--all"
                    || (tok.starts_with('-')
                        && !tok.starts_with("--")
                        && tok[1..].chars().all(|c| c.is_ascii_alphabetic())
                        && tok.contains('a'))
                {
                    shape.get_or_insert_with(|| format!("git commit {}", tok));
                    covers.push((String::new(), true));
                }
            }
        }
    }
    if covers.is_empty() {
        None
    } else {
        Some(SweepShape { shape: shape.unwrap_or_default(), covers })
    }
}

/// The commit-sweep guard (it-4q6t), riding the Bash|PowerShell PreToolUse
/// hook on the C6 channel: a sweep-shaped staging command that would capture
/// another session's uncommitted node files refuses, handing the asking
/// session its own git add line. Nothing foreign pending: silence — the
/// sweep is harmless. The guard blocks the bulk SHAPE, never deliberate
/// adoption: foreign files list with the owner's last-seen age, and a stale
/// owner's line flips to an explicit-path adoption offer that passes by
/// construction. Fires only where the command works the store's own
/// checkout — a worktree fork stages its own inert graph copy, not this one.
pub fn staging_guard(
    store: &crate::store::Store,
    tool: &str,
    command: &str,
    cwd: Option<&Path>,
    session: Option<&str>,
) -> Option<String> {
    if !matches!(tool, "Bash" | "PowerShell") {
        return None;
    }
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/").to_lowercase();
    let root = norm(&store.root);
    if norm(&store.work_root) != root {
        return None; // a fork's git acts on the fork's tree, not this graph
    }
    let cwd_rel = match cwd {
        Some(c) => {
            let c = norm(c);
            if c == root {
                String::new()
            } else if let Some(rel) = c.strip_prefix(&format!("{}/", root)) {
                rel.to_string()
            } else {
                return None; // the command works some other tree
            }
        }
        None => String::new(),
    };
    let sweep = sweep_shape(command, &cwd_rel)?;
    let pending = crate::queries::pending_graph(store)?;
    let captured_foreign: Vec<(&crate::queries::CommitSet, Vec<&crate::queries::CommitFile>)> =
        pending
            .foreign(session)
            .into_iter()
            .filter_map(|set| {
                let files: Vec<&crate::queries::CommitFile> = set
                    .files
                    .iter()
                    .filter(|f| sweep.captures(&f.path, f.tracked))
                    .collect();
                if files.is_empty() {
                    None
                } else {
                    Some((set, files))
                }
            })
            .collect();
    if captured_foreign.is_empty() {
        return None;
    }
    let mut msg = format!(
        "C6: this staging sweep ({}) would capture other sessions' uncommitted graph nodes — the graph log attributes every write, but a sweep buries their work under your commit message. Stage your own commit-set instead:",
        sweep.shape
    );
    match pending.own(session) {
        Some(own) => {
            msg.push_str(&format!("\n  {}", own.add_command()));
            for f in own.files.iter().filter(|f| !f.carries.is_empty()) {
                msg.push_str(&format!(
                    "\n    ({} carries {}'s earlier edits — worth naming in your commit message)",
                    f.node,
                    f.carries.join(", ")
                ));
            }
        }
        None => {
            msg.push_str(
                "\n  (nothing of yours is pending under graph/ — this sweep would only capture others' work)",
            );
        }
    }
    if !pending.ledger.is_empty() {
        msg.push_str(&format!(
            "\n  the shared log shard rides along with any commit (exempt ledger, internally attributed, never split): git add {}",
            pending.ledger.join(" ")
        ));
    }
    msg.push_str("\ntheirs — leave for the owner, or adopt by explicit paths:");
    for (set, files) in &captured_foreign {
        let owner = set.owner.as_deref().unwrap_or("?");
        if crate::queries::owner_stale(set.age_secs) {
            msg.push_str(&format!(
                "\n  · {} ({} — stale; adopt if the work should land):",
                owner,
                crate::queries::age_phrase(set.age_secs)
            ));
            for l in set.adoption_offer() {
                msg.push_str(&format!("\n      {}", l));
            }
        } else {
            msg.push_str(&format!(
                "\n  · {} ({}):",
                owner,
                crate::queries::age_phrase(set.age_secs)
            ));
            for f in files {
                let carries = if f.carries.is_empty() {
                    String::new()
                } else {
                    format!(" (carries {}'s earlier edits)", f.carries.join(", "))
                };
                msg.push_str(&format!("\n      {}{}", f.path, carries));
            }
        }
    }
    msg.push_str(
        "\nexplicit-path staging always passes this guard; the full map: q query commit-set",
    );
    Some(msg)
}

/// Write the generated skill and wire the guard hook into the host repo's
/// .claude/settings.json. Returns a human summary of what happened.
pub fn install_claude(root: &Path) -> Result<Vec<String>> {
    let mut actions = Vec::new();

    let skill_dir = root.join(".claude").join("skills").join("quarry");
    fs::create_dir_all(&skill_dir)?;
    let skill = format!(
        "{}{}\n\n---\nGenerated by `q init --claude` (quarry v{}). Do not hand-edit — re-run \
         after upgrading the tool. Mechanics live in `q --help`.\n",
        SKILL_FRONT,
        GUIDE.trim_end(),
        env!("CARGO_PKG_VERSION")
    );
    fs::write(skill_dir.join("SKILL.md"), skill)?;
    actions.push(".claude/skills/quarry/SKILL.md written".into());

    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("q"));
    let cmd = format!("\"{}\" hook guard", exe.to_string_lossy().replace('\\', "/"));
    let settings_path = root.join(".claude").join("settings.json");
    let mut settings: serde_json::Value = if settings_path.exists() {
        serde_json::from_str(&fs::read_to_string(&settings_path)?)
            .map_err(|e| anyhow!(".claude/settings.json is not valid JSON: {}", e))?
    } else {
        json!({})
    };
    let exe_quoted = cmd.trim_end_matches(" hook guard").to_string();
    let obj = settings
        .as_object_mut()
        .ok_or_else(|| anyhow!(".claude/settings.json is not a JSON object"))?;
    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow!("settings 'hooks' is not an object"))?;
    let mut changed = false;

    let pre = hooks
        .entry("PreToolUse")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow!("settings 'hooks.PreToolUse' is not an array"))?;
    if pre
        .iter()
        .any(|e| serde_json::to_string(e).unwrap_or_default().contains("hook guard"))
    {
        actions.push("guard hook already present in .claude/settings.json".into());
    } else {
        pre.push(json!({
            "matcher": "Write|Edit|NotebookEdit",
            "hooks": [{
                "type": "command",
                "command": cmd,
                "timeout": 10,
                "statusMessage": "quarry: guarding graph/"
            }]
        }));
        actions.push(format!(".claude/settings.json: PreToolUse guard added ({})", cmd));
        changed = true;
    }

    let session_cmd = format!("{} hook session", exe_quoted);
    if pre
        .iter()
        .any(|e| serde_json::to_string(e).unwrap_or_default().contains("hook session"))
    {
        actions.push("session hook already present in .claude/settings.json".into());
    } else {
        pre.push(json!({
            "matcher": "Bash|PowerShell",
            "hooks": [{
                "type": "command",
                "command": session_cmd,
                "timeout": 10,
                "statusMessage": "quarry: session identity"
            }]
        }));
        actions.push(format!(".claude/settings.json: PreToolUse session injection added ({})", session_cmd));
        changed = true;
    }

    let orient_cmd = format!("{} hook orient", exe_quoted);
    let ss = hooks
        .entry("SessionStart")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow!("settings 'hooks.SessionStart' is not an array"))?;
    if ss
        .iter()
        .any(|e| serde_json::to_string(e).unwrap_or_default().contains("hook orient"))
    {
        actions.push("orient hook already present in .claude/settings.json".into());
    } else {
        ss.push(json!({
            "hooks": [{
                "type": "command",
                "command": orient_cmd,
                "timeout": 10,
                "statusMessage": "quarry: orienting"
            }]
        }));
        actions.push(format!(".claude/settings.json: SessionStart orient added ({})", orient_cmd));
        changed = true;
    }

    if changed {
        fs::write(&settings_path, serde_json::to_string_pretty(&settings)? + "\n")?;
    }
    Ok(actions)
}
