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

/// The witness seat (dc-mpg8): an acceptance-authoring context whose
/// session carries a registered kind other than design holds the witness
/// pen — transcription only, held by construction (the pen checks in
/// ops). THE one predicate point, the parse_kind pattern: a new kind is
/// witness-or-not by falling in or out of the design arm here.
///
/// The kindless middle is deliberately NOT a witness: dc-mpg8 says "any
/// session whose kind is not design", but dc-p6z4 teaches the solo
/// station — kindless sessions included — the authoring command directly,
/// and the two rulings contest that middle. Least-committal until ruled
/// (C3): only a session that REGISTERED a non-design kind sits in the
/// witness seat; kindless, unregistered, and unbound contexts keep the
/// design-capable default dc-p6z4 built on. The open call is queued as a
/// thread; widening the seat is one arm change here.
pub struct WitnessSeat {
    pub session: String,
    pub kind: String,
    /// The acting identity key (agent → chat → session precedence) — what
    /// the mark records as author and the executor refusals compare.
    pub key: String,
}

pub fn witness_seat(store: &Store) -> Option<WitnessSeat> {
    let session = current_session()?;
    let kind = load_sessions(store).get(&session)?.kind.clone()?;
    if kind == "design" {
        return None;
    }
    let key = acting_key(
        current_agent().as_deref(),
        current_chat().as_deref(),
        Some(&session),
    )
    .unwrap_or_else(|| format!("session:{}", session));
    Some(WitnessSeat { session, kind, key })
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

/// Does a live lease CONTEND with these globs? Arc report paths on either
/// side never do (it-3prx): a return is a per-arc artifact, unique by
/// construction and naming a file that does not exist yet, so no one can be
/// co-writing it. Counting them would make the reports zone contended
/// ground — every write-set covering `docs/**` would refuse against every
/// live arc's return — for a protection the write guard never offered
/// anyway: a badged write is judged against the BADGE's own lease, foreign
/// exclusivity unconsulted.
fn lease_overlaps(lease: &Lease, globs: &[String]) -> bool {
    lease
        .globs
        .iter()
        .filter(|lg| !is_arc_report(lg))
        .any(|lg| {
            globs
                .iter()
                .filter(|g| !is_arc_report(g))
                .any(|g| globs_overlap(lg, g))
        })
}

/// The shape floor under every lease (it-x4bb): a `--files` value carrying a
/// comma is ONE dead glob, never a list. The flag is repeatable and carries
/// no value delimiter, so `--files "src/ops.rs,src/render.rs,tests/**"`
/// arrives as a single pattern — and `globs_overlap` compares STATIC PREFIXES
/// by the prefix-of relation, so that pattern matches the first path in the
/// value and nothing after it. Every path past the first comma is leased in
/// name only: the brief prints it as leased, and then `teach::lease_check`'s
/// under-a-badge arm denies the agent's write to it as "outside the leased
/// write-set" — mid-arc, in a seat that cannot fix it (extending a lease is
/// release plus re-reserve, and an agent may not release its own lease). It
/// degrades quietly rather than failing: the first path keeps working, so the
/// arc starts, spends context, and dies at whichever write comes second.
///
/// REFUSED, never split — the fork it-x4bb left to build time, settled here:
/// one meaning per flag, never a guess. A comma is legal inside a real
/// filename, so a silent split can mint exactly the dead globs this check
/// exists to prevent; the refusal instead costs one re-run at the station
/// that CAN fix it, and teaches the repeatable flag once.
///
/// One predicate point, two call sites (the stations it-x4bb named):
/// `reserve` calls it, so every lease-taking road inherits it — `q reserve`
/// and the dispatch fallback to an item's recorded write-set alike — and
/// `ops::dispatch` calls it on the flag value ahead of any mutation, so a
/// mistyped fire logs no brief event, takes no lease, and flips no status.
pub fn check_glob_shapes(globs: &[String]) -> Result<()> {
    let Some(bad) = globs.iter().find(|g| g.contains(',')) else {
        return Ok(());
    };
    let repeated = bad
        .split(',')
        .map(str::trim)
        .filter(|g| !g.is_empty())
        .map(|g| format!("--files \"{}\"", g))
        .collect::<Vec<_>>()
        .join(" ");
    bail!(
        "--files takes ONE glob per flag and no value delimiter, so {:?} would be leased whole as a single pattern, never a list: the overlap test compares static prefixes, so that lease matches only the FIRST path in the value — every path after a comma is leased in name only, printed as leased in the agent's brief and then DENIED to it at the write by its own badge, mid-arc, in a seat that cannot extend a lease (it-x4bb). The flag is REPEATABLE — pass each glob its own: {}. (An item's recorded write-set, which a dispatch falls back to, is authored the same way, one glob per act: q set <item> write-set+=\"<glob>\".)",
        bad, repeated
    );
}

// ── the arc's return rides the lease (it-3prx) ─────────────────────────────
//
// The brief demands a report and harvest prints its registration command,
// but a lease derived from `--files` or an item's recorded write-set names
// only the CODE the work touches — so the write guard denied the one
// artifact the contract mandates, mid-arc, in a seat that cannot extend a
// lease. The firing station closes it: every dispatch derives the arc's own
// report path and leases it beside the write-set, so the agent writes and
// registers its return from its own seat.
//
// ONE PATH PER ARC, never the zone. A `docs/reports/**` glob on every
// dispatch would make the reports directory a co-write zone nobody asked
// for — parallel dispatch is the normal shape (dc-ydvb), and `globs_overlap`
// compares static prefixes, so every concurrent arc would collide there and
// each agent would hold write access to its neighbours' returns. Two arcs
// writing distinct files need no shared lease, only their own paths: the
// derived path carries the item id, so no two arcs' paths are ever prefixes
// of one another and concurrent dispatches never overlap.

/// The reports zone: where a dispatch's RETURN lands.
pub const REPORTS_DIR: &str = "docs/reports/";

/// A concrete report file (never a pattern) inside the reports zone — the
/// shape `arc_report_path` mints and the read side picks back out of a
/// lease. A dispatcher's own broader glob (`docs/reports/**`, passed with
/// `--files`) is deliberately NOT this shape: it survives untouched beside
/// the arc's path, and never stands in for it where a path is printed.
pub fn is_arc_report(glob: &str) -> bool {
    let g = glob.replace('\\', "/");
    g.starts_with(REPORTS_DIR) && g.ends_with(".md") && !g.contains(['*', '?', '['])
}

/// The report path a lease carries for its arc, if any. The brief names it
/// in the RETURN spec and harvest prints it into the registration command —
/// one derivation at the fire, read back everywhere after.
pub fn arc_report_in(globs: &[String]) -> Option<&str> {
    globs.iter().find(|g| is_arc_report(g)).map(|g| g.as_str())
}

/// Point a write-set at `path` as its arc report: any PRIOR arc's report
/// path drops (a re-dispatch is a new arc with its own return — leaving the
/// old path would let arc two overwrite arc one's registered file), every
/// other glob stays. True when the globs changed.
pub fn set_arc_report(globs: &mut Vec<String>, path: &str) -> bool {
    let already = {
        let mut carried = globs.iter().filter(|g| is_arc_report(g));
        carried.next().map(|g| g == path).unwrap_or(false) && carried.next().is_none()
    };
    if already {
        return false;
    }
    globs.retain(|g| !is_arc_report(g));
    globs.push(path.to_string());
    true
}

/// A filename slug in the reports register: lowercase, one hyphen per run of
/// anything else, cut at a hyphen boundary under `cap`.
fn slug(title: &str, cap: usize) -> String {
    let mut s = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            s.push(ch.to_ascii_lowercase());
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    let s = s.trim_matches('-');
    if s.len() <= cap {
        return s.to_string();
    }
    match s[..cap].rfind('-') {
        Some(i) if i > 0 => s[..i].to_string(),
        _ => s[..cap].to_string(),
    }
}

/// THE arc report path, derived at the firing station: the reports register's
/// own shape — `<date>-<slug>-<item>.md` — with the item id making it unique
/// among concurrent arcs. A re-dispatch takes the next free `-arcN` name
/// rather than the file its predecessor left behind: a prior arc's report is
/// a registered doc node whose blob stamp still names that path (the partial
/// and landed reports of a re-dispatched item both stand as record), so
/// overwriting it would make one of them a silent lie.
pub fn arc_report_path(store: &Store, item_id: &str, title: &str) -> String {
    let s = slug(title, 40);
    let base = if s.is_empty() {
        format!("{}{}-{}", REPORTS_DIR, Store::today(), item_id)
    } else {
        format!("{}{}-{}-{}", REPORTS_DIR, Store::today(), s, item_id)
    };
    let taken = |p: &str| store.work_root.join(p).exists() || store.root.join(p).exists();
    let first = format!("{}.md", base);
    if !taken(&first) {
        return first;
    }
    for n in 2..99 {
        let c = format!("{}-arc{}.md", base, n);
        if !taken(&c) {
            return c;
        }
    }
    format!("{}-arc99.md", base)
}

/// Re-write the globs of an item's live lease — the FIRING station's own
/// hand, never the agent's (extending a lease is release plus re-reserve,
/// and an agent may not release its own lease). A re-dispatch reuses the
/// standing lease, so this is where the new arc's report path replaces its
/// predecessor's. Silent when the item holds no lease: a research dispatch
/// leases nothing at all.
pub fn set_lease_globs(store: &Store, item_id: &str, globs: &[String]) -> Result<()> {
    let mut leases = load_leases(store);
    let Some(l) = leases.iter_mut().find(|l| l.item == item_id) else {
        return Ok(());
    };
    l.globs = globs.to_vec();
    save_leases(store, &leases)
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
    // The shape floor (it-x4bb), here so every lease-taking road inherits it:
    // a comma-joined value is one dead glob, and the denial it causes lands
    // on the agent, mid-arc, in a seat that cannot fix it.
    check_glob_shapes(&globs)?;
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
// and vanishes wherever the harness provides an agent id. The map holds ONE
// live badge per identity by construction (it-tanf, user-ruled 2026-08-20):
// bind_acting is the sole writer and never overwrites a live binding, so
// q join refuses a second badge and the sequential multi-join shape is dead
// — multi-item work is separate dispatches, or the bundle (th-zzqv) when it
// lands.

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
    /// a different identity refuses — one badge binds one agent, and the
    /// converse holds by construction too: one identity binds one live
    /// badge (it-tanf; see bind_acting).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub joined: Option<String>,
    /// The MODEL the dispatcher spawned this arc with, stamped at the fire
    /// (it-xcvb). The harness hands a subagent's hook input no model of any
    /// kind — measured 2026-08-21: a PreToolUse firing inside a subagent
    /// carries session_id, transcript_path, cwd, prompt_id, permission_mode,
    /// agent_id, agent_type, effort, and the tool fields, and nothing else —
    /// and only SessionStart carries `model`, which a subagent never fires.
    /// So the model cannot be learned at the agent's seat; the dispatcher is
    /// the one party that knows it, and the badge is where it already writes.
    /// Absent means "not stated": the arc keeps inheriting the dispatching
    /// chat's recorded model, which is correct exactly when the spawn
    /// inherited it too.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model: Option<String>,
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

/// The held dispatch a live token belongs to — a PEEK, never a consume:
/// the witness executor check (dc-mpg8) must refuse the authoring badge
/// BEFORE the single-use token is spent, so a refused join leaves the
/// token live for the right agent.
pub fn dispatch_for_token(store: &Store, token: &str) -> Option<DispatchState> {
    load_dispatches(store)
        .held
        .into_values()
        .find(|d| d.token.as_deref() == Some(token))
}

/// The outcome of presenting a join token (dc-zbxj).
pub enum JoinBind {
    /// Token consumed: the identity key is newly bound to the badge and the
    /// association is recorded.
    Bound(DispatchState),
    /// This identity already consumed the token — idempotent re-join.
    Rejoined(DispatchState),
}

/// The identity's standing LIVE badge: its acting row, honored only while
/// that badge's dispatch is held. A row pointing at a cleared dispatch is
/// residue (clear_dispatch prunes; this read never trusts it), so a
/// harvested arc frees its identity for the next join by construction.
pub fn live_acting_badge(m: &DispatchMap, identity: &str) -> Option<String> {
    m.acting
        .get(identity)
        .filter(|b| m.held.contains_key(b.as_str()))
        .cloned()
}

/// The ONE writer of the acting map (it-tanf): an identity binds AT MOST ONE
/// live badge, held by construction, not discipline. The map's keying always
/// gave the shape; what this guard kills is the silent overwrite — sequential
/// joins used to re-point an identity's row, so every later act and file
/// write stamped the LAST join, not the item served, and per-item replay and
/// observed-vs-leased misreported (the lexicon-trio incident, 2026-08-15).
/// Ok(true): newly bound. Ok(false): already bound to this badge — nothing
/// to write. Err(live badge): the identity wears a DIFFERENT live badge; the
/// map is untouched, and the caller refuses loudly (the join road) or
/// declines the learn (the env road, where the per-shell QUARRY_DISPATCH
/// override still stamps env-first and no attribution is lost).
fn bind_acting(m: &mut DispatchMap, identity: &str, badge: &str) -> std::result::Result<bool, String> {
    if let Some(live) = live_acting_badge(m, identity) {
        if live != badge {
            return Err(live);
        }
        return Ok(false);
    }
    m.acting.insert(identity.to_string(), badge.to_string());
    Ok(true)
}

/// Consume a join token: single-use, machine-local. Finds the held entry
/// carrying the token, marks it joined by this identity, and records the
/// acting association that stamping and the write guard resolve. A second
/// DIFFERENT identity refuses — one badge binds one agent (a chat may hold
/// many badges, dc-qyr5, but each badge is one agent's); re-dispatch mints
/// a fresh token when a new agent takes the work over, and a steal takes
/// the whole dispatch to another chat. The converse is construction too
/// (it-tanf): one identity binds one LIVE badge — a second join refuses
/// before the token is spent, naming the live badge and the roads out, so
/// sequential multi-join can never silently re-point stamping. Re-join of
/// the identity's own badge stays the idempotent read.
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
    if let Some(other) = joined.as_ref().filter(|j| j.as_str() != identity) {
        let d = &m.held[&key];
        bail!(
            "this token was already consumed by another identity ({}) — a join token is single-use and one badge binds one agent. If a second agent is to work \"{}\" ({}), the dispatcher re-dispatches (minting a fresh token) or dispatches a separate item.",
            other, d.item_title, d.item
        );
    }
    // One badge per identity (it-tanf), refused BEFORE anything is spent:
    // binding a second live badge would re-point this identity's stamping
    // resolution, so every later act and file write lands on the last join,
    // not the item it serves — the earlier badges observe nothing but their
    // join events, and every surface harvest trusts misreports. The token
    // stays live for the right agent. Checked on the re-join arm too: a
    // legacy blur (joined recorded pre-fix while acting points elsewhere)
    // must not re-render a brief for an item this identity no longer stamps.
    if let Some(live) = live_acting_badge(&m, identity) {
        if live != m.held[&key].item {
            let live_title = m
                .held
                .get(&live)
                .map(|d| d.item_title.clone())
                .unwrap_or_default();
            let d = &m.held[&key];
            bail!(
                "one agent, one badge (it-tanf): this identity ({}) is already bound to a live badge — \"{}\" ({}) — and the acting map holds one badge per identity by construction, so a second join cannot bind: it would re-point stamping, and every later act and file write would land on the last join, not the item it serves. Nothing was spent; the token for \"{}\" ({}) stays live. The roads out: report against \"{}\"'s RETURN spec and stop, so the dispatcher harvests that arc — harvest frees this identity; or the dispatcher hands \"{}\" to a separate agent as its own dispatch. Multi-item work under one agent is the bundle shape (th-zzqv), not yet built.",
                identity, live_title, live, d.item_title, d.item, live_title, d.item_title
            );
        }
    }
    match joined {
        None => {
            let d = m.held.get_mut(&key).expect("entry just found");
            d.joined = Some(identity.to_string());
            let bound = d.clone();
            // Cannot refuse: the live-badge check above already ruled this
            // identity free (or bound to exactly this badge).
            let _ = bind_acting(&mut m, identity, &bound.item);
            save_dispatches(store, &m)?;
            Ok(JoinBind::Bound(bound))
        }
        Some(_) => {
            // Same identity by the filter above — idempotent read, and (the
            // pin-restore pattern, dc-g5x5) a re-join restores a lost acting
            // row: the read repairs the stamping road without a second act.
            let item = m.held[&key].item.clone();
            if bind_acting(&mut m, identity, &item) == Ok(true) {
                save_dispatches(store, &m)?;
            }
            Ok(JoinBind::Rejoined(m.held[&key].clone()))
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
/// maps: stamping reads acting, refusal reads held. One badge per identity
/// (it-tanf): a key already bound to a DIFFERENT live badge declines the
/// learn — the map never re-points, and the env-carried badge still stamps
/// its own shell's acts env-first, so nothing is lost where the override is
/// deliberate.
pub fn record_acting(store: &Store, key: &str, badge: &str) {
    let mut m = load_dispatches(store);
    if !m.held.contains_key(badge) {
        return;
    }
    if bind_acting(&mut m, key, badge) == Ok(true) {
        let _ = save_dispatches(store, &m);
    }
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

/// Clear every trace of an item's badge — the held entry, acting
/// associations, and the store pins the badge planted (dc-g5x5) alike
/// (harvest, land, and the steal's take-over all clear). Exact under
/// multi-held: item-keyed removal never touches the same chat's other live
/// dispatches.
pub fn clear_dispatch(store: &Store, item_id: &str) {
    let mut m = load_dispatches(store);
    let removed = m.held.remove(item_id).is_some();
    let a = m.acting.len();
    m.acting.retain(|_, b| b != item_id);
    if removed || m.acting.len() != a {
        let _ = save_dispatches(store, &m);
    }
    crate::store::clear_pins(&store.root, item_id);
    // Badge-scoped attention dies with the badge (dc-pwyd): a re-dispatch's
    // next agent reads with its own eyes, not a dead arc's cursors.
    clear_area_reads(store, &badge_attention_key(item_id));
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

/// C8's logic applied to boundary acts (it-ymsj): wrap and session resume
/// are the DISPATCHER'S verbs. Under any active badge of this context —
/// shell env, an acting association (a joined agent), or a held entry of
/// this chat, any alone suffices — they refuse with a teaching error that
/// enumerates EVERY live held dispatch with its q harvest command
/// (dc-qyr5: the boundary harvests all; no parking a dispatch across a
/// boundary). The incident this guard exists for: a dispatched agent ran
/// q wrap wearing the dispatcher's injected session identity and consumed
/// its session cursors. A chat with no badge of its own is free to wrap
/// while other chats' dispatches fly — the decisions session keeps its
/// boundary while a steward has work in flight. Retire routes through
/// retire_refusal below (it-e6wq): this capture applies there only when
/// the retiree is the chat's own session.
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

/// The retire guard, scoped to the RETIREE (it-e6wq): the blanket boundary
/// capture treated every retire as the chat closing its own arc, which is
/// overbroad once sessions and dispatches multiply — retiring an ephemeral
/// third session touches nothing about a flying dispatch. Refusal fires
/// only when the retiree is implicated: (1) the chat's own session while
/// this context is mid-dispatch — that IS closing out with harvest owed,
/// so the standing boundary refusal speaks verbatim; (2) a retiree with a
/// dispatch of its own in flight — retire releases its leases, ripping the
/// zone out from under a working agent, so the refusal points at that
/// dispatch's q harvest. A third session with no live dispatch retires
/// clean while unrelated badges fly; its idle leases release with last
/// rites (retire's standing semantics).
pub fn retire_refusal(store: &Store, name: &str) -> Option<String> {
    // (1) The chat's own session: the retire IS this chat's boundary act,
    // so the whole boundary capture applies — every held dispatch of this
    // context enumerates with its q harvest command, exactly as before.
    if current_session().as_deref() == Some(name) {
        if let Some(msg) = boundary_refusal(store, "q session retire") {
            return Some(msg);
        }
    }
    // (2) The retiree itself mid-dispatch: any held entry whose dispatching
    // session is the retiree — matched by the entry's session field, or by
    // a holder recorded as the session key where no chat id reached the
    // dispatching shell.
    let m = load_dispatches(store);
    let holder_key = format!("session:{}", name);
    let theirs: Vec<&DispatchState> = m
        .held
        .values()
        .filter(|d| d.session == name || d.holder == holder_key)
        .collect();
    if theirs.is_empty() {
        return None;
    }
    let lines: Vec<String> = theirs
        .iter()
        .map(|d| format!("  · \"{}\" ({}) — q harvest {}", d.item_title, d.item, d.item))
        .collect();
    Some(format!(
        "retire refused: session {name} is mid-dispatch — {n} live dispatch(es) in flight:\n{list}\nRetiring the session would release its leases out from under a working agent. Harvest each arc first — no parking a dispatch across a boundary (dc-qyr5) — then retire.",
        n = theirs.len(),
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

/// One observed touch: the store-relative path when resolution held, the raw
/// path marked `unresolved` when it did not (it-bj3b: a badged write whose
/// path escapes store-relative resolution is RECORDED — resolution failure
/// is visible accounting, never a dropped fact). `via` names the sight
/// channel: None = a tool write the guard saw exactly; "shell" = a target
/// parsed from a command string, best-effort by nature.
#[derive(Clone, PartialEq)]
pub struct Touch {
    pub path: String,
    pub via: Option<String>,
    pub unresolved: bool,
}

/// Append one touched path (call only for paths not already accrued —
/// `touches_for` gives the prior set). Best-effort: observation never fails
/// a write.
pub fn accrue_touch(store: &Store, key: &str, rel_path: &str) {
    accrue_touch_ext(store, key, rel_path, None, false);
}

/// The full-channel accrual: tool or shell-parsed, resolved or not.
pub fn accrue_touch_ext(
    store: &Store,
    key: &str,
    path: &str,
    via: Option<&str>,
    unresolved: bool,
) {
    use std::io::Write as _;
    let mut line = serde_json::json!({"ts": Store::now(), "key": key, "path": path});
    if let Some(v) = via {
        line["via"] = serde_json::json!(v);
    }
    if unresolved {
        line["unresolved"] = serde_json::json!(true);
    }
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(touched_path(store))
    {
        let _ = writeln!(f, "{}", line);
    }
}

/// Every distinct touch for one key, in first-touch order — resolved and
/// unresolved alike, each carrying its sight channel. Distinctness is by
/// path: the first channel that saw a path keeps it.
pub fn touches_for(store: &Store, key: &str) -> Vec<Touch> {
    let Ok(s) = fs::read_to_string(touched_path(store)) else {
        return vec![];
    };
    let mut out: Vec<Touch> = Vec::new();
    for line in s.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        if v.get("key").and_then(|x| x.as_str()) != Some(key) {
            continue;
        }
        let Some(p) = v.get("path").and_then(|x| x.as_str()) else { continue };
        if out.iter().any(|t| t.path == p) {
            continue;
        }
        out.push(Touch {
            path: p.to_string(),
            via: v.get("via").and_then(|x| x.as_str()).map(String::from),
            unresolved: v.get("unresolved").and_then(|x| x.as_bool()).unwrap_or(false),
        });
    }
    out
}

/// Distinct RESOLVED touched paths for one key, in first-touch order — the
/// store-relative set consumers match against globs and citations. Unresolved
/// raw paths never appear here; they render at harvest from `touches_for`.
pub fn touched_for(store: &Store, key: &str) -> Vec<String> {
    touches_for(store, key)
        .into_iter()
        .filter(|t| !t.unresolved)
        .map(|t| t.path)
        .collect()
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

/// The session half of the attention key: the bound session, or "unbound"
/// for a chat with no identity (imprecise across parallel unbound chats —
/// an accepted, machine-local blur). Attention consumers key on
/// attention_key, which falls back here for unbadged contexts.
pub fn session_key() -> String {
    current_session().unwrap_or_else(|| "unbound".into())
}

/// The attention key for the ACTING identity (dc-pwyd: attention rides the
/// actor — watermarks and drift-delivery consumption answer
/// has-this-READER-seen-it). A context resolving a live dispatch badge
/// spends badge-scoped attention: a joined agent's reads advance its own
/// watermarks and consume its own deliveries, never the holding session's,
/// whose env identity the agent merely inherits (it-csm3). Everything else
/// keys by session, exactly as before.
pub fn attention_key(store: &Store) -> String {
    current_dispatch_badge(store)
        .map(|b| badge_attention_key(&b))
        .unwrap_or_else(session_key)
}

/// The badge-scoped attention key. Prefixed so a badge row can never
/// collide with a session name in the same map.
pub fn badge_attention_key(badge: &str) -> String {
    format!("badge:{}", badge)
}

/// The attention key an EVENT spent when it was logged: the badge it
/// stamped (a joined agent's act carries the dispatch stamp), else its
/// session. touch_area's own/foreign test compares reader to author on
/// this key, so a badged act reads as foreign to the holding session whose
/// env it inherited — the session's eyes never saw the write (it-csm3).
fn event_attention_key(ev: &serde_json::Value) -> String {
    match ev.get("dispatch").and_then(|v| v.as_str()) {
        Some(b) => badge_attention_key(b),
        None => ev
            .get("session")
            .and_then(|v| v.as_str())
            .unwrap_or("unbound")
            .to_string(),
    }
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

/// Has this reader (an attention key: badge-scoped or session) read — or
/// been delivered — this area at all?
pub fn has_area_read(store: &Store, reader: &str, area_id: &str) -> bool {
    load_area_reads(store)
        .get(reader)
        .map_or(false, |m| m.contains_key(area_id))
}

fn record_area_read_at(store: &Store, reader: &str, area_id: &str, index: usize) {
    let mut m = load_area_reads(store);
    m.entry(reader.to_string())
        .or_default()
        .insert(area_id.to_string(), Cursor::Index(index as u64));
    if let Ok(s) = serde_json::to_string_pretty(&m) {
        let _ = fs::write(area_reads_path(store), s + "\n");
    }
}

/// Record that this reader has read (or been delivered) this area's
/// neighborhood — machine-local; the committed log stays mutations-only.
pub fn record_area_read(store: &Store, reader: &str, area_id: &str) {
    let idx = store.read_log().map(|l| l.len()).unwrap_or(0);
    record_area_read_at(store, reader, area_id, idx);
}

/// Drop every area-read row a reader holds — a badge's attention dies with
/// its dispatch (clear_dispatch), so a re-dispatched item's next agent
/// starts with its own eyes, not a dead arc's cursors.
pub fn clear_area_reads(store: &Store, reader: &str) {
    let mut m = load_area_reads(store);
    if m.remove(reader).is_some() {
        if let Ok(s) = serde_json::to_string_pretty(&m) {
            let _ = fs::write(area_reads_path(store), s + "\n");
        }
    }
}

/// The per-(reader, area) watermark surface (user-ruled 2026-08-09; reader
/// keying per dc-pwyd — attention rides the actor).
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

/// Check (and advance) a reader's watermark over one area. The reader is
/// an attention key (badge-scoped for a joined agent, session otherwise —
/// dc-pwyd). Own events — same attention key — advance silently; foreign
/// create/set/link/body events on nodes in the area come back as delta
/// lines, capped, each said once. A badged act is foreign to the holding
/// session it inherited env from: those eyes never saw it (it-csm3). Exact
/// by construction: the cursor is a log position, not a timestamp.
pub fn touch_area(store: &Store, all: &[Node], reader: &str, area_id: &str) -> AreaTouch {
    let Some(cur) = load_area_reads(store)
        .get(reader)
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
        if event_attention_key(ev) == reader {
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
        // Attribute to the hands that wrote: a badged act names its
        // dispatch, not the session identity it inherited (it-csm3).
        let by = match ev.get("dispatch").and_then(|v| v.as_str()) {
            Some(b) => format!("dispatch {}", b),
            None => format!(
                "session {}",
                ev.get("session").and_then(|v| v.as_str()).unwrap_or("unbound")
            ),
        };
        let line = format!("[{}] {} by {}", op, what, by);
        if let Some(pos) = lines.iter().position(|(i, _)| i == id) {
            lines[pos].1 = line;
        } else {
            lines.push((id.to_string(), line));
        }
    }
    record_area_read_at(store, reader, area_id, log.len());
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

/// Seconds since a session's heartbeat; None when it was never seen.
pub fn last_seen_age_secs(store: &Store, session: &str) -> Option<i64> {
    use time::format_description::well_known::Rfc3339;
    let ts = last_seen(store, session)?;
    let t = time::OffsetDateTime::parse(&ts, &Rfc3339).ok()?;
    Some((time::OffsetDateTime::now_utc() - t).whole_seconds())
}

// ── fire-time liveness (it-hapc, dc-crea) ──────────────────────────────────
//
// Routing is a PULL, never a send: at q dispatch from a non-dispatch
// session, derive the dispatch-kind sessions with appropriate coverage for
// the item (kind + purview fit + the last_seen heartbeat). Any live — the
// LEAVE offer: do not fire, the item stays where it is (ready IS the
// dispatcher feed) and the live session(s) are named; there is never a
// choice among live dispatchers — the first to claim dispatches it, and the
// claim point guards the race (dc-qyr5). None awake — the WAKE offer:
// enumerate the registered candidates with their launchers; one candidate
// is offered directly, several defer to the user. Advisory and stateless
// throughout: no routed-waiting state, and fire solo stays legitimate
// (--solo). A surface, never a gate.

/// A dispatch-kind session counts as LIVE when its heartbeat is fresher
/// than this. The heartbeat is write-verb granularity (log_event touches
/// it), so an open-but-quiet dispatcher can read asleep — the window leans
/// forgiving, and every surface prints the age so the user's eyes overrule
/// the threshold. Advisory only: nothing gates on it.
pub const DISPATCH_LIVE_SECS: i64 = 15 * 60;

/// A registered dispatch-kind session judged against an item at fire time.
#[derive(Clone, Debug)]
pub struct DispatchCoverage {
    pub name: String,
    pub charter: Option<String>,
    /// Seconds since the heartbeat; None = never seen on this machine.
    pub age_secs: Option<i64>,
    pub live: bool,
}

/// The fire-time routing judgment (it-hapc): what q dispatch surfaces
/// before it fires. Derivation only — the caller renders and the user
/// chooses; nothing here writes state or denies anything.
pub enum FireRouting {
    /// No routing surface: fire. The dispatching session is itself
    /// dispatch-kind, the item is a continuation of a live dispatch
    /// (re-dispatch, steal — their own surfaces apply), or no registered
    /// dispatch-kind session covers the item.
    Fire,
    /// Live dispatcher(s) cover the item — the leave offer. Every live
    /// covering session, unranked: the first to claim dispatches it.
    Leave(Vec<DispatchCoverage>),
    /// Dispatchers are registered for this item but none is awake — the
    /// wake offer, every candidate enumerated (name order).
    Wake(Vec<DispatchCoverage>),
}

pub fn fire_routing(store: &Store, item: &Node, session: &str) -> FireRouting {
    let reg = load_sessions(store);
    // A dispatch-kind session IS the dispatcher: it never routes to itself.
    if reg
        .get(session)
        .and_then(|p| p.kind.as_deref())
        .map_or(false, |k| k == "dispatch")
    {
        return FireRouting::Fire;
    }
    // A live dispatch on the item is a continuation flow — same-chat
    // re-dispatch and --steal carry their own surfaces (dc-qyr5).
    if dispatch_for_item(store, &item.front.id).is_some() {
        return FireRouting::Fire;
    }
    let item_areas: Vec<&str> = item
        .front
        .edges
        .iter()
        .filter(|e| e.rel == "about")
        .map(|e| e.to.as_str())
        .collect();
    // Purview fit: every about-area of the item inside the session's
    // purview. An area-less item fits any dispatcher vacuously — there is
    // no basis to exclude, and the quarry dispatcher charter is all-areas
    // (dc-wngq).
    let covering: Vec<DispatchCoverage> = reg
        .iter()
        .filter(|(name, p)| {
            name.as_str() != session && p.kind.as_deref() == Some("dispatch")
        })
        .filter(|(_, p)| item_areas.iter().all(|a| p.areas.iter().any(|pa| pa == a)))
        .map(|(name, p)| {
            let age_secs = last_seen_age_secs(store, name);
            DispatchCoverage {
                name: name.clone(),
                charter: p.charter.clone(),
                age_secs,
                live: age_secs.map_or(false, |a| a < DISPATCH_LIVE_SECS),
            }
        })
        .collect();
    if covering.is_empty() {
        return FireRouting::Fire;
    }
    let live: Vec<DispatchCoverage> = covering.iter().filter(|c| c.live).cloned().collect();
    if !live.is_empty() {
        FireRouting::Leave(live)
    } else {
        FireRouting::Wake(covering)
    }
}

/// The wake road for a registered session: the launcher script when one
/// exists at the repo root (`<name>-session.cmd`, written by q session set
/// --launcher), the inline launch command otherwise — the same two roads
/// q session set advertises.
pub fn wake_command(store: &Store, session: &str) -> String {
    let launcher = store.root.join(format!("{}-session.cmd", session));
    if launcher.exists() {
        launcher.display().to_string()
    } else {
        format!("cmd /c \"set QUARRY_SESSION={} && claude\"", session)
    }
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

/// How far back from EOF a transcript is read when deriving the live model
/// (it-j4tx). Sized by measurement, never by taste: across the 23 real
/// transcripts on this machine the last assistant entry sat at most 32,820
/// bytes from EOF, and the largest single line anywhere in them was 144,652
/// bytes — half a megabyte clears their sum three times over. A miss is not a
/// failure: the caller keeps the recorded row.
pub const TRANSCRIPT_TAIL_BYTES: u64 = 512 * 1024;

/// The pure core of the model refresh (it-j4tx): the model named by the LAST
/// assistant entry in a slice of transcript tail, or None when the tail holds
/// no readable one.
///
/// `truncated` says the slice began mid-file, in which case its first line is
/// a fragment and is dropped — a half-line of JSON parses to nothing anyway,
/// but dropping it keeps the scan's meaning exact rather than accidental.
///
/// The transcript format is undocumented and harness-internal, so every
/// judgment here is a filter rather than an assumption: an entry must be
/// `type: assistant` and its `message.model` must be a real model name.
/// `<synthetic>` is the measured counter-example — the harness writes it for
/// assistant entries it composed itself — and any other angle-bracketed
/// placeholder falls with it. Anything unrecognised is skipped, never guessed
/// at; the walk simply continues to the entry before it.
///
/// `skip_sidechain` is the ONE judgment that depends on whose file this is,
/// which is why it is a parameter rather than a constant (it-6ekf). In a
/// CHAT's own transcript a sidechain entry is a subagent's turn written into
/// its parent's file, and adopting it would make a subagent's model the
/// chat's. In a SUBAGENT's own transcript the mark is native, not foreign:
/// measured across the 268 subagent transcripts on this machine, every one of
/// the 261 that holds a readable assistant entry carries `isSidechain: true`
/// on all of them, with not one non-sidechain entry among them — so the same
/// filter there would answer nothing, ever.
fn scan_last_assistant_model(tail: &[u8], truncated: bool, skip_sidechain: bool) -> Option<String> {
    let text = std::str::from_utf8(tail).ok()?;
    let mut lines: Vec<&str> = text.lines().collect();
    if truncated && !lines.is_empty() {
        lines.remove(0);
    }
    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v.get("type").and_then(|t| t.as_str()) != Some("assistant") {
            continue;
        }
        if skip_sidechain && v.get("isSidechain").and_then(|s| s.as_bool()) == Some(true) {
            continue;
        }
        let Some(m) = v.get("message").and_then(|m| m.get("model")).and_then(|m| m.as_str())
        else {
            continue;
        };
        let m = m.trim();
        if m.is_empty() || m.starts_with('<') {
            continue;
        }
        return Some(m.to_string());
    }
    None
}

/// The scan over a CHAT's own transcript, where a sidechain entry is a
/// subagent's turn and never the chat's model.
pub fn last_assistant_model(tail: &[u8], truncated: bool) -> Option<String> {
    scan_last_assistant_model(tail, truncated, true)
}

/// The scan over a SUBAGENT's own transcript (it-6ekf), where every entry
/// carries the sidechain mark because the whole file is one.
pub fn last_agent_model(tail: &[u8], truncated: bool) -> Option<String> {
    scan_last_assistant_model(tail, truncated, false)
}

/// The I/O half: the model currently producing a transcript's turns, read
/// backwards from the end. Best-effort by construction — a missing, unreadable
/// or unrecognisable transcript returns None and the caller keeps whatever it
/// already had. A hook must never fail a shell over this.
fn tail_model(path: &std::path::Path, skip_sidechain: bool) -> Option<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let take = len.min(TRANSCRIPT_TAIL_BYTES);
    let truncated = take < len;
    f.seek(SeekFrom::Start(len - take)).ok()?;
    let mut buf = Vec::with_capacity(take as usize);
    f.take(take).read_to_end(&mut buf).ok()?;
    scan_last_assistant_model(&buf, truncated, skip_sidechain)
}

/// A CHAT's live model, from the transcript the hook was handed.
pub fn transcript_model(path: &std::path::Path) -> Option<String> {
    tail_model(path, true)
}

/// A SUBAGENT's live model, from the subagent's own transcript (it-6ekf) —
/// the same read one directory over, with the sidechain filter relaxed
/// because there the mark is native.
pub fn agent_transcript_model(path: &std::path::Path) -> Option<String> {
    tail_model(path, false)
}

/// The chat's model as of THIS fire (it-j4tx) — the one derivation point for
/// the chat-actor row, and the only place it is read.
///
/// `record_chat_actor` is called once, from the SessionStart arm, and nothing
/// re-read it: a `/model` switch or a resume onto a different model left the
/// row naming a model that had stopped writing the session, and every node the
/// rest of the session minted filed under it. Measured on the dispatching chat
/// itself, 2026-08-21: the row said claude-fable-5 while all 400 assistant
/// entries in that chat's own transcript said claude-opus-5.
///
/// The transcript is the one channel that answers per-fire. The PreToolUse
/// payload carries no model (cl-dqt4) but it does carry `transcript_path`, and
/// the last assistant entry there names the model producing turns right now.
/// It is read tail-first, so the cost does not grow with the session.
///
/// CHAT-KEYED BY CONSTRUCTION, which is why a subagent's fire may write here.
/// Measured 2026-08-21: a PreToolUse firing inside a subagent carries the
/// PARENT CHAT's `transcript_path`, not the subagent's own (which exists, one
/// directory down, and is not what the harness hands the hook). So the value
/// derived from it is the chat's model whoever fires, and storing it under the
/// chat key can never smear a subagent's model onto its dispatcher — the
/// hazard `badge_model` refuses in the other direction. A joined subagent's
/// own model still comes from the badge stamp, which outranks this.
///
/// The row is rewritten only when the derived value differs, so a settled
/// session's fires are reads. When the transcript cannot answer, the recorded
/// row stands and attribution keeps its old SessionStart grain.
pub fn refreshed_chat_actor(
    store: &Store,
    chat_id: &str,
    transcript: Option<&str>,
) -> Option<String> {
    let recorded = chat_actor(store, chat_id);
    let Some(live) = transcript
        .map(std::path::Path::new)
        .and_then(transcript_model)
    else {
        return recorded;
    };
    if recorded.as_deref() != Some(live.as_str()) {
        record_chat_actor(store, chat_id, &live);
    }
    Some(live)
}

/// The model a JOINED SUBAGENT's work files under (it-xcvb) — the one
/// derivation point, pure over the dispatch map so the defect and the cure
/// are both measurable without a store.
///
/// Keyed on the AGENT identity alone, deliberately. An agent id is the mark
/// the harness gives only to a subagent, and a subagent is exactly the
/// population whose model no channel reports; every other identity is a real
/// chat, which fired SessionStart and therefore already has its own model in
/// the chat-actor map. Reading a chat-keyed acting row here would buy nothing
/// and would spread the badge's model onto the dispatching chat's own shells
/// in the no-agent-id fallback, where parent and subagent are
/// indistinguishable (see `record_acting`) — mis-attribution in the other
/// direction, which is the expensive one.
///
/// `live_acting_badge` does the honest work: a row pointing at a cleared
/// dispatch is residue, so a harvested arc stops overriding by construction.
pub fn badge_model(m: &DispatchMap, agent: &str) -> Option<String> {
    let badge = live_acting_badge(m, &format!("agent:{}", agent))?;
    m.held
        .get(&badge)?
        .model
        .clone()
        .filter(|s| !s.trim().is_empty())
}

/// The store-reading half of `badge_model`, already through `safe_actor` —
/// what the session hook injects as QUARRY_ACTOR for a joined agent's shells.
/// None means "nothing stamped": the caller keeps its existing resolution.
pub fn badge_actor(store: &Store, agent: Option<&str>) -> Option<String> {
    let agent = agent?;
    badge_model(&load_dispatches(store), agent).map(|m| safe_actor(&m))
}

/// Where the harness keeps its OWN record of a chat's subagents (it-6ekf) —
/// beside the chat transcript, in a directory named for the chat, one pair of
/// files per agent id:
///
/// ```text
///   <projects>/<chat>.jsonl                        ← what the hook is handed
///   <projects>/<chat>/subagents/agent-<id>.jsonl        the agent's turns
///   <projects>/<chat>/subagents/agent-<id>.meta.json    the spawn record
/// ```
///
/// The agent id is exactly what the session hook already injects as
/// QUARRY_AGENT (cl-z6gc), so nothing has to arrive in a hook payload for
/// this to be reachable — which is the whole reason the road exists, since a
/// PreToolUse firing inside a subagent carries no model of any kind (cl-dqt4)
/// and SessionStart, the one event that does, never fires for a subagent.
///
/// UNDOCUMENTED AND HARNESS-INTERNAL, stated plainly wherever it is read.
/// This is the same class of dependency cl-cv92 took on knowingly for the
/// chat transcript; taken a second time it is a real exposure, and the whole
/// derivation is therefore best-effort — every miss degrades to the caller's
/// existing answer rather than to an error.
///
/// The `.jsonl` suffix is stripped BY NAME rather than by `with_extension`,
/// which would eat the tail of any chat id that ever carries a dot.
pub fn subagent_records(
    chat_transcript: &std::path::Path,
    agent: &str,
) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    let name = chat_transcript.file_name()?.to_str()?;
    let stem = name.strip_suffix(".jsonl").unwrap_or(name);
    let dir = chat_transcript.parent()?.join(stem).join("subagents");
    Some((
        dir.join(format!("agent-{}.jsonl", agent)),
        dir.join(format!("agent-{}.meta.json", agent)),
    ))
}

/// The spawn model recorded in a subagent's metadata sidecar (it-6ekf).
///
/// This is the ALIAS the dispatcher typed — `opus`, `sonnet`, `fable` — never
/// a resolved model id, and the two are not interchangeable: measured across
/// the 268 sidecars on this machine, `opus` resolved to claude-opus-5 in 128
/// arcs and to claude-opus-4-8 in 43 of them. So no table can map one to the
/// other, and the sidecar is the FALLBACK rather than the source: it is read
/// only when the agent's own transcript cannot answer yet, which is the
/// window before the agent's first turn reaches disk.
///
/// A sidecar with no `model` key at all means the spawn named none (88 of the
/// 268). That is NOT the same as "runs the parent chat's model" — see
/// `agent_model`.
pub fn sidecar_model(path: &std::path::Path) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
    let m = v.get("model")?.as_str()?.trim();
    if m.is_empty() || m.starts_with('<') {
        return None;
    }
    Some(m.to_string())
}

/// A SUBAGENT's own model, resolved from the harness's record of it and keyed
/// on the injected agent id alone (it-6ekf) — the structural road cl-dqt4
/// could not find and dc-zbxj's "identity is structural, never discipline"
/// asks for. Nothing here depends on the dispatcher remembering `--model`.
///
/// THE AGENT'S OWN TRANSCRIPT LEADS, because it names the RESOLVED model —
/// the same value the chat road produces, so an arc's attribution reads in
/// one spelling — and because it covers models the sidecar never sees. The
/// sidecar records only what the spawn CALL named; a model set in an agent
/// type's own definition, or inherited by a nested spawn from its parent
/// agent rather than from the chat, is invisible there. Measured on the 88
/// sidecars carrying no model: 9 of them ran on a model their parent chat was
/// not running (a claude-code-guide arc on haiku under a fable chat, four
/// depth-2 Explore arcs on opus under fable chats, and four arcs whose chat
/// changed model after the spawn). So the sidecar's silence is not evidence
/// of inheritance, and gating on it would have missed one unstamped subagent
/// in ten.
///
/// THE SIDECAR IS THE PRE-FIRST-TURN FALLBACK. The transcript is written
/// incrementally and live (measured on this arc's own file, 423 KB of it
/// mid-session), but the very first PreToolUse of an arc — the `q join` shell
/// — can fire before that arc's first assistant entry is flushed, and there
/// is no earlier entry to fall back on the way a chat has. The sidecar exists
/// from the spawn, so it answers there; the cost is the coarse alias for that
/// one shell, which is a family-correct answer where the alternative is the
/// dispatcher's model, which is wrong outright.
///
/// NOTHING IS RECORDED. cl-dqt4 could not write a per-agent row because
/// nothing at the agent's seat knew the answer; now something does, and a row
/// still buys nothing — it would be empty in the one window where the read
/// is hard, and it would need a lifecycle nobody owns. Derived at every fire,
/// like `refreshed_chat_actor`, and the tail read keeps the cost flat.
///
/// None means "the harness record cannot answer": no transcript path in the
/// payload to locate the directory from, no such agent recorded, or neither
/// file readable. The caller keeps its existing resolution.
pub fn agent_model(chat_transcript: Option<&str>, agent: &str) -> Option<String> {
    let (jsonl, meta) = subagent_records(std::path::Path::new(chat_transcript?), agent)?;
    agent_transcript_model(&jsonl).or_else(|| sidecar_model(&meta))
}

/// The actor half of `agent_model`, through `safe_actor` so a sidecar alias
/// can never derive USER provenance — what the session hook injects as
/// QUARRY_ACTOR for a subagent whose badge carries no stamp. None means the
/// harness record answered nothing: the caller falls through to the chat row,
/// which is the right answer for a genuinely inheriting spawn.
pub fn agent_actor(chat_transcript: Option<&str>, agent: Option<&str>) -> Option<String> {
    agent_model(chat_transcript, agent?).map(|m| safe_actor(&m))
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
