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
rulings, uncommitted graph changes. In conversation, refer to nodes by
TITLE — ids belong in commands, not in prose to the user.

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

REFS AND STALENESS
Cite nodes and files with edges; stamps are automatic. behind is
information, not noise: severity 1 means something you cite was refuted or
superseded — read it before building further. When a claim falls, blast
enumerates everything leaning on it. That list IS the correction; there is
no sweep. Mutating verbs print the HOMEWORK an action creates — citers put
behind, work unblocked, threads made answerable — with the command that
addresses each. Do the homework (or queue it) before moving on.

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
claim/decision) · supersedes (same type) · source (claim → doc)

ENVIRONMENT
QUARRY_ACTOR (your model identity) and QUARRY_SESSION are auto-injected by
the session hook for chats running in this repo — you should never set
them by hand. Set QUARRY_ACTOR manually only when operating outside hook
coverage (e.g. from a parent directory). If unset entirely, provenance
safely derives as assistant; user provenance is always explicit.
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
    let inject_actor = if env_actor.is_none() {
        Some(crate::coord::safe_actor(
            &chat_id
                .and_then(|cid| crate::coord::chat_actor(store, cid))
                .unwrap_or_else(|| "claude".into()),
        ))
    } else {
        None
    };
    let inject_sess = if env_sess.is_none() { bound.clone() } else { None };
    let mut updated_input: Option<serde_json::Map<String, serde_json::Value>> = None;
    if matches!(tool, "Bash" | "PowerShell")
        && (inject_sess.is_some() || (inject_actor.is_some() && chat_id.is_some()))
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
                let mut u = ti.clone();
                u.insert("command".into(), serde_json::json!(format!("{}{}", prefix, cmd)));
                updated_input = Some(u);
            }
        }
    }
    let alert = session.as_deref().and_then(|s| alerts(store, s));
    if updated_input.is_none() && alert.is_none() {
        return None;
    }
    let mut hso = serde_json::Map::new();
    hso.insert("hookEventName".into(), serde_json::json!("PreToolUse"));
    if let Some(u) = updated_input {
        hso.insert("updatedInput".into(), serde_json::Value::Object(u));
    }
    if let Some(a) = alert {
        hso.insert("additionalContext".into(), serde_json::json!(a));
    }
    Some(serde_json::json!({ "hookSpecificOutput": serde_json::Value::Object(hso) }))
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AlertCursor {
    cursor: String,
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
        map.insert(session.into(), AlertCursor { cursor: now.clone(), checked: now });
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
    let lines = if let Some(p) = reg.get(session) {
        let ids: Vec<&str> = p.areas.iter().map(|s| s.as_str()).collect();
        let log = store.read_log().ok()?;
        alerts_between(&all, &log, session, &ids, &cur.cursor)
    } else {
        vec![]
    };
    map.insert(session.into(), AlertCursor { cursor: now.clone(), checked: now });
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
    cursor: &str,
) -> Vec<String> {
    use crate::model::Node as N;
    let title_of = |id: &str| {
        all.iter()
            .find(|n| n.front.id == id)
            .map(|n| n.front.title.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let mut out: Vec<String> = Vec::new();
    for ev in log {
        let ts = ev.get("ts").and_then(|x| x.as_str()).unwrap_or("");
        if ts <= cursor {
            continue;
        }
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
                                "new from {}: \"{}\" [{}] — q open {}",
                                ev_sess.unwrap_or("?"),
                                n.front.title,
                                n.front.status,
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
                        "your lease on \"{}\" was taken by {}: {}",
                        title_of(victim),
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
                            "unblocked: \"{}\" — {} landed \"{}\"",
                            d.front.title,
                            ev_sess.unwrap_or("?"),
                            title_of(node_id)
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

/// Check a repo-relative write path against the live leases. A foreign
/// EXCLUSIVE lease covering the path denies for everyone (that is what the
/// lease means). Under QUARRY_DISPATCH, a write outside the own session's
/// lease denies — the brief's write-set is the contract. A main session
/// holding leases but writing outside all of them gets a warning: scope
/// creep made visible, not forbidden.
pub fn lease_check(
    leases: &[crate::coord::Lease],
    session: Option<&str>,
    dispatch_item: Option<&str>,
    rel_path: &str,
) -> LeaseCheck {
    if leases.is_empty() || rel_path.starts_with("graph/") {
        return LeaseCheck::Allow;
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
    let covered_own = leases.iter().any(|l| {
        session.map_or(false, |s| l.session == s)
            && l.globs.iter().any(|g| crate::coord::globs_overlap(g, rel_path))
    });
    if let Some(item) = dispatch_item {
        if !covered_own {
            return LeaseCheck::Deny(format!(
                "dispatch write outside the leased write-set: {} is not covered by the lease for {} — the brief's write-set is the contract; ask the dispatcher to extend the lease.",
                rel_path, item
            ));
        }
        return LeaseCheck::Allow;
    }
    let holds_any = session.map_or(false, |s| leases.iter().any(|l| l.session == s));
    if holds_any && !covered_own {
        return LeaseCheck::Warn(format!(
            "quarry: this write ({}) lands outside every lease your session holds — scope creep, or a lease wanting extension?",
            rel_path
        ));
    }
    LeaseCheck::Allow
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
