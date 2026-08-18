use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::model::{At, Node};
use crate::store::Store;

/// A live blocker of an item or thread: a depends-on target not yet landed/resolved.
pub fn live_blockers<'a>(all: &'a [Node], n: &Node) -> Vec<&'a Node> {
    let mut out = Vec::new();
    for e in &n.front.edges {
        if e.rel != "depends-on" {
            continue;
        }
        if let Some(t) = all.iter().find(|x| x.front.id == e.to) {
            let live = match t.front.ty.as_str() {
                "item" => !matches!(t.front.status.as_str(), "done" | "dropped"),
                "thread" => t.front.status != "resolved",
                "decision" => t.front.status != "in-force",
                _ => false,
            };
            if live {
                out.push(t);
            }
        }
    }
    out
}

/// Items marked ready with nothing actually blocking them.
pub fn ready<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| n.front.ty == "item" && n.front.status == "ready")
        .filter(|n| live_blockers(all, n).is_empty())
        .collect()
}

/// Upcoming work (sketch/shaped) with whatever blocks each piece.
pub fn shaping<'a>(all: &'a [Node]) -> Vec<(&'a Node, Vec<&'a Node>)> {
    all.iter()
        .filter(|n| n.front.ty == "item" && matches!(n.front.status.as_str(), "sketch" | "shaped"))
        .map(|n| (n, live_blockers(all, n)))
        .collect()
}

/// Threads awaiting the user whose prerequisites have landed — answerable now.
pub fn queue<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    let mut q: Vec<&Node> = all
        .iter()
        .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
        .filter(|n| live_blockers(all, n).is_empty())
        .collect();
    q.sort_by(|a, b| a.front.created.cmp(&b.front.created));
    q
}

/// The acceptance-gate filter (dc-p6z4): live items with no acceptance
/// lines — DERIVED at read, never stored (the load-is-display pattern).
/// Nothing writes this state, authoring acceptance clears it by
/// construction, and coverage is total from mint whether or not an item
/// ever approached the gate. Queryable at any live status — a ready or
/// in-flight hit is a gate breach showing itself; settled items await
/// nothing. Pressure cuts at shaped: the design wake counts only the
/// shaped stratum, and sketches stay quiet (early absence is legitimate).
pub fn awaiting_acceptance<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    let rank = |s: &str| match s {
        "in-flight" => 0,
        "ready" => 1,
        "shaped" => 2,
        _ => 3,
    };
    let mut v: Vec<&Node> = all
        .iter()
        .filter(|n| n.front.ty == "item" && !n.front.archived)
        .filter(|n| !matches!(n.front.status.as_str(), "done" | "dropped"))
        .filter(|n| n.front.acceptance.is_empty())
        .collect();
    v.sort_by(|a, b| {
        rank(&a.front.status)
            .cmp(&rank(&b.front.status))
            .then_with(|| a.front.id.cmp(&b.front.id))
    });
    v
}

/// The lean prompt's enumeration (dc-ez67, dc-grrb; it-6349): body-cited
/// decisions and claims with no edge between them and the item in either
/// direction, each paired with its ready-made link command — depends-on
/// for a ruling the item stands on; supports, reversed, for a claim that
/// holds the item up (item → claim has no legal rel; the lean runs claim
/// → item, which is exactly how weight_held counts the load). Superseded,
/// refuted, and archived targets never enumerate — the prompt recommends
/// leans, and leaning on settled strata is C5's deliberate act, never a
/// recommendation. Derived at the flip to ready, rendered as a presence
/// prompt, never a gate: mention-only is often correct, and the judgment
/// is the shaper's, at the station where shaping completes.
pub fn unleaned_citations<'a>(all: &'a [Node], item: &Node) -> Vec<(&'a Node, String)> {
    crate::mention::mentions_out(all, item)
        .into_iter()
        .filter(|t| !t.front.archived)
        .filter(|t| match t.front.ty.as_str() {
            "decision" => t.front.status == "in-force",
            "claim" => !matches!(t.front.status.as_str(), "refuted" | "superseded"),
            _ => false,
        })
        .filter(|t| {
            !item.front.edges.iter().any(|e| e.to == t.front.id)
                && !t.front.edges.iter().any(|e| e.to == item.front.id)
        })
        .map(|t| {
            let cmd = if t.front.ty == "decision" {
                format!("q link {} depends-on {}", item.front.id, t.front.id)
            } else {
                format!("q link {} supports {}", t.front.id, item.front.id)
            };
            (t, cmd)
        })
        .collect()
}

/// A stale citation, carried as ATOMS (carrier-replumb, dc-nnf5): the print
/// layer can render any register from this without a starved (id, title)
/// pair baking the missing fields into the data layer.
pub struct Behind {
    pub src: crate::surface::Atom,
    pub rel: String,
    pub to: String,
    /// Node targets resolve to their atom; file targets and danglers carry
    /// None and describe themselves through `to_title`.
    pub to_atom: Option<crate::surface::Atom>,
    pub to_title: String,
    pub at: String,
    pub current: String,
    pub severity: u8, // 1 dead target · 2 dangling · 3 content changed · 4 file drifted
    pub reason: String,
    /// The classifier (dc-6gn9): sediment is ordinary drift (sev 3/4) under
    /// a reading — dated by design, collapsed to a count at every surface.
    /// Everything else — drift on living measurements, dead or dangling
    /// targets anywhere — is rot or breakage and enumerates loud.
    pub sediment: bool,
}

pub fn behind(store: &Store, all: &[Node]) -> Vec<Behind> {
    let mut out = Vec::new();
    for n in all {
        let reading_src =
            n.front.ty == "claim" && n.front.kind.as_deref() == Some("reading");
        for e in &n.front.edges {
            match &e.at {
                At::V(v) => match all.iter().find(|t| t.front.id == e.to) {
                    None => out.push(Behind {
                        src: crate::surface::atom(all, n),
                        rel: e.rel.clone(),
                        to: e.to.clone(),
                        to_atom: None,
                        to_title: "?".into(),
                        at: format!("v{}", v),
                        current: "missing".into(),
                        severity: 2,
                        reason: "dangling — target not found".into(),
                        sediment: false,
                    }),
                    Some(t) if t.front.v > *v => {
                        let dead = matches!(t.front.status.as_str(), "refuted" | "superseded");
                        out.push(Behind {
                            src: crate::surface::atom(all, n),
                            rel: e.rel.clone(),
                            to: t.front.id.clone(),
                            to_atom: Some(crate::surface::atom(all, t)),
                            to_title: crate::surface::title_raw(t).into(),
                            at: format!("v{}", v),
                            current: format!("v{}", t.front.v),
                            severity: if dead { 1 } else { 3 },
                            reason: if dead {
                                format!("target is {}", t.front.status)
                            } else {
                                "target content changed".into()
                            },
                            // A refuted or superseded source rots anywhere,
                            // reading or not (dc-6gn9).
                            sediment: reading_src && !dead,
                        });
                    }
                    _ => {}
                },
                At::Blob(b) => {
                    if let Some(f) = e.to.strip_prefix("file:") {
                        match store.blob(f) {
                            Ok(nb) if &nb != b => out.push(Behind {
                                src: crate::surface::atom(all, n),
                                rel: e.rel.clone(),
                                to: e.to.clone(),
                                to_atom: None,
                                to_title: f.into(),
                                at: b.clone(),
                                current: nb,
                                severity: 4,
                                reason: "file drifted".into(),
                                sediment: reading_src,
                            }),
                            Err(_) => out.push(Behind {
                                src: crate::surface::atom(all, n),
                                rel: e.rel.clone(),
                                to: e.to.clone(),
                                to_atom: None,
                                to_title: f.into(),
                                at: b.clone(),
                                current: "missing".into(),
                                severity: 2,
                                reason: "dangling — file not found".into(),
                                sediment: false,
                            }),
                            _ => {}
                        }
                    }
                }
            }
        }
        // A path-backed doc whose file moved on without an affirm.
        if n.front.ty == "doc" {
            if let (Some(p), Some(old)) = (&n.front.path, &n.front.blob) {
                if let Ok(nb) = store.blob(p) {
                    if &nb != old {
                        out.push(Behind {
                            src: crate::surface::atom(all, n),
                            rel: "path".into(),
                            to: format!("file:{}", p),
                            to_atom: None,
                            to_title: p.clone(),
                            at: old.clone(),
                            current: nb,
                            severity: 4,
                            reason: "registered doc drifted — affirm to bump".into(),
                            sediment: false,
                        });
                    }
                }
            }
        }
    }
    out.sort_by_key(|b| b.severity);
    out
}

/// Affirm is species-shaped by method (dc-6gn9): the ratified teaching the
/// affirm surface prints for the node under review. Test-methodized measured
/// claims (a file:tests source — the instrument) teach re-read; manual
/// measured claims teach re-run; readings teach their own sediment framing
/// (an affirm on sediment was never owed). Prompts, never gates — the verb
/// proceeds regardless.
pub fn affirm_teaching(n: &Node) -> Option<&'static str> {
    if n.front.ty != "claim" {
        return None;
    }
    match n.front.kind.as_deref() {
        Some("measured") => {
            let instrumented = n.front.edges.iter().any(|e| {
                e.rel == "source"
                    && e.to.strip_prefix("file:").map_or(false, |f| {
                        crate::store::strip_line(f)
                            .split(['/', '\\'])
                            .any(|c| c == "tests")
                    })
            });
            Some(if instrumented {
                crate::framings::AFFIRM_INSTRUMENT
            } else {
                crate::framings::AFFIRM_MANUAL
            })
        }
        Some("reading") => Some(crate::framings::first_sentence(crate::framings::READINGS)),
        _ => None,
    }
}

/// Who leans on this node: forward closure of its `supports`, plus inbound
/// `depends-on`, `source`, and `builds-on` edges, transitively. Reverse
/// builds-on is the lineage walk: a killed capability enumerates the specs
/// and decisions standing on it — the list IS the correction, never a sweep.
pub fn blast(all: &[Node], start: &str) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(start.to_string());
    let mut q: VecDeque<String> = VecDeque::new();
    q.push_back(start.to_string());
    let mut out = Vec::new();
    while let Some(cur) = q.pop_front() {
        if let Some(n) = all.iter().find(|n| n.front.id == cur) {
            for e in &n.front.edges {
                if e.rel == "supports" && seen.insert(e.to.clone()) {
                    out.push(e.to.clone());
                    q.push_back(e.to.clone());
                }
            }
        }
        for n in all {
            for e in &n.front.edges {
                if e.to == cur
                    && matches!(e.rel.as_str(), "depends-on" | "source" | "builds-on")
                    && seen.insert(n.front.id.clone())
                {
                    out.push(n.front.id.clone());
                    q.push_back(n.front.id.clone());
                }
            }
        }
    }
    out
}

/// Assistant-provenance settles/supersedes against user-provenance targets.
/// C3 denies these at write time; nonzero here means data arrived another way.
pub fn contested<'a>(all: &'a [Node]) -> Vec<(&'a Node, &'a str, &'a Node)> {
    let mut out = Vec::new();
    for n in all {
        if n.front.provenance != "assistant" {
            continue;
        }
        for e in &n.front.edges {
            if !matches!(e.rel.as_str(), "settles" | "supersedes") {
                continue;
            }
            if let Some(t) = all.iter().find(|t| t.front.id == e.to) {
                if t.front.provenance == "user" {
                    out.push((n, e.rel.as_str(), t));
                }
            }
        }
    }
    out
}

/// Nodes nothing points at, older than `days`.
pub fn idle<'a>(all: &'a [Node], days: i64) -> Vec<&'a Node> {
    let now = OffsetDateTime::now_utc();
    all.iter()
        .filter(|n| {
            !all.iter()
                .any(|m| m.front.edges.iter().any(|e| e.to == n.front.id))
        })
        .filter(|n| {
            OffsetDateTime::parse(&n.front.created, &Rfc3339)
                .map(|c| (now - c).whole_days() >= days)
                .unwrap_or(false)
        })
        .collect()
}

/// Assistant claims never verified: no method, still asserted.
pub fn unverified<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| {
            n.front.ty == "claim"
                && n.front.provenance == "assistant"
                && n.front.status == "asserted"
                && n.front.method.is_none()
        })
        .collect()
}

/// Weight held (dc-drr6): how many standing builds this claim's supports
/// edges hold up — the vein supports the feature, so the feature stands on
/// the vein. Derived at render, display only, never status. A target
/// counts when it resolves and still stands (not archived, not
/// dropped/refuted/superseded — a done item stands: it is built).
pub fn weight_held(all: &[Node], c: &Node) -> usize {
    let mut seen: Vec<&str> = Vec::new();
    for e in &c.front.edges {
        if e.rel != "supports" || seen.contains(&e.to.as_str()) {
            continue;
        }
        let standing = all.iter().any(|t| {
            t.front.id == e.to
                && !t.front.archived
                && !matches!(t.front.status.as_str(), "dropped" | "refuted" | "superseded")
        });
        if standing {
            seen.push(&e.to);
        }
    }
    seen.len()
}

/// Load-bearing but never assayed (dc-drr6): claims on the
/// asserted-to-ratified ladder that builds stand on while no judge has
/// verified them — fool's gold risk rises with weight, so the heaviest
/// lead. Measured claims live by their instrument, readings settle as
/// sediment, and contracts keep their normative ladder — none of those
/// belong here.
pub fn load_bearing_unassayed<'a>(all: &'a [Node]) -> Vec<(&'a Node, usize)> {
    let mut out: Vec<(&Node, usize)> = all
        .iter()
        .filter(|n| {
            n.front.ty == "claim"
                && !n.front.archived
                && n.front.status == "asserted"
                && !matches!(
                    n.front.kind.as_deref(),
                    Some("measured") | Some("reading") | Some("contract")
                )
        })
        .map(|n| (n, weight_held(all, n)))
        .filter(|(_, w)| *w > 0)
        .collect();
    out.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| crate::surface::title_raw(a.0).cmp(crate::surface::title_raw(b.0)))
    });
    out
}

/// Claim ids minted under an item's dispatch badge — read off the log's
/// badge stamps, so the trace survives release and clear_dispatch.
pub fn badge_claim_mints(log: &[serde_json::Value], item_id: &str) -> Vec<String> {
    log.iter()
        .filter(|ev| ev.get("dispatch").and_then(|v| v.as_str()) == Some(item_id))
        .filter(|ev| {
            ev.get("op").and_then(|v| v.as_str()) == Some("create")
                && ev.get("type").and_then(|v| v.as_str()) == Some("claim")
        })
        .filter_map(|ev| ev.get("node").and_then(|v| v.as_str()).map(String::from))
        .collect()
}

/// Claim ids a session minted from `since` on — the solo arc's own mints
/// (dc-drr6: a session landing its own work ratifies its own mints).
/// RFC3339 UTC stamps compare lexicographically; `since` is inclusive.
pub fn session_claim_mints(log: &[serde_json::Value], session: &str, since: &str) -> Vec<String> {
    log.iter()
        .filter(|ev| ev.get("session").and_then(|v| v.as_str()) == Some(session))
        .filter(|ev| {
            ev.get("op").and_then(|v| v.as_str()) == Some("create")
                && ev.get("type").and_then(|v| v.as_str()) == Some("claim")
        })
        .filter(|ev| ev.get("ts").and_then(|v| v.as_str()).map_or(false, |ts| ts >= since))
        .filter_map(|ev| ev.get("node").and_then(|v| v.as_str()).map(String::from))
        .collect()
}

/// Whether an item ever flew as a dispatch: a dispatch event on the item,
/// or any act stamped with its badge. Splits the landing's assay paths —
/// a dispatched arc ratifies its badge mints; only a never-dispatched arc
/// self-ratifies as solo (dc-drr6).
pub fn item_was_dispatched(log: &[serde_json::Value], item_id: &str) -> bool {
    log.iter().any(|ev| {
        (ev.get("op").and_then(|v| v.as_str()) == Some("dispatch")
            && ev.get("node").and_then(|v| v.as_str()) == Some(item_id))
            || ev.get("dispatch").and_then(|v| v.as_str()) == Some(item_id)
    })
}

/// The claims a landing may assay (dc-drr6): of the arc's mints, the vein
/// and feature receipts still on the asserted rung. The ladder is
/// species-shaped — measured, readings, contracts, and kindless mints
/// never ratify here.
pub fn assayable<'a>(all: &'a [Node], mint_ids: &[String]) -> Vec<&'a Node> {
    let mut out: Vec<&Node> = Vec::new();
    for id in mint_ids {
        let Some(n) = all.iter().find(|n| &n.front.id == id) else { continue };
        if n.front.ty == "claim"
            && !n.front.archived
            && n.front.status == "asserted"
            && matches!(n.front.kind.as_deref(), Some("vein") | Some("feature"))
            && !out.iter().any(|m| m.front.id == n.front.id)
        {
            out.push(n);
        }
    }
    out
}

/// The kindless backtick-titled mints of an arc (it-pgn9): claims still on
/// the asserted rung, no species recorded, a registered name leading the
/// title — the shape that minted cl-ybth kindless and was silently repaired
/// at harvest. The assay ladder never reaches these (`assayable` is
/// species-shaped), so the harvest ask lists them with the settle command
/// before ratification passes them by — a prompt, never a gate (dc-grrb).
pub fn kindless_backtick_mints<'a>(all: &'a [Node], mint_ids: &[String]) -> Vec<&'a Node> {
    let mut out: Vec<&Node> = Vec::new();
    for id in mint_ids {
        let Some(n) = all.iter().find(|n| &n.front.id == id) else { continue };
        if n.front.ty == "claim"
            && !n.front.archived
            && n.front.status == "asserted"
            && n.front.kind.is_none()
            && backtick_titled(crate::surface::title_raw(n))
            && !out.iter().any(|m| m.front.id == n.front.id)
        {
            out.push(n);
        }
    }
    out
}

/// Graph-generic vocabulary excluded from relatedness matching: on any
/// quarry graph these words appear everywhere and carry no subject signal.
const GENERIC_TOKENS: &[&str] = &[
    "quarry", "graph", "session", "sessions", "node", "nodes", "area", "areas",
    "item", "items", "thread", "threads", "claim", "claims", "decision",
    "decisions", "doc", "docs", "verb", "verbs", "surface", "status", "user",
    "assistant", "title", "titles", "about", "content", "record", "records",
];

/// Distinctive tokens of a title: length ≥ 5, generic graph vocabulary
/// dropped. Hyphen compounds are kept whole AND emit their halves of
/// five-plus chars (it-sc2u, user-ruled 2026-08-15): the name stays an
/// atom — deep-time survives — while spelling variance (core sample) and
/// bare fragments (sample) join; `relatedness` scores compound hits above
/// fragment hits, so fragment noise stays behind the score gate.
fn sig_tokens(title: &str) -> Vec<String> {
    fn push(out: &mut Vec<String>, t: &str) {
        if t.len() >= 5 && !GENERIC_TOKENS.contains(&t) && !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    }
    let lower = title.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    for raw in lower.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        let t = raw.trim_matches('-');
        push(&mut out, t);
        if t.contains('-') {
            for half in t.split('-') {
                push(&mut out, half);
            }
        }
    }
    out
}

/// Backticked spans of a text, trimmed and lowercased — the naming
/// register's marks (`name`). One extraction shared by the relatedness
/// matcher and the intent delta: plan and reality join on one vocabulary.
/// The deliberate-name floor (dc-qvtz, user-ruled 2026-08-15): backticked
/// joins from TWO characters — floors are the noise gate, backticks are
/// the deliberateness gate, so a short name someone chose to register
/// (`cli`) deserves to join while bare short prose stays below the
/// bare-token floors.
pub fn backticked_spans(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() >= 2 && s.len() <= 60)
        .collect()
}

/// A backtick-name-first title: a registered name leads (`name`: ...) —
/// the register vein and receipt claims are titled in (dc-wrnc, dc-yd9s).
/// The leading span honors the deliberate-name floor above (dc-qvtz: two
/// characters, sixty the cap). One predicate point shared by the mint
/// prompt and the harvest ask (it-pgn9): a kindless claim in this shape
/// draws the vein species question — a prompt, never a gate (dc-grrb).
pub fn backtick_titled(title: &str) -> bool {
    let Some(rest) = title.trim_start().strip_prefix('`') else {
        return false;
    };
    match rest.find('`') {
        Some(i) => (2..=60).contains(&rest[..i].trim().len()),
        None => false,
    }
}

/// The lexicon join's word predicate (relatedness): `word` occurs in
/// `text` not embedded in a longer token ("wrap" must not hit "wrapper").
/// Boundary rule: word chars are ASCII alphanumerics only — the hyphen is
/// a boundary, find's edge rule adopted 2026-08-15 (it-sc2u), so "sample"
/// hits inside "core-sample" and spelling variance joins. Names stay
/// atoms not through this predicate but through emission and scoring:
/// `sig_tokens` emits compounds whole plus their halves, and
/// `relatedness` scores compound hits above fragment hits.
/// Compare-time plural fold (it-nuw5, lexicon side ONLY): s/es folds in
/// both directions — watches meets watch, leases meets lease. This fold
/// is the remaining deliberate divergence from `find_word` below, find's
/// predicate, which stays exact for short human queries.
pub(crate) fn contains_word(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    if find_word(text, word) {
        return true;
    }
    // The plural fold: try the word's own s/es variants, so either side
    // of the compare may carry the inflection.
    let mut variants: Vec<String> = vec![format!("{word}s"), format!("{word}es")];
    if let Some(stem) = word.strip_suffix("es") {
        if stem.len() >= 3 {
            variants.push(stem.to_string());
        }
    }
    if let Some(stem) = word.strip_suffix('s') {
        if stem.len() >= 3 {
            variants.push(stem.to_string());
        }
    }
    variants.iter().any(|v| find_word(text, v))
}

/// Find's word predicate. Its rule: word chars are ASCII alphanumerics
/// ONLY — everything else is a boundary, hyphens included, so a short
/// query like "cli" hits "cli-area" and "the cli" but never "click".
/// The lexicon join's `contains_word` above adopted this edge rule
/// 2026-08-15 (it-sc2u); the remaining deliberate divergence is the
/// plural fold — the lexicon folds s/es at compare time (it-nuw5), find
/// stays exact. Full unification considered and declined 2026-08-12
/// (it-hjed).
pub fn find_word(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let i = start + pos;
        let before_ok =
            text[..i].chars().last().map_or(true, |c| !c.is_ascii_alphanumeric());
        let j = i + word.len();
        let after_ok =
            text[j..].chars().next().map_or(true, |c| !c.is_ascii_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        start = j;
    }
    false
}

/// find's tiered matches (it-hjed). Tier one is id substring plus title
/// and body word-boundary hits (`find_word`); substring-only hits are the
/// loose tail — always shown, never hidden, labeled at render. The query
/// arrives lowercased by the caller.
pub struct FindHits<'a> {
    /// Tier one, unlabeled: the query is a substring of the id or a
    /// word-boundary hit in the title.
    pub strong: Vec<&'a Node>,
    /// Tier one, body word-boundary only — keeps the matched-in-body label.
    pub body: Vec<&'a Node>,
    /// Substring-only hits in title or body: the labeled loose tail.
    pub loose: Vec<&'a Node>,
}

/// Tier every node against a lowercased find query. Graph order is kept
/// within each tier; rendering order (strong, body, loose) is the caller's.
pub fn find_hits<'a>(all: &'a [Node], q: &str) -> FindHits<'a> {
    let mut hits = FindHits { strong: Vec::new(), body: Vec::new(), loose: Vec::new() };
    for n in all {
        let title = crate::surface::title_raw(n).to_lowercase();
        let body = n.body.to_lowercase();
        if n.front.id.contains(q) || find_word(&title, q) {
            hits.strong.push(n);
        } else if find_word(&body, q) {
            hits.body.push(n);
        } else if title.contains(q) || body.contains(q) {
            hits.loose.push(n);
        }
    }
    hits
}

/// Mint-time relatedness: an index of candidates the new node's text touches,
/// for the minting agent to review — never auto-linked. Forward: the existing
/// title lexicon matched against the new node's title+body (backticked spans
/// strengthen). Reverse: the new title's distinctive tokens matched against
/// existing bodies — prior mentions of a concept that just earned its node.
/// Archived nodes, areas, and already-linked neighbors are excluded; the
/// strongest few qualify (silence is the default).
pub fn relatedness<'a>(all: &'a [Node], node: &Node) -> Vec<(&'a Node, String)> {
    let new_text =
        format!("{} {}", crate::surface::title_raw(node), node.body).to_lowercase();
    let new_title_toks: Vec<String> = sig_tokens(crate::surface::title_raw(node))
        .into_iter()
        .filter(|t| t.len() >= 6)
        .collect();
    let backticked: Vec<String> = backticked_spans(&node.body);
    let mut scored: Vec<(i32, &Node, String)> = Vec::new();
    for cand in all {
        if cand.front.id == node.front.id || cand.front.archived || cand.front.ty == "area" {
            continue;
        }
        if node.front.edges.iter().any(|e| e.to == cand.front.id)
            || cand.front.edges.iter().any(|e| e.to == node.front.id)
        {
            continue;
        }
        let title_lower = crate::surface::title_raw(cand).to_lowercase();
        let toks = sig_tokens(crate::surface::title_raw(cand));
        let hits: Vec<&String> = toks.iter().filter(|t| contains_word(&new_text, t)).collect();
        let full_title = title_lower.len() >= 8 && new_text.contains(&title_lower);
        // Word-boundary, not substring: with the two-char deliberate-name
        // floor (dc-qvtz) a raw contains would let `cli` hit "click" —
        // the debris species it-hjed evicted from find.
        let tick = backticked
            .iter()
            .any(|b| find_word(&title_lower, b) || toks.iter().any(|t| t == b));
        if full_title || hits.len() >= 2 || hits.iter().any(|t| t.len() >= 6) || tick {
            // Compound hits outrank fragment hits (it-sc2u): a hit on the
            // whole hyphenated name weighs double a hit on a bare half.
            let weight: i32 = hits.iter().map(|t| if t.contains('-') { 2 } else { 1 }).sum();
            let why = if full_title {
                "mentions its title".to_string()
            } else if let Some(t) = hits.iter().find(|t| t.contains('-')).or(hits.first()) {
                format!("mentions '{}'", t)
            } else {
                "backtick reference".to_string()
            };
            let score = weight + if full_title { 2 } else { 0 } + if tick { 2 } else { 0 };
            scored.push((score, cand, why));
            continue;
        }
        if !cand.body.is_empty() {
            let cbody = cand.body.to_lowercase();
            let rhits: Vec<&String> = new_title_toks
                .iter()
                .filter(|t| contains_word(&cbody, t))
                .collect();
            if !rhits.is_empty() {
                let weight: i32 =
                    rhits.iter().map(|t| if t.contains('-') { 2 } else { 1 }).sum();
                let named = rhits.iter().find(|t| t.contains('-')).unwrap_or(&rhits[0]);
                scored.push((weight, cand, format!("its body mentions '{}'", named)));
            }
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.front.id.cmp(&b.1.front.id)));
    scored.truncate(4);
    scored.into_iter().map(|(_, n, w)| (n, w)).collect()
}

/// The intent delta, both directions. Never a sweep — an index for judgment.
pub struct IntentDelta<'a> {
    /// Named in a live item's acceptance, no live vein claim in a shared
    /// area carries it — intended but unlanded.
    pub unlanded: Vec<(String, &'a Node)>,
    /// A live vein claim's registered name that no item's acceptance ever
    /// named — landed but unintended: emergent scope, visible not silent.
    pub unintended: Vec<(String, &'a Node)>,
}

/// Derive the intent delta (the intent ruling, 2026-08-10): acceptance
/// lines name their intended capabilities in the vein register (lands
/// `name`: what it provides); vein claim titles carry the names that
/// exist. Same vocabulary, so the comparison derives — joined per shared
/// area, scoped by `area` when given. Unlanded intent is read off LIVE
/// items only (settled intent is no longer owed); the unintended check
/// consults items of ANY status — landing an item does not un-intend what
/// its acceptance named.
pub fn intent_delta<'a>(all: &'a [Node], area: Option<&str>) -> IntentDelta<'a> {
    let areas_of = |n: &Node| -> Vec<String> {
        n.front
            .edges
            .iter()
            .filter(|e| e.rel == "about")
            .filter(|e| all.iter().any(|a| a.front.id == e.to && a.front.ty == "area"))
            .map(|e| e.to.clone())
            .collect()
    };
    let in_scope = |areas: &[String]| area.map_or(true, |a| areas.iter().any(|x| x == a));
    let shares = |a: &[String], b: &[String]| a.iter().any(|x| b.contains(x));
    // Live vein claims carrying registered names in their titles.
    let veins: Vec<(&Node, Vec<String>, Vec<String>)> = all
        .iter()
        .filter(|n| {
            n.front.ty == "claim" && !matches!(n.front.status.as_str(), "refuted" | "superseded")
        })
        .map(|n| (n, backticked_spans(crate::surface::title_raw(n)), areas_of(n)))
        .filter(|(_, names, areas)| !names.is_empty() && !areas.is_empty())
        .collect();
    // Intent: backtick-named acceptance lines, per item (any status), with areas.
    let intents: Vec<(&Node, Vec<String>, Vec<String>)> = all
        .iter()
        .filter(|n| n.front.ty == "item")
        .map(|n| {
            let names: Vec<String> = n
                .front
                .acceptance
                .iter()
                .flat_map(|a| backticked_spans(a))
                .collect();
            (n, names, areas_of(n))
        })
        .filter(|(_, names, areas)| !names.is_empty() && !areas.is_empty())
        .collect();
    let mut unlanded: Vec<(String, &Node)> = Vec::new();
    for (item, names, iareas) in intents
        .iter()
        .filter(|(n, _, _)| !matches!(n.front.status.as_str(), "done" | "dropped") && !n.front.archived)
    {
        if !in_scope(iareas) {
            continue;
        }
        for name in names {
            let landed = veins
                .iter()
                .any(|(_, snames, sareas)| snames.contains(name) && shares(iareas, sareas));
            if !landed && !unlanded.iter().any(|(x, i)| x == name && i.front.id == item.front.id) {
                unlanded.push((name.clone(), item));
            }
        }
    }
    let mut unintended: Vec<(String, &Node)> = Vec::new();
    for (claim, names, careas) in &veins {
        if !in_scope(careas) {
            continue;
        }
        for name in names {
            let intended = intents
                .iter()
                .any(|(_, inames, iareas)| inames.contains(name) && shares(careas, iareas));
            if !intended && !unintended.iter().any(|(x, c)| x == name && c.front.id == claim.front.id) {
                unintended.push((name.clone(), claim));
            }
        }
    }
    IntentDelta { unlanded, unintended }
}

/// The shelf match (dc-xfgz, dc-hjad): how strongly a backdrop candidate
/// bears on the dispatched work. Distinct shared terms count, occurrences
/// never; no term damping — in this register the common words are the
/// ontology. The adjacency bonus applies only with at least one lexical
/// match; a shared backticked capability name is the automatic-full-body
/// signal, tier machinery aside.
pub struct ShelfMatch {
    /// Distinct shared terms plus the adjacency bonus — the sort key
    /// (weight descending, then alphabetical by title).
    pub weight: i32,
    /// The matched terms themselves (lowercased) — the truncated register
    /// renders exactly the body lines carrying one of these.
    pub terms: Vec<String>,
    /// A backticked capability name shared by both sides: automatic full
    /// body (dc-xfgz).
    pub capability: bool,
}

/// The work item's side of the shelf match, resolved once per brief:
/// its match text (title + body + acceptance, lowercased), its term set
/// (title lexicon plus backticked names — the trio-repaired join: plural
/// fold, compound halves, two-char backtick floor all apply), and the
/// adjacency set (dc-hjad): every node one cared-about edge — depends-on,
/// builds-on, supports — from the item, either direction. about is
/// excluded (area membership is the backdrop's baseline); part-of is
/// excluded (hierarchy is th-zzqv's business).
pub struct ShelfCtx {
    pub item_id: String,
    pub text: String,
    pub terms: Vec<String>,
    pub ticks: Vec<String>,
    pub adjacent: HashSet<String>,
}

fn cared_rel(rel: &str) -> bool {
    matches!(rel, "depends-on" | "builds-on" | "supports")
}

/// Match text of a node: title, body, and acceptance capability names —
/// the three surfaces dc-xfgz says matching reads, on both sides.
fn match_text(n: &Node) -> String {
    format!(
        "{}\n{}\n{}",
        crate::surface::title_raw(n),
        n.body,
        n.front.acceptance.join("\n")
    )
    .to_lowercase()
}

/// A node's term set: the title lexicon (compounds whole plus their
/// five-plus halves, it-sc2u) plus every backticked name in its match
/// text (two-char floor, dc-qvtz) — deliberate names join from either
/// side's body or acceptance, not only titles.
fn shelf_terms(n: &Node) -> (Vec<String>, Vec<String>) {
    let ticks = backticked_spans(&match_text(n));
    let mut terms = sig_tokens(crate::surface::title_raw(n));
    for t in &ticks {
        if !terms.iter().any(|x| x == t) {
            terms.push(t.clone());
        }
    }
    (terms, ticks)
}

pub fn shelf_ctx(all: &[Node], item: &Node) -> ShelfCtx {
    let mut adjacent: HashSet<String> = HashSet::new();
    for e in &item.front.edges {
        if cared_rel(&e.rel) && !e.to.starts_with("file:") {
            adjacent.insert(e.to.clone());
        }
    }
    for n in all {
        if n.front.edges.iter().any(|e| cared_rel(&e.rel) && e.to == item.front.id) {
            adjacent.insert(n.front.id.clone());
        }
    }
    let (terms, ticks) = shelf_terms(item);
    ShelfCtx {
        item_id: item.front.id.clone(),
        text: match_text(item),
        terms,
        ticks,
        adjacent,
    }
}

/// Score one backdrop candidate against the work (dc-xfgz tiers,
/// dc-hjad adjacency). Terms match through `contains_word` — the
/// trio-repaired lexicon join — in both directions; the bonus is +1 per
/// cared-about edge between the candidate and the item or its adjacency
/// set, and applies only when at least one term matched.
pub fn shelf_match(all: &[Node], ctx: &ShelfCtx, cand: &Node) -> ShelfMatch {
    let cand_text = match_text(cand);
    let (cand_terms, cand_ticks) = shelf_terms(cand);
    let mut terms: Vec<String> = Vec::new();
    for t in &cand_terms {
        if contains_word(&ctx.text, t) && !terms.iter().any(|x| x == t) {
            terms.push(t.clone());
        }
    }
    for t in &ctx.terms {
        if contains_word(&cand_text, t) && !terms.iter().any(|x| x == t) {
            terms.push(t.clone());
        }
    }
    let capability = ctx.ticks.iter().any(|t| cand_ticks.contains(t));
    let mut weight = terms.len() as i32;
    if weight > 0 {
        let leans_on = |to: &str| to == ctx.item_id || ctx.adjacent.contains(to);
        let mut bonus = cand
            .front
            .edges
            .iter()
            .filter(|e| cared_rel(&e.rel) && leans_on(&e.to))
            .count();
        for n in all {
            if n.front.id != cand.front.id
                && (n.front.id == ctx.item_id || ctx.adjacent.contains(&n.front.id))
            {
                bonus += n
                    .front
                    .edges
                    .iter()
                    .filter(|e| cared_rel(&e.rel) && e.to == cand.front.id)
                    .count();
            }
        }
        weight += bonus as i32;
    }
    ShelfMatch { weight, terms, capability }
}

/// Citers of `id` whose stamp is now behind the target's version — the
/// staleness homework a bump creates.
pub fn citers_behind<'a>(all: &'a [Node], id: &str) -> Vec<(&'a Node, &'a crate::model::Edge)> {
    let Some(target) = all.iter().find(|n| n.front.id == id) else {
        return vec![];
    };
    let mut out = Vec::new();
    for n in all {
        for e in &n.front.edges {
            if e.to == id {
                if let At::V(v) = &e.at {
                    if target.front.v > *v {
                        out.push((n, e));
                    }
                }
            }
        }
    }
    out
}

/// Nodes whose blockage just cleared because `id` reached a satisfied state —
/// the flow homework a resolution creates. Empty if `id` still blocks.
pub fn unblocked_by<'a>(all: &'a [Node], id: &str) -> Vec<&'a Node> {
    let Some(target) = all.iter().find(|n| n.front.id == id) else {
        return vec![];
    };
    let satisfied = match target.front.ty.as_str() {
        "item" => matches!(target.front.status.as_str(), "done" | "dropped"),
        "thread" => target.front.status == "resolved",
        "decision" => target.front.status == "in-force",
        _ => false,
    };
    if !satisfied {
        return vec![];
    }
    all.iter()
        .filter(|n| n.front.edges.iter().any(|e| e.rel == "depends-on" && e.to == id))
        .filter(|n| live_blockers(all, n).is_empty())
        .collect()
}

/// Does any claim or doc cite files under these globs? The land-time
/// landmark check is PRESENCE of citation, never quality — quality is
/// judged at review, and a gate here would breed Goodhart claims.
pub fn files_cited(all: &[Node], globs: &[String]) -> bool {
    all.iter().any(|n| {
        (matches!(n.front.ty.as_str(), "claim" | "doc")
            && n.front.edges.iter().any(|e| {
                e.to.strip_prefix("file:").map_or(false, |f| {
                    let path = crate::store::strip_line(f);
                    globs.iter().any(|g| crate::coord::globs_overlap(g, &path))
                })
            }))
            || (n.front.ty == "doc"
                && n.front
                    .path
                    .as_ref()
                    .map_or(false, |p| globs.iter().any(|g| crate::coord::globs_overlap(g, p))))
    })
}

/// Live items whose declared write-sets or file edges cover any of these
/// paths — the derived "which item is this arc?" match behind the leaseless
/// threshold nudge. A match is a resemblance to review, never an auto-claim.
pub fn items_matching_files<'a>(all: &'a [Node], files: &[String]) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| {
            n.front.ty == "item"
                && !n.front.archived
                && !matches!(n.front.status.as_str(), "done" | "dropped")
        })
        .filter(|n| {
            n.front
                .write_set
                .iter()
                .any(|g| files.iter().any(|f| crate::coord::globs_overlap(g, f)))
                || n.front.edges.iter().any(|e| {
                    e.to.strip_prefix("file:").map_or(false, |fr| {
                        let p = crate::store::strip_line(fr);
                        files.iter().any(|f| crate::coord::globs_overlap(&p, f))
                    })
                })
        })
        .collect()
}

/// Dispatched items whose report was never harvested: a dispatch event with
/// no later harvest for the same item, the item still live. The report is
/// owed — wrap confronts the dispatcher.
pub fn unharvested_dispatches<'a>(
    all: &'a [Node],
    log: &[serde_json::Value],
) -> Vec<&'a Node> {
    let mut last_dispatch: Vec<(String, usize)> = Vec::new();
    let mut last_harvest: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, ev) in log.iter().enumerate() {
        let Some(id) = ev.get("node").and_then(|v| v.as_str()) else { continue };
        match ev.get("op").and_then(|v| v.as_str()) {
            Some("dispatch") => match last_dispatch.iter_mut().find(|(d, _)| d == id) {
                Some(e) => e.1 = i,
                None => last_dispatch.push((id.to_string(), i)),
            },
            Some("harvest") => {
                last_harvest.insert(id.to_string(), i);
            }
            _ => {}
        }
    }
    last_dispatch
        .into_iter()
        .filter(|(id, di)| last_harvest.get(id).map_or(true, |hi| hi < di))
        .filter_map(|(id, _)| all.iter().find(|n| n.front.id == id))
        .filter(|n| !matches!(n.front.status.as_str(), "done" | "dropped"))
        .collect()
}

/// Nodes this session created or adjusted since its last wrap, with each
/// node's most recent op — the boundary-time final-review list (the wrap
/// event itself plants the cursor). `sess = None` means an unbound chat:
/// events carrying no session stamp.
pub fn session_touched(
    log: &[serde_json::Value],
    sess: Option<&str>,
) -> Vec<(String, String)> {
    let matches_key = |ev: &serde_json::Value| -> bool {
        match (sess, ev.get("session").and_then(|v| v.as_str())) {
            (Some(s), Some(es)) => s == es,
            (None, None) => true,
            _ => false,
        }
    };
    let cursor = log
        .iter()
        .rev()
        .find(|ev| ev.get("op").and_then(|v| v.as_str()) == Some("wrap") && matches_key(ev))
        .and_then(|ev| ev.get("ts").and_then(|v| v.as_str()))
        .unwrap_or("");
    let mut touched: Vec<(String, String)> = Vec::new();
    for ev in log {
        let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        if ts <= cursor || !matches_key(ev) {
            continue;
        }
        let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
        if op == "wrap" {
            continue;
        }
        if let Some(id) = ev.get("node").and_then(|v| v.as_str()) {
            if let Some(pos) = touched.iter().position(|(i, _)| i == id) {
                touched[pos].1 = op.to_string();
            } else {
                touched.push((id.to_string(), op.to_string()));
            }
        }
    }
    touched
}

/// Events for one node, oldest first.
pub fn node_log(store: &Store, id: &str) -> Result<Vec<serde_json::Value>> {
    Ok(store
        .read_log()?
        .into_iter()
        .filter(|e| e.get("node").and_then(|v| v.as_str()) == Some(id))
        .collect())
}
