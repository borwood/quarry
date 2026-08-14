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
    /// The session's kind (dc-ad8b): registry data, an open set. Parse and
    /// validation live in `parse_kind` alone; every surface renders the kind
    /// it finds without branching on the value. Kindless entries stay legal
    /// and render exactly as before the field existed.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub charter: Option<String>,
    /// Sessions persist by default — defining one makes it re-enterable
    /// (launcher or adopt). Ephemeral is the marked odd case: this chat only.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub ephemeral: bool,
}

/// The charter-at-wake render (it-sumw): one text, every wake choke point.
/// dc-ydvb names the charter as kind's home until orientation consumes it —
/// q session resume and the SessionStart orient both print this line, so a
/// session cannot wake without meeting its own kind. None when the session
/// has no charter: those wake exactly as before.
pub fn charter_line(p: &Purview) -> Option<String> {
    p.charter.as_ref().map(|c| format!("charter: {}", c))
}

/// THE one validation point for session kind (dc-ad8b, it-skpa). The set is
/// OPEN: adding a kind is a data change plus a new arm HERE — never a sweep.
/// Nothing downstream re-validates; surfaces render whatever kind an entry
/// carries.
pub fn parse_kind(s: &str) -> Result<String> {
    let k = s.trim().to_lowercase();
    match k.as_str() {
        "design" | "dispatch" => Ok(k),
        other => bail!(
            "unknown session kind '{}' — known kinds: design · dispatch. The set is open (dc-ad8b): a new kind is one arm in coord::parse_kind.",
            other
        ),
    }
}

/// The kind-at-wake render, charter_line's sibling: one text, every wake
/// choke point, printed beside the charter. Renders the kind the entry
/// carries — no per-kind branch. None when the session is kindless: those
/// wake exactly as before the field.
pub fn kind_line(p: &Purview) -> Option<String> {
    p.kind.as_ref().map(|k| format!("kind: {}", k))
}

/// What a wake surface OWES, derived from the session's kind — the one
/// match point for kind at the wake tier (it-wub5, dc-ad8b). Surfaces
/// consult the shape they are handed, never the kind string, so a new
/// kind's wake is one new arm HERE; unknown or absent kinds fall through
/// the catch-all and keep the generic brief (parse_kind stays the only
/// judge of the string — this maps, it never validates).
pub struct WakeShape {
    /// Lead with what a dispatcher owes: ready in purview, in-flight items
    /// each with its q harvest command, homework residue (dc-wngq — the
    /// dispatch session resumes into its own work, not the user's).
    pub dispatcher_lead: bool,
    /// Enumerate the threads owed to the user. Omitted under the dispatch
    /// kind: threads are not a dispatch session's to settle (dc-wngq).
    pub owed_threads: bool,
}

pub fn wake_shape(kind: Option<&str>) -> WakeShape {
    match kind {
        Some("dispatch") => WakeShape { dispatcher_lead: true, owed_threads: false },
        _ => WakeShape { dispatcher_lead: false, owed_threads: true },
    }
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
    kind: Option<String>,
    charter: Option<String>,
    ephemeral: bool,
) -> Result<()> {
    let mut reg = load_sessions(store);
    reg.insert(name.to_string(), Purview { areas, kind, charter, ephemeral });
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
            "{} already holds a lease — release it first (q release {}) or reserve a different item",
            crate::surface::atom_ref(&crate::surface::atom(&[], item)),
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
        item_title: crate::surface::title_raw(item).to_string(),
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
        bail!("{} holds no lease", crate::surface::atom_ref(&crate::surface::atom(&[], item)));
    };
    if leases[pos].session != session {
        bail!(
            "the lease on {} is held by session {} — theirs to release (or reserve with --steal)",
            crate::surface::atom_ref(&crate::surface::atom(&[], item)),
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

/// Re-home an item's lease under a new session — the dispatch-steal path
/// (dc-qyr5): a steal takes the dispatch WHOLE, lease included, so the
/// globs survive while holder session, actor, and clock reset. The caller
/// logs the steal event; this is the state move only.
pub fn rehome_lease(store: &Store, item: &Node, session: &str, actor: &str) -> Result<()> {
    let mut leases = load_leases(store);
    let Some(l) = leases.iter_mut().find(|l| l.item == item.front.id) else {
        bail!(
            "{} holds no lease to take over",
            crate::surface::atom_ref(&crate::surface::atom(&[], item))
        );
    };
    l.session = session.to_string();
    l.actor = actor.to_string();
    l.since = Store::now();
    save_leases(store, &leases)?;
    Ok(())
}

// ── the dispatch badge (machine-local, per item) ───────────────────────────
//
// The hand-off is a FETCH (dc-zbxj): q dispatch mints a single-use join
// token into a one-line spawn prompt; q join consumes it, binds the acting
// agent identity to the badge in the ASSOCIATION map, and renders the brief
// fresh. Hooks do identity injection only (QUARRY_AGENT / QUARRY_CHAT /
// QUARRY_SESSION); QUARRY_DISPATCH env survives solely as the out-of-hook-
// coverage override. Multi-held dispatch is the normal shape (dc-qyr5): a
// chat holds ANY number of live dispatches, an item belongs to ONE chat —
// held entries are keyed by item, each carrying its holder (the dispatching
// chat's key), cleared per item at harvest/release. Stamping follows the
// WORK, never the holding chat: held entries resolve refusal and boundary
// only; badge resolution for stamping and the write guard reads env and the
// association map. In the no-agent-id fallback (a subagent is otherwise
// indistinguishable from its parent chat — probed 2026-08-13), a chat-keyed
// association may name the dispatching chat itself; that blur is accepted
// and vanishes wherever the harness provides an agent id.

/// The active dispatch, with the contract captured at dispatch time so the
/// write guard can echo it without loading the graph.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DispatchState {
    pub item: String,
    pub item_title: String,
    pub session: String,
    /// The dispatching chat's key ("chat:<id>", or "session:<name>" where no
    /// chat id reached the dispatching shell). Per-item ownership (dc-qyr5):
    /// the item is the map key, the holder rides the entry. Empty only
    /// transiently while loading pre-multi-held files, where the map key WAS
    /// the holder — load_dispatches fills it in.
    #[serde(default)]
    pub holder: String,
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
    /// The single-use join token minted at dispatch (dc-zbxj). Absent on
    /// pre-token entries — a live legacy dispatch still harvests cleanly;
    /// it just has nothing to join.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub token: Option<String>,
    /// The identity key ("agent:<id>" / "chat:<id>" / "session:<name>")
    /// that consumed the token. Re-join by the same identity is idempotent;
    /// a different identity refuses — one badge binds one agent.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub joined: Option<String>,
}

/// The machine-local dispatch state: one held entry per live-dispatched
/// ITEM (dc-qyr5: an item belongs to one chat, a chat holds many), plus the
/// learned acting-chat associations that let hook processes resolve a
/// dispatched agent's chat to its badge.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DispatchMap {
    /// Item id → the live dispatch on it. The entry's holder field names
    /// the dispatching chat; keying by item keeps the double-hold refusal
    /// and the per-item clear exact under multi-held.
    #[serde(default)]
    pub held: BTreeMap<String, DispatchState>,
    /// Acting-chat key ("chat:<id>") → badge item id, learned when a badged
    /// q act carries both QUARRY_DISPATCH and QUARRY_CHAT. Lives and dies
    /// with the held dispatch it points at.
    #[serde(default)]
    pub acting: BTreeMap<String, String>,
}

fn dispatch_path(store: &Store) -> std::path::PathBuf {
    store.root.join("graph").join(".dispatch.json")
}

/// The chat identity the session hook injects (QUARRY_CHAT) — per-chat
/// machine-local state resolves by it inside q processes.
pub fn current_chat() -> Option<String> {
    std::env::var("QUARRY_CHAT").ok().filter(|s| !s.trim().is_empty())
}

/// The agent identity the session hook injects (QUARRY_AGENT) when the
/// harness names one — hooks run in a subagent carry an agent id, and the
/// subagent is otherwise indistinguishable from its parent chat in both
/// hook session_id and environment (probed 2026-08-13).
pub fn current_agent() -> Option<String> {
    std::env::var("QUARRY_AGENT").ok().filter(|s| !s.trim().is_empty())
}

/// The association key for an acting context, most-specific identity first:
/// the agent id (a subagent's only distinguishing mark), then the chat,
/// then the session. None when no identity reached the process at all.
pub fn acting_key(agent: Option<&str>, chat: Option<&str>, session: Option<&str>) -> Option<String> {
    agent
        .map(|a| format!("agent:{}", a))
        .or_else(|| chat.map(|c| format!("chat:{}", c)))
        .or_else(|| session.map(|s| format!("session:{}", s)))
}

/// The key a dispatch is held under: the dispatching chat where the session
/// hook's injection reached this process, the q session otherwise.
pub fn dispatch_key(session: &str) -> String {
    current_chat()
        .map(|c| format!("chat:{}", c))
        .unwrap_or_else(|| format!("session:{}", session))
}

/// Load the full dispatch state. Tolerates both legacy on-disk shapes: the
/// single-slot file (a bare DispatchState at top level) and the per-chat-
/// keyed map (held keyed by dispatching chat, no holder field) — each
/// migrates in memory to the per-item keying with the old key becoming the
/// holder, and persists in the new shape at the next save. A live dispatch
/// survives the upgrade.
pub fn load_dispatches(store: &Store) -> DispatchMap {
    let Ok(s) = fs::read_to_string(dispatch_path(store)) else {
        return DispatchMap::default();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) else {
        return DispatchMap::default();
    };
    if v.get("item").map_or(false, |x| x.is_string()) {
        // Legacy single-slot shape: the session key was the holder.
        if let Ok(mut d) = serde_json::from_value::<DispatchState>(v) {
            let mut m = DispatchMap::default();
            d.holder = format!("session:{}", d.session);
            m.held.insert(d.item.clone(), d);
            return m;
        }
        return DispatchMap::default();
    }
    let mut m: DispatchMap = serde_json::from_value(v).unwrap_or_default();
    // Re-key by item (dc-qyr5). A pre-multi-held entry carries no holder —
    // its map key WAS the dispatching chat, so the key moves into the field.
    // Current-shape entries (key == item, holder set) reinsert unchanged.
    let held = std::mem::take(&mut m.held);
    for (k, mut d) in held {
        if d.holder.is_empty() {
            d.holder = k;
        }
        m.held.insert(d.item.clone(), d);
    }
    m
}

fn save_dispatches(store: &Store, m: &DispatchMap) -> Result<()> {
    if m.held.is_empty() && m.acting.is_empty() {
        let _ = fs::remove_file(dispatch_path(store));
        return Ok(());
    }
    fs::write(dispatch_path(store), serde_json::to_string_pretty(m)? + "\n")?;
    Ok(())
}

/// Record a dispatch under its item key (upsert — a re-dispatch or steal
/// replaces the item's entry; every other item's entry is untouched). The
/// state carries its holder.
pub fn save_dispatch(store: &Store, d: &DispatchState) -> Result<()> {
    let mut m = load_dispatches(store);
    m.held.insert(d.item.clone(), d.clone());
    save_dispatches(store, &m)
}

/// Every live dispatch a chat holds, by its holder key — multi-held
/// (dc-qyr5): any number, enumerated in item order.
pub fn held_dispatches(store: &Store, holder: &str) -> Vec<DispatchState> {
    load_dispatches(store)
        .held
        .into_values()
        .filter(|d| d.holder == holder)
        .collect()
}

/// The live dispatch on this item, if any chat holds one — the per-item
/// read harvest, trace, the double-hold refusal, and the write guard's
/// contract echo use. Exact: held entries key by item.
pub fn dispatch_for_item(store: &Store, item_id: &str) -> Option<DispatchState> {
    load_dispatches(store).held.get(item_id).cloned()
}

/// The outcome of presenting a join token (dc-zbxj).
pub enum JoinBind {
    /// Token consumed: the identity key is newly bound to the badge and the
    /// association is recorded.
    Bound(DispatchState),
    /// This identity already consumed the token — idempotent re-join.
    Rejoined(DispatchState),
}

/// Consume a join token: single-use, machine-local. Finds the held entry
/// carrying the token, marks it joined by this identity, and records the
/// acting association that stamping and the write guard resolve. A second
/// DIFFERENT identity refuses — one badge binds one agent (a chat may hold
/// many badges, dc-qyr5, but each badge is one agent's); re-dispatch mints
/// a fresh token when a new agent takes the work over, and a steal takes
/// the whole dispatch to another chat.
pub fn consume_join_token(store: &Store, token: &str, identity: &str) -> Result<JoinBind> {
    let mut m = load_dispatches(store);
    let Some(key) = m
        .held
        .iter()
        .find(|(_, d)| d.token.as_deref() == Some(token))
        .map(|(k, _)| k.clone())
    else {
        bail!(
            "unknown join token '{}' — join tokens are minted by q dispatch (single-use) and die at harvest. If the dispatch was re-issued or harvested, ask the dispatcher; q dispatch <item> mints a fresh token into a new spawn prompt.",
            token
        );
    };
    let joined = m.held[&key].joined.clone();
    match joined {
        None => {
            let d = m.held.get_mut(&key).expect("entry just found");
            d.joined = Some(identity.to_string());
            let bound = d.clone();
            m.acting.insert(identity.to_string(), bound.item.clone());
            save_dispatches(store, &m)?;
            Ok(JoinBind::Bound(bound))
        }
        Some(j) if j == identity => Ok(JoinBind::Rejoined(m.held[&key].clone())),
        Some(other) => {
            let d = &m.held[&key];
            bail!(
                "this token was already consumed by another identity ({}) — a join token is single-use and one badge binds one agent. If a second agent is to work \"{}\" ({}), the dispatcher re-dispatches (minting a fresh token) or dispatches a separate item.",
                other, d.item_title, d.item
            )
        }
    }
}

/// Tie an acting identity key ("agent:<id>" / "chat:<id>" / "session:<name>")
/// to a live badge: stamping and the write guard resolve the identity's work
/// through this. q join CONSTRUCTS the association; a badged env act (the
/// out-of-hook-coverage override) teaches it. Recorded only while some chat
/// holds the dispatch. A key that also holds a dispatch may legally carry an
/// association (the no-agent-id fallback, where a subagent is
/// indistinguishable from its parent chat) — held and acting are separate
/// maps: stamping reads acting, refusal reads held.
pub fn record_acting(store: &Store, key: &str, badge: &str) {
    let mut m = load_dispatches(store);
    if !m.held.contains_key(badge) {
        return;
    }
    if m.acting.get(key).map(|b| b.as_str()) == Some(badge) {
        return;
    }
    m.acting.insert(key.to_string(), badge.to_string());
    let _ = save_dispatches(store, &m);
}

/// Chat-keyed convenience over record_acting.
pub fn record_acting_chat(store: &Store, chat_id: &str, badge: &str) {
    record_acting(store, &format!("chat:{}", chat_id), badge);
}

/// The env-transported form: a badged q act from an identified context
/// teaches the machine which agent (or chat) wears the badge — the
/// out-of-hook-coverage override's road into the association map; q join is
/// the constructed road. Called on every logged event; early-outs make it
/// O(1) when there is nothing to learn.
pub fn note_acting(store: &Store) {
    let badge = std::env::var("QUARRY_DISPATCH").ok().filter(|s| !s.trim().is_empty());
    let Some(b) = badge else { return };
    // Best key only: with an agent id present the chat id may be the
    // PARENT's (subagent indistinguishability), so a chat-keyed record
    // would smear the badge onto the dispatching chat.
    if let Some(key) = acting_key(current_agent().as_deref(), current_chat().as_deref(), None) {
        record_acting(store, &key, &b);
    }
}

/// Clear every trace of an item's badge — the held entry and acting
/// associations alike (harvest, land, and the steal's take-over all clear).
/// Exact under multi-held: item-keyed removal never touches the same chat's
/// other live dispatches.
pub fn clear_dispatch(store: &Store, item_id: &str) {
    let mut m = load_dispatches(store);
    let removed = m.held.remove(item_id).is_some();
    let a = m.acting.len();
    m.acting.retain(|_, b| b != item_id);
    if removed || m.acting.len() != a {
        let _ = save_dispatches(store, &m);
    }
}

/// Resolve the badge that STAMPS this acting context's work — env identity
/// or a joined/taught association, NEVER the held entry (dc-zbxj: stamping
/// follows the work, so the dispatching chat's unrelated acts never stamp
/// into its dispatch's trace). QUARRY_DISPATCH env wins (explicit,
/// per-shell — the out-of-hook-coverage override); then the association map
/// by agent, chat, then session key, each honored only while its badge's
/// dispatch is live. A context with no identity, or one foreign to every
/// association, resolves nothing — no-attribution over mis-attribution.
pub fn badge_for(
    store: &Store,
    agent: Option<&str>,
    chat_id: Option<&str>,
    session: Option<&str>,
) -> Option<String> {
    if let Ok(b) = std::env::var("QUARRY_DISPATCH") {
        if !b.trim().is_empty() {
            return Some(b);
        }
    }
    let m = load_dispatches(store);
    let keys = [
        agent.map(|a| format!("agent:{}", a)),
        chat_id.map(|c| format!("chat:{}", c)),
        session.map(|s| format!("session:{}", s)),
    ];
    for key in keys.iter().flatten() {
        if let Some(b) = m.acting.get(key) {
            if m.held.contains_key(b) {
                return Some(b.clone());
            }
        }
    }
    None
}

/// The acting badge for a q process: env identity (QUARRY_AGENT,
/// QUARRY_CHAT, QUARRY_SESSION are hook-injected) resolved through
/// badge_for. Stamping resolution — held entries never appear here.
pub fn current_dispatch_badge(store: &Store) -> Option<String> {
    badge_for(
        store,
        current_agent().as_deref(),
        current_chat().as_deref(),
        current_session().as_deref(),
    )
}

/// Every badge that captures this context's BOUNDARY verbs — wider than the
/// stamping resolution: every HELD entry this chat holds counts (a
/// dispatching chat is mid-dispatch even though its acts no longer stamp),
/// alongside env and the acting associations. Multi-held (dc-qyr5): the
/// boundary harvests ALL, so this enumerates rather than first-finds.
/// Never used for stamping.
pub fn boundary_badges(store: &Store) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(b) = current_dispatch_badge(store) {
        out.push(b);
    }
    let m = load_dispatches(store);
    let mine: Vec<String> = [
        current_chat().map(|c| format!("chat:{}", c)),
        current_session().map(|s| format!("session:{}", s)),
    ]
    .into_iter()
    .flatten()
    .collect();
    for d in m.held.values() {
        if mine.contains(&d.holder) && !out.contains(&d.item) {
            out.push(d.item.clone());
        }
    }
    out
}

/// C8's logic applied to boundary acts (it-ymsj): wrap and session
/// resume/retire are the DISPATCHER'S verbs. Under any active badge of this
/// context — shell env, an acting association (a joined agent), or a held
/// entry of this chat, any alone suffices — they refuse with a teaching
/// error that enumerates EVERY live held dispatch with its q harvest
/// command (dc-qyr5: the boundary harvests all; no parking a dispatch
/// across a boundary). The incident this guard exists for: a dispatched
/// agent ran q wrap wearing the dispatcher's injected session identity and
/// consumed its session cursors. A chat with no badge of its own is free to
/// wrap while other chats' dispatches fly — the decisions session keeps its
/// boundary while a steward has work in flight.
pub fn boundary_refusal(store: &Store, verb: &str) -> Option<String> {
    let badges = boundary_badges(store);
    if badges.is_empty() {
        return None;
    }
    let m = load_dispatches(store);
    let lines: Vec<String> = badges
        .iter()
        .map(|b| match m.held.get(b) {
            Some(d) => format!("  · \"{}\" ({}) — q harvest {}", d.item_title, b, b),
            None => format!("  · {} — q harvest {}", b, b),
        })
        .collect();
    Some(format!(
        "boundary-verb capture: {verb} is a session-boundary act, and this chat is mid-dispatch — {n} live dispatch(es) held:\n{list}\nA badged boundary verb runs wearing the dispatching session's identity and consumes its cursors — the incident class this guard exists for. A dispatched agent reports against the RETURN spec and stops; the boundary belongs to the dispatcher, who harvests every arc first — no parking a dispatch across a boundary (dc-qyr5).",
        n = badges.len(),
        list = lines.join("\n")
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
        let what = all
            .iter()
            .find(|n| n.front.id == id)
            .map(|n| crate::surface::atom_line(&crate::surface::atom(all, n)))
            .unwrap_or_else(|| format!("({})", id));
        let line = format!("[{}] {} by session {}", op, what, ev_key);
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
                bail!("{} is not an area", crate::surface::atom_ref(&crate::surface::atom(all, n)));
            }
            Ok(n.front.id.clone())
        })
        .collect::<Result<Vec<_>>>()
        .map_err(|e| anyhow!("{}", e))
}
