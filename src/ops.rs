use anyhow::{anyhow, bail, Result};
use serde_json::json;

use crate::model::*;
use crate::store::Store;

pub fn derive_provenance(actor: &str) -> String {
    // The CLI is agent-operated (ruled 2026-08-08): an unattributed write
    // defaults to ASSISTANT. Without this, a fresh agent with QUARRY_ACTOR
    // unset falls back to the git user.name — a human name — and silently
    // mints assistant work as user provenance, corrupting the one field the
    // whole system treats as hard. User provenance is explicit (--provenance
    // user / --by user) or comes from a deliberate non-claude QUARRY_ACTOR;
    // it is never inferred from a git config.
    if std::env::var("QUARRY_ACTOR").map_or(true, |s| s.trim().is_empty()) {
        return "assistant".into();
    }
    if actor.to_lowercase().contains("claude") {
        "assistant".into()
    } else {
        "user".into()
    }
}

fn truncate_title(text: &str, max: usize) -> String {
    let one_line = text.lines().next().unwrap_or("").trim();
    if one_line.chars().count() <= max {
        one_line.to_string()
    } else {
        let cut: String = one_line.chars().take(max - 1).collect();
        format!("{}…", cut.trim_end())
    }
}

pub struct NewArgs {
    pub ty: String,
    pub title: String,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub provenance: Option<String>,
    pub about: Vec<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub ratified_by: Option<String>,
    pub body: String,
    pub acceptance: Vec<String>,
    /// Project-declared fields, "k=v" (protocol layer).
    pub fields: Vec<String>,
    pub note: Option<String>,
}

impl NewArgs {
    pub fn bare(ty: &str, title: &str) -> NewArgs {
        NewArgs {
            ty: ty.into(),
            title: title.into(),
            kind: None,
            status: None,
            provenance: None,
            about: vec![],
            path: None,
            method: None,
            ratified_by: None,
            body: String::new(),
            acceptance: vec![],
            fields: vec![],
            note: None,
        }
    }
}

fn parse_project_field(f: &str) -> Result<(String, serde_yaml::Value)> {
    let (k, v) = f
        .split_once('=')
        .ok_or_else(|| anyhow!("expected field=value, got '{}'", f))?;
    if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        bail!("field name '{}' must be alphanumeric/underscore", k);
    }
    Ok((k.to_string(), serde_yaml::Value::String(v.to_string())))
}

pub fn new_node(store: &Store, a: NewArgs) -> Result<Node> {
    let all = store.load_all()?;
    prefix_of(&a.ty)?;
    let actor = Store::actor();
    let provenance = a
        .provenance
        .clone()
        .unwrap_or_else(|| derive_provenance(&actor));
    let status = a
        .status
        .clone()
        .unwrap_or_else(|| default_status(&a.ty).to_string());
    if !allowed_statuses(&a.ty).contains(&status.as_str()) {
        bail!(
            "status '{}' not valid for {} (allowed: {})",
            status,
            a.ty,
            allowed_statuses(&a.ty).join(", ")
        );
    }
    let id = store.mint_id(&a.ty, &all)?;
    let mut edges = Vec::new();
    for t in &a.about {
        edges.push(stamp_edge(store, &all, &a.ty, "about", t, false)?);
    }
    let blob = match (a.ty.as_str(), &a.path) {
        ("doc", Some(p)) => Some(store.blob(p)?),
        _ => None,
    };
    let ratified = a.ratified_by.as_ref().map(|by| Ratified {
        by: by.clone(),
        date: Store::today(),
    });
    let mut extra = std::collections::BTreeMap::new();
    for f in &a.fields {
        let (k, v) = parse_project_field(f)?;
        extra.insert(k, v);
    }
    let front = Front {
        id: id.clone(),
        ty: a.ty.clone(),
        title: a.title.clone(),
        v: 1,
        status,
        provenance,
        created: Store::now(),
        actor: actor.clone(),
        kind: a.kind,
        path: a.path,
        blob,
        method: a.method,
        ratified,
        acceptance: a.acceptance,
        write_set: vec![],
        aliases: vec![],
        archived: false,
        edges,
        extra,
    };
    let file = store
        .nodes_dir()
        .join(&a.ty)
        .join(format!("{}-{}.md", id, slugify(&a.title)));
    let node = Node {
        front,
        body: a.body,
        file,
    };
    store.save(&node)?;
    store.log_event(json!({
        "ts": Store::now(), "node": id, "v": 1, "op": "create",
        "type": a.ty, "title": a.title, "actor": actor, "note": a.note
    }))?;
    Ok(node)
}

/// Build a stamped edge (target version or file blob), enforcing the matrix and C5.
pub fn stamp_edge(
    store: &Store,
    all: &[Node],
    src_ty: &str,
    rel: &str,
    dst: &str,
    acknowledge: bool,
) -> Result<Edge> {
    if !RELS.contains(&rel) {
        bail!("unknown rel '{}' (one of: {})", rel, RELS.join(", "));
    }
    if let Some(fref) = dst.strip_prefix("file:") {
        validate_edge(src_ty, rel, None)?;
        let blob = store.blob(fref)?;
        return Ok(Edge {
            rel: rel.into(),
            to: dst.into(),
            at: At::Blob(blob),
        });
    }
    let target = store.find(all, dst)?;
    validate_edge(src_ty, rel, Some(&target.front.ty))?;
    if matches!(target.front.status.as_str(), "refuted" | "superseded") && !acknowledge {
        bail!(
            "C5: target {} \"{}\" is {} — pass --acknowledge to cite it anyway",
            target.front.id,
            target.front.title,
            target.front.status
        );
    }
    Ok(Edge {
        rel: rel.into(),
        to: target.front.id.clone(),
        at: At::V(target.front.v),
    })
}

/// Bump a node's version, save it, and log the event (the only mutation path).
pub fn bump(
    store: &Store,
    node: &mut Node,
    mut ev: serde_json::Value,
    note: Option<String>,
) -> Result<()> {
    node.front.v += 1;
    store.save(node)?;
    let o = ev
        .as_object_mut()
        .ok_or_else(|| anyhow!("event must be an object"))?;
    o.insert("ts".into(), json!(Store::now()));
    o.insert("node".into(), json!(node.front.id));
    o.insert("v".into(), json!(node.front.v));
    o.insert("actor".into(), json!(Store::actor()));
    if let Some(n) = note {
        o.insert("note".into(), json!(n));
    }
    store.log_event(ev)
}

pub fn link(
    store: &Store,
    src_key: &str,
    rel: &str,
    dst: &str,
    acknowledge: bool,
    note: Option<String>,
) -> Result<Edge> {
    let all = store.load_all()?;
    let src = store.find(&all, src_key)?.clone();
    // C3: assistant-provenance may not settle/supersede user-provenance.
    if matches!(rel, "settles" | "supersedes") && src.front.provenance == "assistant" {
        if !dst.starts_with("file:") {
            if let Ok(t) = store.find(&all, dst) {
                if t.front.provenance == "user" {
                    bail!(
                        "C3: an assistant-provenance {} may not {} the user-provenance {} \"{}\" — record the user's own ruling (`q rule --by user`) or queue a thread",
                        src.front.ty, rel, t.front.ty, t.front.title
                    );
                }
            }
        }
    }
    let edge = stamp_edge(store, &all, &src.front.ty, rel, dst, acknowledge)?;
    if src
        .front
        .edges
        .iter()
        .any(|e| e.rel == edge.rel && e.to == edge.to)
    {
        bail!("edge {} -[{}]-> {} already exists", src.front.id, rel, edge.to);
    }
    let mut src = src;
    src.front.edges.push(edge.clone());
    bump(
        store,
        &mut src,
        json!({"op": "link", "rel": edge.rel, "to": edge.to, "at": edge.at}),
        note,
    )?;
    // supersedes flips the target's status.
    if rel == "supersedes" {
        let all2 = store.load_all()?;
        let mut t = store.find(&all2, dst)?.clone();
        if matches!(t.front.ty.as_str(), "decision" | "claim" | "doc")
            && t.front.status != "superseded"
        {
            let from = t.front.status.clone();
            t.front.status = "superseded".into();
            bump(
                store,
                &mut t,
                json!({"op": "set", "field": "status", "from": from, "to": "superseded", "cause": src.front.id}),
                None,
            )?;
        }
    }
    Ok(edge)
}

pub fn set(store: &Store, key: &str, fields: &[String], note: Option<String>) -> Result<Node> {
    let all = store.load_all()?;
    let mut node = store.find(&all, key)?.clone();
    let mut from_status: Option<String> = None;
    for f in fields {
        let (k, v) = f
            .split_once('=')
            .ok_or_else(|| anyhow!("expected field=value, got '{}'", f))?;
        match k {
            "status" => {
                if !allowed_statuses(&node.front.ty).contains(&v) {
                    bail!(
                        "status '{}' not valid for {} (allowed: {})",
                        v,
                        node.front.ty,
                        allowed_statuses(&node.front.ty).join(", ")
                    );
                }
                from_status = Some(node.front.status.clone());
                node.front.status = v.into();
            }
            "title" => {
                if let Some(old_slug) = node.slug() {
                    if !node.front.aliases.contains(&old_slug) {
                        node.front.aliases.push(old_slug);
                    }
                }
                node.front.title = v.into();
            }
            "kind" => node.front.kind = Some(v.into()),
            "method" => node.front.method = Some(v.into()),
            "path" => node.front.path = Some(v.into()),
            "acceptance+" => node.front.acceptance.push(v.into()),
            "write-set+" | "write_set+" => node.front.write_set.push(v.into()),
            "ratified" => {
                node.front.ratified = Some(Ratified {
                    by: v.into(),
                    date: Store::today(),
                })
            }
            _ => {
                // Project-declared field (protocol layer): preserved, visible,
                // never interpreted by the engine.
                let (key, val) = parse_project_field(f)?;
                node.front.extra.insert(key, val);
            }
        }
    }
    let mut ev = json!({"op": "set", "fields": fields});
    if let Some(fs) = from_status {
        ev.as_object_mut().unwrap().insert("from_status".into(), json!(fs));
    }
    bump(store, &mut node, ev, note)?;
    Ok(node)
}

pub fn edit_body(store: &Store, key: &str, body: String, note: Option<String>) -> Result<Node> {
    let all = store.load_all()?;
    let mut node = store.find(&all, key)?.clone();
    node.body = body;
    bump(store, &mut node, json!({"op": "body"}), note)?;
    Ok(node)
}

pub fn rule(
    store: &Store,
    thread_key: &str,
    text: &str,
    by: Option<String>,
    title: Option<String>,
) -> Result<Node> {
    let all = store.load_all()?;
    let thread = store.find(&all, thread_key)?.clone();
    if thread.front.ty != "thread" {
        bail!("{} is a {}, not a thread", thread.front.id, thread.front.ty);
    }
    if thread.front.status == "resolved" {
        bail!("thread {} is already resolved", thread.front.id);
    }
    let actor = Store::actor();
    let provenance = if by.as_deref() == Some("user") {
        "user".to_string()
    } else {
        derive_provenance(&actor)
    };
    if provenance == "assistant" && thread.front.provenance == "user" {
        bail!(
            "C3: thread \"{}\" is user-provenance; an assistant decision cannot settle it. Pass --by user when recording the user's own ruling.",
            thread.front.title
        );
    }
    let dtitle = title.unwrap_or_else(|| truncate_title(text, 64));
    // The decision inherits the thread's subject attachments.
    let about: Vec<String> = thread
        .front
        .edges
        .iter()
        .filter(|e| e.rel == "about")
        .map(|e| e.to.clone())
        .collect();
    let mut args = NewArgs::bare("decision", &dtitle);
    args.provenance = Some(provenance);
    args.about = about;
    args.body = text.to_string();
    args.ratified_by = by;
    let decision = new_node(store, args)?;
    // Resolve the thread BEFORE stamping the settles edge, so the decision
    // cites the resolved version and rulings never leave a behind marker.
    let all2 = store.load_all()?;
    let mut t = store.find(&all2, &thread.front.id)?.clone();
    let from = t.front.status.clone();
    t.front.status = "resolved".into();
    bump(
        store,
        &mut t,
        json!({"op": "set", "field": "status", "from": from, "to": "resolved", "cause": decision.front.id}),
        None,
    )?;
    link(store, &decision.front.id, "settles", &thread.front.id, true, None)?;
    Ok(decision)
}

#[allow(clippy::too_many_arguments)]
pub fn claim(
    store: &Store,
    text: &str,
    title: Option<String>,
    about: Vec<String>,
    source: Option<String>,
    method: Option<String>,
    provenance: Option<String>,
    status: Option<String>,
) -> Result<Node> {
    if about.is_empty() {
        bail!("C1: a claim needs at least one --about (an area or file: target)");
    }
    let actor = Store::actor();
    let prov = provenance.unwrap_or_else(|| {
        if method.is_some() {
            "measured".into()
        } else {
            derive_provenance(&actor)
        }
    });
    // C2 revised (ruled 2026-08-08): grounding, not documents. A claim must
    // carry a source (doc or file:), a method, or user provenance — the
    // guard is against free-floating assistant assertions.
    if source.is_none() && method.is_none() && prov != "user" {
        bail!(
            "C2: an assistant claim needs grounding — --source <doc or file:path> (where it was extracted or read from), --method \"...\" (how it was measured), or user provenance. No free-floating assertions."
        );
    }
    let title = title.unwrap_or_else(|| truncate_title(text, 72));
    let mut args = NewArgs::bare("claim", &title);
    args.body = text.to_string();
    args.about = about;
    args.method = method;
    args.provenance = Some(prov);
    args.status = status;
    let node = new_node(store, args)?;
    if let Some(s) = source {
        link(store, &node.front.id, "source", &s, false, None)?;
    }
    // Reload so the returned node carries the source edge.
    let all = store.load_all()?;
    Ok(store.find(&all, &node.front.id)?.clone())
}

/// Flip a claim to refuted, record the evidence edge, return the blast radius.
pub fn refute(
    store: &Store,
    claim_key: &str,
    by: &str,
    note: Option<String>,
) -> Result<(Node, Vec<Node>)> {
    let all = store.load_all()?;
    let c = store.find(&all, claim_key)?.clone();
    if c.front.ty != "claim" {
        bail!("{} is a {}, not a claim", c.front.id, c.front.ty);
    }
    if c.front.status == "refuted" {
        bail!("claim {} is already refuted", c.front.id);
    }
    let mut c2 = c.clone();
    let from = c2.front.status.clone();
    c2.front.status = "refuted".into();
    bump(
        store,
        &mut c2,
        json!({"op": "set", "field": "status", "from": from, "to": "refuted", "cause": by}),
        note.clone(),
    )?;
    link(store, by, "refutes", &c2.front.id, true, note)?;
    let all2 = store.load_all()?;
    let ids = crate::queries::blast(&all2, &c2.front.id);
    let nodes = ids
        .iter()
        .filter_map(|id| all2.iter().find(|n| &n.front.id == id).cloned())
        .collect();
    Ok((c2, nodes))
}

/// Archive (or --undo): a presentation flag, never a removal. Only settled
/// statuses qualify; parents with live children never do — a parent indexes
/// its archived offspring. Docs/journals are the record and never archive;
/// areas retire by status. No version bump: archiving is bookkeeping, not
/// content (the affirm-no-bump ruling's rationale), so citers stay current.
pub fn archive(store: &Store, key: &str, undo: bool) -> Result<Node> {
    let all = store.load_all()?;
    let mut node = store.find(&all, key)?.clone();
    if undo {
        if !node.front.archived {
            bail!("\"{}\" is not archived", node.front.title);
        }
        node.front.archived = false;
        store.save(&node)?;
        store.log_event(json!({
            "ts": Store::now(), "node": node.front.id, "v": node.front.v,
            "op": "unarchive", "actor": Store::actor()
        }))?;
        return Ok(node);
    }
    if node.front.archived {
        bail!("\"{}\" is already archived", node.front.title);
    }
    match node.front.ty.as_str() {
        "area" => bail!("areas never archive — they are the map; retire one with status=retired"),
        "doc" => bail!("docs and journals never archive — they are the record"),
        _ => {}
    }
    if !matches!(
        node.front.status.as_str(),
        "done" | "dropped" | "resolved" | "refuted" | "superseded"
    ) {
        bail!(
            "only settled statuses archive; \"{}\" is [{}] — archive follows status, never age",
            node.front.title,
            node.front.status
        );
    }
    let live_children: Vec<&Node> = all
        .iter()
        .filter(|c| {
            !c.front.archived
                && c.front.edges.iter().any(|e| e.rel == "part-of" && e.to == node.front.id)
        })
        .collect();
    if !live_children.is_empty() {
        bail!(
            "\"{}\" has {} live child(ren) — parents index their offspring; archive the leaves first ({})",
            node.front.title,
            live_children.len(),
            live_children
                .iter()
                .map(|c| c.front.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    node.front.archived = true;
    store.save(&node)?;
    store.log_event(json!({
        "ts": Store::now(), "node": node.front.id, "v": node.front.v,
        "op": "archive", "actor": Store::actor()
    }))?;
    Ok(node)
}

#[derive(Debug)]
pub struct DispatchOutcome {
    pub item_id: String,
    pub item_title: String,
    pub globs: Vec<String>,
    /// Re-dispatch: this session already held the lease and kept it.
    pub reused_lease: bool,
    /// The hand-off block: badge instruction + derived brief, paste-whole.
    pub payload: String,
}

/// `q dispatch <item>`: one act — derived brief (rendered and logged, C8's
/// substance), lease, in-flight, machine-local badge, hand-off payload.
/// Never a landing: the item comes back through `q harvest` by the
/// dispatcher's own hand.
pub fn dispatch(
    store: &Store,
    key: &str,
    files: Vec<String>,
    shared: bool,
    session: &str,
    actor: &str,
) -> Result<DispatchOutcome> {
    let all = store.load_all()?;
    let item = store.find(&all, key)?.clone();
    if item.front.ty != "item" {
        bail!("{} is a {}, not an item — dispatch hands off items", item.front.id, item.front.ty);
    }
    if matches!(item.front.status.as_str(), "done" | "dropped") {
        bail!(
            "\"{}\" is already [{}] — dispatch moves live work. If the arc truly resumes, reopen it deliberately first: q set {} status=ready",
            item.front.title, item.front.status, item.front.id
        );
    }
    if let Some(d) = crate::coord::load_dispatch(store) {
        if d.item != item.front.id {
            bail!(
                "a dispatch is already active on this machine: \"{}\" ({}) — one badge at a time keeps observation honest. Harvest it first: q harvest {}",
                d.item_title, d.item, d.item
            );
        }
    }
    // The brief act is logged up front (C8: the lease follows a brief); the
    // TEXT renders after the lease is taken, so the payload's write-set
    // section shows the contract the agent actually works under.
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "brief", "actor": actor, "session": session
    }))?;
    let existing = crate::coord::load_leases(store)
        .into_iter()
        .find(|l| l.item == item.front.id);
    let (globs, reused_lease) = match existing {
        Some(l) if l.session == session => (l.globs, true),
        Some(l) => bail!(
            "\"{}\" is leased by session {} ({:?}) — a dispatch would double-hold. Coordinate with the holder, or they harvest/release first.",
            item.front.title, l.session, l.globs
        ),
        None => {
            let globs = if !files.is_empty() { files } else { item.front.write_set.clone() };
            if globs.is_empty() {
                bail!(
                    "a dispatch leases a write-set — pass --files <globs> (use ** to cover files the work will create): q dispatch {} --files \"src/**\"",
                    item.front.id
                );
            }
            crate::coord::reserve(store, &item, session, actor, globs.clone(), shared, false, None)?;
            (globs, false)
        }
    };
    if item.front.status != "in-flight" {
        set(store, &item.front.id, &["status=in-flight".to_string()], None)?;
    }
    let cursor = store.read_log().map(|l| l.len()).unwrap_or(0) as u64;
    let now = Store::now();
    crate::coord::save_dispatch(
        store,
        &crate::coord::DispatchState {
            item: item.front.id.clone(),
            item_title: item.front.title.clone(),
            session: session.to_string(),
            globs: globs.clone(),
            acceptance: item.front.acceptance.clone(),
            since: now.clone(),
            cursor,
            checked: now,
        },
    )?;
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "dispatch", "actor": actor, "session": session, "globs": globs
    }))?;
    // Rendered after the lease and the state save: the payload's write-set
    // section shows the live contract, not the pre-dispatch void.
    let brief_text = crate::render::brief(store, &item.front.id)?;
    let payload = format!(
        "You are dispatched under badge QUARRY_DISPATCH={id}. Export it in every shell that runs q:\n  \
         pwsh: $env:QUARRY_DISPATCH='{id}'   ·   bash: export QUARRY_DISPATCH={id}\n\
         (the write guard also reads the badge recorded machine-locally at dispatch, so your file\n\
         writes are observed even where your shell env cannot reach; the export stamps your q acts.)\n\n{brief}",
        id = item.front.id,
        brief = brief_text
    );
    Ok(DispatchOutcome {
        item_id: item.front.id.clone(),
        item_title: item.front.title.clone(),
        globs,
        reused_lease,
        payload,
    })
}

/// Re-stamp behind edges (and a path-backed doc's blob) after actual review.
pub fn affirm(store: &Store, key: &str, only_to: Option<String>) -> Result<usize> {
    let all = store.load_all()?;
    let mut node = store.find(&all, key)?.clone();
    let mut count = 0usize;
    let mut edges = node.front.edges.clone();
    for e in &mut edges {
        if let Some(filter) = &only_to {
            if &e.to != filter {
                continue;
            }
        }
        match &e.at {
            At::V(v) => {
                if let Some(t) = all.iter().find(|n| n.front.id == e.to) {
                    if t.front.v > *v {
                        e.at = At::V(t.front.v);
                        count += 1;
                    }
                }
            }
            At::Blob(b) => {
                if let Some(f) = e.to.strip_prefix("file:") {
                    if let Ok(nb) = store.blob(f) {
                        if &nb != b {
                            e.at = At::Blob(nb);
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    // A path-backed doc re-affirms its own file.
    let mut new_blob = None;
    if node.front.ty == "doc" {
        if let (Some(p), Some(old)) = (&node.front.path, &node.front.blob) {
            if let Ok(nb) = store.blob(p) {
                if &nb != old {
                    new_blob = Some(nb);
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        node.front.edges = edges;
        if let Some(nb) = new_blob {
            node.front.blob = Some(nb);
        }
        // Ruled 2026-08-08 (user, settles th-uvu9): an affirm that only
        // restamps does NOT bump v — a restamp is bookkeeping, not content,
        // so review never cascades review. Logged, but the version holds.
        store.save(&node)?;
        store.log_event(json!({
            "ts": Store::now(), "node": node.front.id, "v": node.front.v,
            "op": "affirm", "restamped": count, "actor": Store::actor()
        }))?;
    }
    Ok(count)
}
