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
) -> Result<()> {
    let mut reg = load_sessions(store);
    reg.insert(name.to_string(), Purview { areas, charter });
    fs::write(sessions_path(store), serde_json::to_string_pretty(&reg)? + "\n")?;
    Ok(())
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
