//! Multi-session coordination: the committed session registry (purviews) and
//! the machine-local lease file (reservations). A session's durable identity
//! is its purview over areas; a lease is per-item operational plumbing,
//! acquired at dispatch and released explicitly. Leases never enter the
//! knowledge log except as the meaningful acts reserve / release / steal.

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;

use crate::model::Node;
use crate::store::Store;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Purview {
    pub areas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub charter: Option<String>,
    /// Sessions persist by default — defining one makes it re-enterable
    /// (launcher or adopt). Ephemeral is the marked odd case: this chat only.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub ephemeral: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Lease {
    pub item: String,
    pub item_title: String,
    pub session: String,
    pub actor: String,
    pub globs: Vec<String>,
    pub shared: bool,
    pub since: String,
}

pub fn current_session() -> Option<String> {
    std::env::var("QUARRY_SESSION").ok().filter(|s| !s.trim().is_empty())
}

fn sessions_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join("sessions.json")
}

fn leases_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".reservations.json")
}

pub fn load_sessions(store: &Store) -> BTreeMap<String, Purview> {
    fs::read_to_string(sessions_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_session(
    store: &Store,
    name: &str,
    areas: Vec<String>,
    charter: Option<String>,
    ephemeral: bool,
) -> Result<()> {
    let mut reg = load_sessions(store);
    reg.insert(name.to_string(), Purview { areas, charter, ephemeral });
    fs::write(sessions_path(store), serde_json::to_string_pretty(&reg)? + "\n")?;
    Ok(())
}

/// Retire a session: registry entry removed, its leases released, its
/// heartbeat cleared — the logged event is its last rites. The default close
/// act for an ephemeral session at wrap; available to any session by
/// deliberate choice.
pub fn retire_session(store: &Store, name: &str, actor: &str) -> Result<()> {
    let mut reg = load_sessions(store);
    if reg.remove(name).is_none() {
        bail!("session '{}' is not registered", name);
    }
    fs::write(sessions_path(store), serde_json::to_string_pretty(&reg)? + "\n")?;
    let mut leases = load_leases(store);
    let before = leases.len();
    leases.retain(|l| l.session != name);
    if leases.len() != before {
        save_leases(store, &leases)?;
    }
    let mut live: BTreeMap<String, String> = fs::read_to_string(live_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    if live.remove(name).is_some() {
        let _ = fs::write(
            live_path(store),
            serde_json::to_string_pretty(&live).unwrap_or_default() + "\n",
        );
    }
    store.log_event(json!({
        "ts": Store::now(), "node": format!("session:{}", name), "v": 0,
        "op": "retire-session", "actor": actor, "released_leases": before - leases.len()
    }))?;
    Ok(())
}

/// Where a proposed purview intersects existing sessions' purviews.
/// Overlap is legal (shared areas exist) — but it must be seen, not slipped.
pub fn purview_overlaps(store: &Store, areas: &[String]) -> Vec<(String, Vec<String>)> {
    load_sessions(store)
        .into_iter()
        .filter_map(|(name, p)| {
            let shared: Vec<String> = p
                .areas
                .iter()
                .filter(|a| areas.contains(a))
                .cloned()
                .collect();
            if shared.is_empty() {
                None
            } else {
                Some((name, shared))
            }
        })
        .collect()
}

pub fn load_leases(store: &Store) -> Vec<Lease> {
    fs::read_to_string(leases_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_leases(store: &Store, leases: &[Lease]) -> Result<()> {
    fs::write(leases_path(store), serde_json::to_string_pretty(leases)? + "\n")?;
    Ok(())
}

/// The static prefix of a glob: everything before the first wildcard char.
fn static_prefix(glob: &str) -> &str {
    let idx = glob.find(['*', '?', '[']).unwrap_or(glob.len());
    &glob[..idx]
}

/// Conservative pattern-intersection test: two globs are taken to overlap
/// when either one's static prefix is a prefix of the other's. Exact for the
/// common `dir/**` shapes; over-approximates for mid-pattern wildcards,
/// which errs toward denial (steal or --shared are the overrides).
pub fn globs_overlap(a: &str, b: &str) -> bool {
    let (pa, pb) = (
        static_prefix(a).replace('\\', "/"),
        static_prefix(b).replace('\\', "/"),
    );
    pa.starts_with(&pb) || pb.starts_with(&pa)
}

fn lease_overlaps(lease: &Lease, globs: &[String]) -> bool {
    lease
        .globs
        .iter()
        .any(|lg| globs.iter().any(|g| globs_overlap(lg, g)))
}

#[derive(Debug)]
pub struct ReserveOutcome {
    pub co_holders: Vec<Lease>,
    pub stolen: Vec<Lease>,
}

#[allow(clippy::too_many_arguments)]
pub fn reserve(
    store: &Store,
    item: &Node,
    session: &str,
    actor: &str,
    globs: Vec<String>,
    shared: bool,
    steal: bool,
    reason: Option<&str>,
) -> Result<ReserveOutcome> {
    if globs.is_empty() {
        bail!("a lease needs at least one --files glob (use ** to cover files the work will create)");
    }
    let mut leases = load_leases(store);
    if leases.iter().any(|l| l.item == item.front.id) {
        bail!(
            "\"{}\" already holds a lease — release it first (q release {}) or reserve a different item",
            item.front.title,
            item.front.id
        );
    }
    let foreign: Vec<Lease> = leases
        .iter()
        .filter(|l| l.session != session && lease_overlaps(l, &globs))
        .cloned()
        .collect();
    let conflicts: Vec<&Lease> = foreign.iter().filter(|l| !(shared && l.shared)).collect();
    let mut stolen = Vec::new();
    if !conflicts.is_empty() {
        if !steal {
            let who = conflicts
                .iter()
                .map(|l| {
                    format!(
                        "session {} holds {:?} for \"{}\" (since {})",
                        l.session, l.globs, l.item_title, l.since
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            bail!(
                "C7: write-set overlaps a live lease — {}. Coordinate with the holder, use --shared if this is a co-write zone, or --steal (loud, logged).",
                who
            );
        }
        let victim_items: Vec<String> = conflicts.iter().map(|l| l.item.clone()).collect();
        for l in &foreign {
            if victim_items.contains(&l.item) {
                stolen.push(l.clone());
            }
        }
        leases.retain(|l| !victim_items.contains(&l.item));
        for v in &stolen {
            store.log_event(json!({
                "ts": Store::now(), "node": item.front.id, "v": item.front.v,
                "op": "steal", "from_session": v.session, "from_item": v.item,
                "globs": v.globs, "actor": actor, "session": session,
                "reason": reason
            }))?;
        }
    }
    let co_holders: Vec<Lease> = foreign
        .into_iter()
        .filter(|l| shared && l.shared && !stolen.iter().any(|s| s.item == l.item))
        .collect();
    leases.push(Lease {
        item: item.front.id.clone(),
        item_title: item.front.title.clone(),
        session: session.to_string(),
        actor: actor.to_string(),
        globs: globs.clone(),
        shared,
        since: Store::now(),
    });
    save_leases(store, &leases)?;
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "reserve", "globs": globs, "shared": shared,
        "actor": actor, "session": session
    }))?;
    Ok(ReserveOutcome { co_holders, stolen })
}

pub fn release(store: &Store, item: &Node, session: &str, actor: &str) -> Result<()> {
    let mut leases = load_leases(store);
    let Some(pos) = leases.iter().position(|l| l.item == item.front.id) else {
        bail!("\"{}\" holds no lease", item.front.title);
    };
    if leases[pos].session != session {
        bail!(
            "the lease on \"{}\" is held by session {} — theirs to release (or reserve with --steal)",
            item.front.title,
            leases[pos].session
        );
    }
    leases.remove(pos);
    save_leases(store, &leases)?;
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "release", "actor": actor, "session": session
    }))?;
    Ok(())
}

// ── the dispatch badge (machine-local) ─────────────────────────────────────
//
// QUARRY_DISPATCH in a shell's env stamps that shell's q acts, but a hook
// process spawned by the harness never sees the agent's shell env — so the
// badge also lives machine-locally, written at `q dispatch` and cleared at
// harvest/release. Known blur (accepted, single-orchestrator): while a
// dispatch is active, the orchestrator's own code writes on this machine are
// indistinguishable from the dispatched agent's.

/// The active dispatch, with the contract captured at dispatch time so the
/// write guard can echo it without loading the graph.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DispatchState {
    pub item: String,
    pub item_title: String,
    pub session: String,
    pub globs: Vec<String>,
    #[serde(default)]
    pub acceptance: Vec<String>,
    pub since: String,
    /// Log position at dispatch — the mid-flight drift check reads forward
    /// from here, never the whole log.
    #[serde(default)]
    pub cursor: u64,
    /// Wall-clock throttle stamp for the drift check.
    #[serde(default)]
    pub checked: String,
}

fn dispatch_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".dispatch.json")
}

pub fn save_dispatch(store: &Store, d: &DispatchState) -> Result<()> {
    fs::write(dispatch_path(store), serde_json::to_string_pretty(d)? + "\n")?;
    Ok(())
}

pub fn load_dispatch(store: &Store) -> Option<DispatchState> {
    serde_json::from_str(&fs::read_to_string(dispatch_path(store)).ok()?).ok()
}

/// Clear the badge if it names this item (harvest and land both clear).
pub fn clear_dispatch(store: &Store, item_id: &str) {
    if load_dispatch(store).map_or(false, |d| d.item == item_id) {
        let _ = fs::remove_file(dispatch_path(store));
    }
}

/// The active badge: QUARRY_DISPATCH env wins (explicit, per-shell); the
/// machine-local state is the fallback for processes the env cannot reach.
pub fn current_dispatch_badge(store: &Store) -> Option<String> {
    std::env::var("QUARRY_DISPATCH")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| load_dispatch(store).map(|d| d.item))
}

/// C8's logic applied to boundary acts (it-ymsj): wrap and session
/// resume/retire are the DISPATCHER'S verbs. Under an active badge — the
/// shell env or the machine-local dispatch state, either alone suffices —
/// they refuse with a teaching error. The incident this guard exists for:
/// a dispatched agent ran q wrap wearing the dispatcher's injected session
/// identity and consumed its session cursors.
pub fn boundary_refusal(store: &Store, verb: &str) -> Option<String> {
    let badge = current_dispatch_badge(store)?;
    Some(format!(
        "boundary-verb capture: {verb} is a session-boundary act, and an active dispatch badge ({badge}) marks this machine mid-dispatch. A badged boundary verb runs wearing the dispatching session's identity and consumes its cursors — the incident class this guard exists for. A dispatched agent reports against the RETURN spec and stops; the boundary belongs to the dispatcher, who closes the arc first: q harvest {badge}"
    ))
}

// ── the touched-set accrual (machine-local) ────────────────────────────────
//
// Leaseless code writes are observed, never denied (the lease is an arc
// declaration; a blocked write breeds junk leases). The write guard accrues
// every allowed code write here: per-item under a badge, per-session
// leaseless. Append-only JSONL — O(1) on the write path, no graph load.

/// Distinct source files a leaseless session touches before the once-per-
/// session nudge fires. One or two is a casual edit; three is an arc forming.
pub const LEASELESS_NUDGE_THRESHOLD: usize = 3;

fn touched_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".touched.jsonl")
}

/// The accrual key: `item:<id>` under a badge, `session:<name>` leaseless.
pub fn touch_key(badge: Option<&str>, session: Option<&str>) -> String {
    match badge {
        Some(b) => format!("item:{}", b),
        None => format!("session:{}", session.unwrap_or("unbound")),
    }
}

/// Append one touched path (call only for paths not already accrued —
/// `touched_for` gives the prior set). Best-effort: observation never fails
/// a write.
pub fn accrue_touch(store: &Store, key: &str, rel_path: &str) {
    use std::io::Write as _;
    let line = serde_json::json!({"ts": Store::now(), "key": key, "path": rel_path});
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(touched_path(store))
    {
        let _ = writeln!(f, "{}", line);
    }
}

/// Distinct touched paths for one key, in first-touch order.
pub fn touched_for(store: &Store, key: &str) -> Vec<String> {
    let Ok(s) = fs::read_to_string(touched_path(store)) else {
        return vec![];
    };
    let mut out: Vec<String> = Vec::new();
    for line in s.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        if v.get("key").and_then(|x| x.as_str()) == Some(key) {
            if let Some(p) = v.get("path").and_then(|x| x.as_str()) {
                if !out.iter().any(|x| x == p) {
                    out.push(p.to_string());
                }
            }
        }
    }
    out
}

/// Drop a key's entries once delivered (wrap pickup) or landed (release).
pub fn clear_touched(store: &Store, key: &str) {
    let Ok(s) = fs::read_to_string(touched_path(store)) else { return };
    let kept: Vec<&str> = s
        .lines()
        .filter(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|v| v.get("key").and_then(|x| x.as_str()).map(String::from))
                .map_or(false, |k| k != key)
        })
        .collect();
    let body = if kept.is_empty() { String::new() } else { kept.join("\n") + "\n" };
    let _ = fs::write(touched_path(store), body);
}

fn area_reads_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".area-reads.json")
}

/// The session key for machine-local attention state: the bound session, or
/// "unbound" for a chat with no identity (imprecise across parallel unbound
/// chats — an accepted, machine-local blur).
pub fn session_key() -> String {
    current_session().unwrap_or_else(|| "unbound".into())
}

/// A machine-local cursor over the event log. Log-INDEX based (user-ruled
/// 2026-08-09): the log is append-only, so "events after position N" is
/// exact — second-granularity timestamps could permanently hide a foreign
/// event landing the same second as the cursor. Legacy timestamp cursors
/// deserialize as Ts and convert on first read.
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Cursor {
    Index(u64),
    Ts(String),
}

/// Resolve a cursor to a log position; a legacy timestamp counts the events
/// at-or-before its stamp (matching the old `>` scan, so nothing re-delivers).
pub fn cursor_index(c: &Cursor, log: &[serde_json::Value]) -> usize {
    match c {
        Cursor::Index(i) => *i as usize,
        Cursor::Ts(t) => log
            .iter()
            .filter(|ev| {
                ev.get("ts")
                    .and_then(|v| v.as_str())
                    .map_or(false, |ts| ts <= t.as_str())
            })
            .count(),
    }
}

type AreaReads = BTreeMap<String, BTreeMap<String, Cursor>>;

fn load_area_reads(store: &Store) -> AreaReads {
    fs::read_to_string(area_reads_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Has this session read (or been delivered) this area at all?
pub fn has_area_read(store: &Store, sess: &str, area_id: &str) -> bool {
    load_area_reads(store)
        .get(sess)
        .map_or(false, |m| m.contains_key(area_id))
}

fn record_area_read_at(store: &Store, sess: &str, area_id: &str, index: usize) {
    let mut m = load_area_reads(store);
    m.entry(sess.to_string())
        .or_default()
        .insert(area_id.to_string(), Cursor::Index(index as u64));
    if let Ok(s) = serde_json::to_string_pretty(&m) {
        let _ = fs::write(area_reads_path(store), s + "\n");
    }
}

/// Record that this session has read (or been delivered) this area's
/// neighborhood — machine-local; the committed log stays mutations-only.
pub fn record_area_read(store: &Store, sess: &str, area_id: &str) {
    let idx = store.read_log().map(|l| l.len()).unwrap_or(0);
    record_area_read_at(store, sess, area_id, idx);
}

/// The per-(session, area) watermark surface (user-ruled 2026-08-09).
pub enum AreaTouch {
    /// No recorded read this session — the caller gates (q new) or nudges
    /// (other verbs), then records the delivery.
    FirstTouch,
    /// Foreign content events landed in this area since the recorded read —
    /// delivered once; the cursor has advanced.
    Drift(Vec<String>),
    /// Nothing foreign since the cursor; cursor advanced silently.
    Current,
}

/// Check (and advance) a session's watermark over one area. Own-session
/// events advance silently; foreign create/set/link/body events on nodes in
/// the area come back as delta lines, capped, each said once. Exact by
/// construction: the cursor is a log position, not a timestamp.
pub fn touch_area(store: &Store, all: &[Node], sess: &str, area_id: &str) -> AreaTouch {
    let Some(cur) = load_area_reads(store)
        .get(sess)
        .and_then(|m| m.get(area_id))
        .cloned()
    else {
        return AreaTouch::FirstTouch;
    };
    let Ok(log) = store.read_log() else {
        return AreaTouch::Current;
    };
    let from = cursor_index(&cur, &log);
    let mut lines: Vec<(String, String)> = Vec::new(); // node id → line
    for ev in log.iter().skip(from) {
        let ev_key = ev
            .get("session")
            .and_then(|v| v.as_str())
            .unwrap_or("unbound");
        if ev_key == sess {
            continue;
        }
        let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("");
        if !matches!(op, "create" | "set" | "link" | "body") {
            continue;
        }
        let Some(id) = ev.get("node").and_then(|v| v.as_str()) else { continue };
        let in_area = id == area_id
            || all
                .iter()
                .find(|n| n.front.id == id)
                .map_or(false, |n| in_purview(n, &[area_id]));
        if !in_area {
            continue;
        }
        let title = all
            .iter()
            .find(|n| n.front.id == id)
            .map(|n| n.front.title.clone())
            .unwrap_or_else(|| id.to_string());
        let line = format!("[{}] \"{}\" ({}) by session {}", op, title, id, ev_key);
        if let Some(pos) = lines.iter().position(|(i, _)| i == id) {
            lines[pos].1 = line;
        } else {
            lines.push((id.to_string(), line));
        }
    }
    record_area_read_at(store, sess, area_id, log.len());
    if lines.is_empty() {
        AreaTouch::Current
    } else {
        let mut out: Vec<String> = lines.into_iter().map(|(_, l)| l).collect();
        if out.len() > 6 {
            let extra = out.len() - 6;
            out.truncate(6);
            out.push(format!("…and {} more — q open {}", extra, area_id));
        }
        AreaTouch::Drift(out)
    }
}

fn topic_queue_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".topic-queue.json")
}

/// The single-thread topic queue (DESIGN.md § 9): an ordered list of thread
/// ids the user works one at a time. Machine-local working state — the
/// threads themselves, and what the user owes, live in the graph.
pub fn load_topic_queue(store: &Store) -> Vec<String> {
    fs::read_to_string(topic_queue_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_topic_queue(store: &Store, q: &[String]) -> Result<()> {
    fs::write(topic_queue_path(store), serde_json::to_string_pretty(q)? + "\n")?;
    Ok(())
}

/// Load the topic queue with resolved/vanished threads pruned out.
/// Returns (live queue, pruned ids); saves only if something was pruned.
pub fn topic_queue_pruned(store: &Store, all: &[Node]) -> (Vec<String>, Vec<String>) {
    let q = load_topic_queue(store);
    let (live, pruned): (Vec<String>, Vec<String>) = q.into_iter().partition(|id| {
        all.iter()
            .any(|n| &n.front.id == id && n.front.ty == "thread" && n.front.status != "resolved")
    });
    if !pruned.is_empty() {
        let _ = save_topic_queue(store, &live);
    }
    (live, pruned)
}

/// C8 support: has this session rendered a brief for this item? A lease
/// follows a brief — reserve refuses without one on the session's log.
pub fn briefed_this_session(store: &Store, item_id: &str, session: &str) -> bool {
    store.read_log().map_or(false, |log| {
        log.iter().rev().any(|ev| {
            ev.get("op").and_then(|v| v.as_str()) == Some("brief")
                && ev.get("node").and_then(|v| v.as_str()) == Some(item_id)
                && ev.get("session").and_then(|v| v.as_str()) == Some(session)
        })
    })
}

fn live_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".sessions-live.json")
}

/// Best-effort heartbeat: write verbs touch this so a fresh incarnation can
/// tell whether "its" session was active moments ago (double-chat tell).
pub fn touch_session(store: &Store, session: &str) {
    let mut map: BTreeMap<String, String> = fs::read_to_string(live_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    map.insert(session.to_string(), Store::now());
    if let Ok(s) = serde_json::to_string_pretty(&map) {
        let _ = fs::write(live_path(store), s + "\n");
    }
}

pub fn last_seen(store: &Store, session: &str) -> Option<String> {
    let map: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(live_path(store)).ok()?).ok()?;
    map.get(session).cloned()
}

fn bindings_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".chat-sessions.json")
}

fn adopt_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".adopt-request.json")
}

/// Bind a Claude chat session_id to a q session (machine-local).
pub fn bind_chat(store: &Store, chat_id: &str, q_session: &str) -> Result<()> {
    let mut map: BTreeMap<String, String> = fs::read_to_string(bindings_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    map.insert(chat_id.to_string(), q_session.to_string());
    fs::write(bindings_path(store), serde_json::to_string_pretty(&map)? + "\n")?;
    Ok(())
}

pub fn chat_binding(store: &Store, chat_id: &str) -> Option<String> {
    let map: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(bindings_path(store)).ok()?).ok()?;
    map.get(chat_id).cloned()
}

fn actors_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".chat-actors.json")
}

/// SessionStart records which model a chat runs (when the harness provides
/// it); the PreToolUse hook injects it as QUARRY_ACTOR so agents never set
/// attribution by hand — same class, same cure as session identity.
pub fn record_chat_actor(store: &Store, chat_id: &str, model: &str) {
    let mut map: BTreeMap<String, String> = fs::read_to_string(actors_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    map.insert(chat_id.to_string(), model.to_string());
    if let Ok(s) = serde_json::to_string_pretty(&map) {
        let _ = fs::write(actors_path(store), s + "\n");
    }
}

pub fn chat_actor(store: &Store, chat_id: &str) -> Option<String> {
    let map: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(actors_path(store)).ok()?).ok()?;
    map.get(chat_id).cloned()
}

/// Provenance derivation keys on "claude" in the actor string; a display
/// name like "Fable 5" would silently derive USER provenance. Any actor the
/// hook injects passes through this guard.
pub fn safe_actor(model: &str) -> String {
    if model.to_lowercase().contains("claude") {
        model.to_string()
    } else {
        format!("claude:{}", model)
    }
}

/// `q session adopt` writes this; the NEXT PreToolUse hook (which knows the
/// chat's session_id) consumes it and binds. TTL 120s; single pending slot.
pub fn write_adopt_request(store: &Store, q_session: &str) -> Result<()> {
    fs::write(
        adopt_path(store),
        serde_json::to_string_pretty(&serde_json::json!({
            "session": q_session, "ts": Store::now()
        }))? + "\n",
    )?;
    Ok(())
}

pub fn take_adopt_request(store: &Store) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(adopt_path(store)).ok()?).ok()?;
    let _ = fs::remove_file(adopt_path(store));
    let ts = v.get("ts")?.as_str()?;
    let fresh = {
        use time::format_description::well_known::Rfc3339;
        let cutoff = time::OffsetDateTime::now_utc() - time::Duration::seconds(120);
        ts > cutoff.format(&Rfc3339).ok()?.as_str()
    };
    if fresh {
        v.get("session")?.as_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// Resolve the current session's purview to concrete area ids, if registered.
pub fn purview<'a>(store: &Store, all: &'a [Node]) -> Option<(String, Vec<&'a Node>)> {
    let sess = current_session()?;
    let reg = load_sessions(store);
    let p = reg.get(&sess)?;
    let areas: Vec<&Node> = all
        .iter()
        .filter(|n| n.front.ty == "area" && p.areas.contains(&n.front.id))
        .collect();
    Some((sess, areas))
}

/// Does this node attach to any of the given areas?
pub fn in_purview(node: &Node, area_ids: &[&str]) -> bool {
    node.front
        .edges
        .iter()
        .any(|e| e.rel == "about" && area_ids.contains(&e.to.as_str()))
}

pub fn resolve_area_ids(store: &Store, all: &[Node], keys: &[String]) -> Result<Vec<String>> {
    keys.iter()
        .map(|k| {
            let n = store.find(all, k)?;
            if n.front.ty != "area" {
                bail!("\"{}\" is a {}, not an area", n.front.title, n.front.ty);
            }
            Ok(n.front.id.clone())
        })
        .collect::<Result<Vec<_>>>()
        .map_err(|e| anyhow!("{}", e))
}
