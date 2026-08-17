//! The surfacing atom (dc-nnf5): one struct, three renderers, one module,
//! lint-enforced. Every register renders nodes through the Atom; no node
//! title is formatted anywhere else. The invariant is held by construction:
//! `front.title` outside this file fails the lint (tests/surface_lint.rs,
//! and `q wrap` re-checks it when run inside quarry's own repo). Even
//! non-render access (matching, carriers, the retitle act) rides the raw
//! accessors here, so the lint stays a total grep — no judgment calls about
//! which reference is "formatting".

use crate::model::Node;

/// Everything a surface may say about a node, resolved once: areas as
/// titles (ids kept beside them for the home/foreign comparison), grounding
/// presence for claims, the archived flag. Registers differ only in how
/// much of the atom they render, never in where they are built.
#[derive(Clone, Debug)]
pub struct Atom {
    pub id: String,
    pub ty: String,
    pub kind: Option<String>,
    pub title: String,
    pub status: String,
    pub v: u64,
    /// Titles of the areas this node is about, in edge order.
    pub areas: Vec<String>,
    /// The same areas by id — the stable key for foreign-mention checks.
    pub area_ids: Vec<String>,
    pub provenance: String,
    /// Claims only: source/method presence. A user-provenance claim with
    /// neither carries None (the provenance is the grounding); a non-user
    /// claim with neither says so loudly.
    pub grounding: Option<String>,
    /// Claims only: weight held — how many standing builds this claim's
    /// supports edges hold up (dc-drr6: load is display, never status).
    /// None for non-claims; Some(0) renders nothing.
    pub weight: Option<usize>,
    pub archived: bool,
}

/// Resolve a node into its atom. Area titles come from about-edges; a
/// dangling about-edge simply contributes nothing (behind reports it).
pub fn atom(all: &[Node], n: &Node) -> Atom {
    let mut areas = Vec::new();
    let mut area_ids = Vec::new();
    for e in &n.front.edges {
        if e.rel != "about" {
            continue;
        }
        if let Some(a) = all.iter().find(|t| t.front.id == e.to && t.front.ty == "area") {
            area_ids.push(a.front.id.clone());
            areas.push(a.front.title.clone());
        }
    }
    let grounding = if n.front.ty == "claim" {
        let sourced = n.front.edges.iter().any(|e| e.rel == "source");
        let method = n.front.method.is_some();
        match (sourced, method) {
            (true, true) => Some("sourced·method".to_string()),
            (true, false) => Some("sourced".to_string()),
            (false, true) => Some("method".to_string()),
            (false, false) if n.front.provenance == "user" => None,
            (false, false) => Some("ungrounded".to_string()),
        }
    } else {
        None
    };
    let weight = if n.front.ty == "claim" {
        Some(crate::queries::weight_held(all, n))
    } else {
        None
    };
    Atom {
        id: n.front.id.clone(),
        ty: n.front.ty.clone(),
        kind: n.front.kind.clone(),
        title: n.front.title.clone(),
        status: n.front.status.clone(),
        v: n.front.v,
        areas,
        area_ids,
        provenance: n.front.provenance.clone(),
        grounding,
        weight,
        archived: n.front.archived,
    }
}

/// The status-slot label: type and status, dead statuses keeping their
/// comma emphasis, the archived flag riding along. One label for refs and
/// unpacks alike.
fn label(a: &Atom) -> String {
    let dead = matches!(a.status.as_str(), "refuted" | "superseded" | "dropped");
    let mut l = format!("{}{}{}", a.ty, if dead { ", " } else { " " }, a.status);
    if a.archived {
        l.push_str(", archived");
    }
    l
}

/// atom_line — the full atom on one line, for every list register (find,
/// queue, ready, shaping, touches, homework, brief shelves, hook
/// enumerations). Kind rides type; areas render as titles; archived rides
/// the status slot; grounding presence rides provenance on claims.
pub fn atom_line(a: &Atom) -> String {
    let mut s = format!("\"{}\" — {}", a.title, a.ty);
    if let Some(k) = &a.kind {
        s.push('·');
        s.push_str(k);
    }
    s.push_str(&format!(
        " [{}{}] v{}",
        a.status,
        if a.archived { ", archived" } else { "" },
        a.v
    ));
    if !a.areas.is_empty() {
        s.push_str(&format!(" · {}", a.areas.join(", ")));
    }
    s.push_str(&format!(" · {}", a.provenance));
    if let Some(g) = &a.grounding {
        s.push('·');
        s.push_str(g);
    }
    // Load display (dc-drr6): weight held renders wherever the claim does —
    // silence at zero, never a status.
    if let Some(w) = a.weight {
        if w > 0 {
            s.push_str(&format!(" · holds {}", w));
        }
    }
    s.push_str(&format!(" ({})", a.id));
    s
}

/// atom_ref — the floor for every binary-composed sentence (errors,
/// confirmations, refusals): "title" [type status] (id). Status is
/// unconditional; archived rides the status slot.
pub fn atom_ref(a: &Atom) -> String {
    format!("\"{}\" [{}] ({})", a.title, label(a), a.id)
}

/// The area announcement of a foreign mention: Some(joined titles) when the
/// cited node reaches outside the citing node's areas, None when home (or
/// when the cited node has no areas to announce).
fn foreign_areas(a: &Atom, citing_area_ids: &[String]) -> Option<String> {
    if a.area_ids.is_empty() || a.area_ids.iter().all(|id| citing_area_ids.contains(id)) {
        return None;
    }
    Some(a.areas.join(", "))
}

/// atom_unpack — the in-place expansion of a bare id in a rendered body:
/// id-anchored, current title beside it, areas announced only when the
/// cited node's areas differ from the citing node's. Foreign mentions
/// announce themselves; home stays quiet.
pub fn atom_unpack(a: &Atom, citing_area_ids: &[String]) -> String {
    let areas = foreign_areas(a, citing_area_ids)
        .map(|t| format!(" — areas: {}", t))
        .unwrap_or_default();
    format!("{} [{}: `{}`{}]", a.id, label(a), a.title, areas)
}

/// The open header — the fullest register: the atom's identity block plus
/// the node-only extras (actor, created, ratified, path, method). The body,
/// acceptance, and edge shelves compose around it at the call site.
pub fn atom_head(all: &[Node], n: &Node) -> String {
    let a = atom(all, n);
    let mut s = format!(
        "■ {}  {}{}  v{}  [{}]{}\n",
        a.id,
        a.ty,
        a.kind.as_deref().map(|k| format!("·{}", k)).unwrap_or_default(),
        a.v,
        a.status,
        if a.archived { "  ARCHIVED" } else { "" }
    );
    s.push_str(&format!("  {}\n", a.title));
    let mut meta = format!(
        "  {} · {} · {}",
        a.provenance,
        n.front.actor,
        n.front.created.split('T').next().unwrap_or("")
    );
    if let Some(g) = &a.grounding {
        meta.push_str(&format!(" · grounding: {}", g));
    }
    if let Some(k) = &a.kind {
        meta.push_str(&format!(" · kind: {}", k));
    }
    if let Some(r) = &n.front.ratified {
        meta.push_str(&format!(" · RATIFIED by {} {}", r.by, r.date));
    }
    if let Some(w) = a.weight {
        if w > 0 {
            meta.push_str(&format!(" · holds {} build(s)", w));
        }
    }
    if let Some(p) = &n.front.path {
        meta.push_str(&format!(" · path: {}", p));
    }
    if let Some(m) = &n.front.method {
        meta.push_str(&format!(" · method: {}", m));
    }
    if !a.areas.is_empty() {
        meta.push_str(&format!(" · areas: {}", a.areas.join(", ")));
    }
    s.push_str(&meta);
    s.push('\n');
    s
}

/// Raw title access for NON-RENDER paths: matching, lexicons, carriers,
/// serialization. Reaching for this to build a display line is exactly what
/// the lint exists to catch — registers ride the renderers above.
pub fn title_raw(n: &Node) -> &str {
    &n.front.title
}

/// The one mutation path (q set title=): a retitle is an act, not a render.
pub fn retitle(n: &mut Node, title: String) {
    n.front.title = title;
}

/// The atom lint, callable from wrap and from CI: every `front.title` token
/// under src/ lives in this file. Returns the offending file:line pairs;
/// empty means the invariant holds. `src_root` is the directory holding the
/// quarry sources (the repo's src/), so host-repo wraps that carry no
/// quarry source skip it naturally.
pub fn lint_sources(src_root: &std::path::Path) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(src_root) else {
        return out;
    };
    for e in entries.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.extension().map_or(true, |x| x != "rs") {
            continue;
        }
        if p.file_stem().map_or(false, |s| s == "surface") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_string();
        for (i, line) in text.lines().enumerate() {
            if line.contains("front.title") {
                out.push((name.clone(), i + 1));
            }
        }
    }
    out
}
