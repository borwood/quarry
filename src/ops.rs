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

/// The witness pen's line check (dc-mpg8): from a witness seat an
/// acceptance line is a transcription — a witnessed defect's negation —
/// and no name enters the register from that seat. A backticked span in
/// the line refuses by construction; naming is design's. One check point
/// for both authoring stations (q new --acceptance and q set acceptance+=).
fn witness_line_check(seat: &crate::coord::WitnessSeat, lines: &[String]) -> Result<()> {
    for line in lines {
        let names = crate::queries::backticked_spans(line);
        if let Some(name) = names.first() {
            bail!(
                "witness pen (dc-mpg8): the acceptance line carries a backticked register name (`{}`) and this seat (session {}, kind {}) holds the witness pen — transcription only: a witnessed defect's negation, never invented intent, and no new names enter the register from this seat. Naming is design's. If the plea is real, it is a thread: put the evidence on the thread it informs, or open one — q new thread \"<the plea>\" --about <area>.",
                name, seat.session, seat.kind
            );
        }
    }
    Ok(())
}

/// The witness pen's sequence check (dc-mpg8): a seat that met the
/// acceptance gate's refusal on an item may not then author that item's
/// contract — the self-authorization sequence (the it-hapc class) refuses
/// by construction. The memory is the gate-refusal event
/// acceptance_backstop logs; the match is the seat's session or its
/// acting identity key.
fn witness_sequence_check(store: &Store, item: &Node, seat: &crate::coord::WitnessSeat) -> Result<()> {
    let log = store.read_log()?;
    let met = log.iter().any(|ev| {
        ev.get("op").and_then(|v| v.as_str()) == Some("gate-refusal")
            && ev.get("node").and_then(|v| v.as_str()) == Some(item.front.id.as_str())
            && (ev.get("session").and_then(|v| v.as_str()) == Some(seat.session.as_str())
                || ev.get("refused_key").and_then(|v| v.as_str()) == Some(seat.key.as_str()))
    });
    if met {
        bail!(
            "witness pen (dc-mpg8): this seat (session {}) met the acceptance gate's refusal on {} — authoring the contract after the gate refused it is the self-authorization sequence the pen refuses by construction (the it-hapc class). The pen stays with design: acceptance authored there re-readies the item, or the plea rides a thread — evidence onto the thread it informs, or q new thread \"<the plea>\" --about <area>.",
            seat.session,
            crate::surface::atom_ref(&crate::surface::atom(&[], item))
        );
    }
    Ok(())
}

/// The marks a witness seat's authoring stamps (dc-mpg8): one per line,
/// author and seat transcribed, unratified — the design wake's review
/// channel surfaces them until the user's word lands.
fn witness_marks(seat: &crate::coord::WitnessSeat, lines: &[String]) -> Vec<WitnessMark> {
    lines
        .iter()
        .map(|l| WitnessMark {
            line: l.clone(),
            by: seat.key.clone(),
            session: Some(seat.session.clone()),
            kind: Some(seat.kind.clone()),
            date: Store::today(),
            ratified: None,
            removed: false,
        })
        .collect()
}

/// The removal mark (it-ds6b): a witness seat's `acceptance-=` is
/// authoring-by-subtraction (dc-mpg8), so the removal itself rides the
/// design wake's review channel until the user ratifies. The mark stores
/// the FULL resolved line removed — the record is faithful whatever was
/// typed — and `removed` keeps its flag live though the line is gone.
fn witness_removal_marks(seat: &crate::coord::WitnessSeat, lines: &[String]) -> Vec<WitnessMark> {
    lines
        .iter()
        .map(|l| WitnessMark {
            line: l.clone(),
            by: seat.key.clone(),
            session: Some(seat.session.clone()),
            kind: Some(seat.kind.clone()),
            date: Store::today(),
            ratified: None,
            removed: true,
        })
        .collect()
}

/// The witness pen's executor check (dc-mpg8): the authoring badge cannot
/// join or solo-build the item it authored — author is never executor,
/// mechanically. Compared: the acting identity key against each mark's
/// author, and (on the solo station only) the session against the mark's
/// authoring session — a joined subagent legitimately inherits the
/// dispatcher's session env, so the join road compares keys alone.
/// Authorship does not wash off with ratification: a user-ratified line
/// still bars its author from executing.
pub fn witness_execution_check(
    store: &Store,
    item_key: &str,
    key: &str,
    session: Option<&str>,
) -> Result<()> {
    let all = store.load_all()?;
    let Ok(item) = store.find(&all, item_key) else {
        return Ok(());
    };
    for m in &item.front.witness {
        let by_key = m.by == key;
        let by_session = session.is_some() && m.session.as_deref() == session;
        if by_key || by_session {
            let who = if by_key {
                format!("this identity ({})", key)
            } else {
                format!("this session ({})", session.unwrap_or("?"))
            };
            bail!(
                "witness pen (dc-mpg8): {} authored acceptance on {} from a witness seat — author is never executor, by construction. The work belongs to another mind: the dispatcher re-dispatches it to a fresh agent (q dispatch {}), and this seat's further evidence rides the thread it informs, never the contract.",
                who,
                crate::surface::atom_ref(&crate::surface::atom(&all, item)),
                item.front.id
            );
        }
    }
    Ok(())
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
    // The acceptance gate at mint (dc-p6z4): the ready flip refuses without
    // acceptance lines, so minting straight to ready must refuse the same —
    // ready-implies-acceptance holds by construction on every path to ready.
    if a.ty == "item" && status == "ready" && a.acceptance.is_empty() {
        bail!(
            "acceptance gate (dc-p6z4): an item cannot mint straight to ready with no acceptance lines — ready is stored intent, and intent without acceptance names is unmeasurable. State what done means: --acceptance \"<outcome>\" (repeatable, one line per outcome), or mint it shaped and author acceptance before the flip."
        );
    }
    // The witness pen at filing (dc-mpg8): acceptance authored at mint from
    // a witness seat passes the line check (no register names) and is
    // marked. No sequence check here — a gate refusal presupposes an
    // existing item, and this one is being born.
    let witness = if a.ty == "item" && !a.acceptance.is_empty() {
        match crate::coord::witness_seat(store) {
            Some(seat) => {
                witness_line_check(&seat, &a.acceptance)?;
                witness_marks(&seat, &a.acceptance)
            }
            None => vec![],
        }
    } else {
        vec![]
    };
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
        witness,
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
            "C5: target {} is {} — pass --acknowledge to cite it anyway",
            crate::surface::atom_ref(&crate::surface::atom(&[], target)),
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
                        "C3: an assistant-provenance {} may not {} the user-provenance {} — record the user's own ruling (`q rule --by user`) or queue a thread",
                        src.front.ty, rel,
                        crate::surface::atom_ref(&crate::surface::atom(&all, t))
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

/// `q unlink <src> <rel> <dst>`: retire an edge as a logged act (DESIGN.md
/// § 5 — edge mutations are log events). The edge leaves the source's
/// frontmatter with NO version bump on either node: retirement is
/// bookkeeping, not content (the affirm-no-bump rationale), so citers of
/// the source never go behind over housekeeping. The log records who, when,
/// and — via --note — why. Retiring an edge that does not exist refuses,
/// teaching what does.
pub fn unlink(
    store: &Store,
    src_key: &str,
    rel: &str,
    dst: &str,
    note: Option<String>,
) -> Result<(Node, Edge)> {
    let all = store.load_all()?;
    let mut src = store.find(&all, src_key)?.clone();
    // Resolve the target: file: refs stay verbatim; node keys resolve to the
    // id; an unresolvable key falls back to the literal — a dangling edge
    // (target since deleted) must still be retirable.
    let dst_id = if dst.starts_with("file:") {
        dst.to_string()
    } else {
        store
            .find(&all, dst)
            .map(|n| n.front.id.clone())
            .unwrap_or_else(|_| dst.to_string())
    };
    let Some(pos) = src
        .front
        .edges
        .iter()
        .position(|e| e.rel == rel && e.to == dst_id)
    else {
        let existing: Vec<String> = src
            .front
            .edges
            .iter()
            .map(|e| format!("-[{}]-> {} (at {})", e.rel, e.to, e.at))
            .collect();
        bail!(
            "no edge {} -[{}]-> {} to retire — what {} carries:\n  {}",
            src.front.id,
            rel,
            dst_id,
            src.front.id,
            if existing.is_empty() {
                "(no edges at all)".to_string()
            } else {
                existing.join("\n  ")
            }
        );
    };
    let edge = src.front.edges.remove(pos);
    // NO bump on either node: an edge retirement adds no content for the
    // source's citers, so it must not put them behind (dc-q8p3's rationale).
    store.save(&src)?;
    let mut ev = json!({
        "ts": Store::now(), "node": src.front.id, "v": src.front.v,
        "op": "unlink", "rel": edge.rel, "to": edge.to, "at": edge.at,
        "actor": Store::actor()
    });
    if let Some(n) = note {
        ev.as_object_mut().unwrap().insert("note".into(), json!(n));
    }
    store.log_event(ev)?;
    Ok((src, edge))
}

/// What a `q set` act did beyond the node itself (it-ds6b): the FULL
/// resolved acceptance lines removed — the record is the resolved line,
/// never what was typed (the mint-echo pattern: a wrong-but-real match
/// reads wrong in the echo) — the loud demotion when a strip left a
/// readied item contract-less (dc-p6z4 at mutation time), and whether the
/// removal was authored from a witness seat (dc-mpg8: authoring-by-
/// subtraction is authoring; the review channel carries it).
#[derive(Debug)]
pub struct SetOutcome {
    pub node: Node,
    /// Full resolved acceptance lines removed, in act order.
    pub removed: Vec<String>,
    /// The status the strip demoted from (ready or in-flight → shaped).
    pub demoted_from: Option<String>,
    /// Removal marks recorded (witness seat only; design seats mutate free).
    pub witness_removals: usize,
}

/// Resolve one `acceptance-=` value to the line it removes (it-ds6b):
/// the exact line wins outright; otherwise any substring matching exactly
/// one line resolves — forgiving input, faithful record. Zero or multiple
/// matches refuse, listing candidates — never a silent no-op. Strict-
/// verbatim input would rebuild the forcing incident's trap (long
/// backtick-laden lines round-tripping shell quoting) on the removal side.
fn resolve_acceptance_removal(node: &Node, typed: &str) -> Result<usize> {
    let lines = &node.front.acceptance;
    let aref = crate::surface::atom_ref(&crate::surface::atom(&[], node));
    if lines.is_empty() {
        bail!(
            "acceptance-= on {} — it carries no acceptance lines; nothing to remove.",
            aref
        );
    }
    if let Some(i) = lines.iter().position(|l| l == typed) {
        return Ok(i);
    }
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains(typed))
        .map(|(i, _)| i)
        .collect();
    match hits.len() {
        1 => Ok(hits[0]),
        0 => bail!(
            "acceptance-= matched no line on {} — the value is the exact line, or any substring matching exactly one. Nothing was removed. What stands:\n  {}",
            aref,
            lines.join("\n  ")
        ),
        n => bail!(
            "acceptance-= is ambiguous on {} — \"{}\" sits in {} lines. Nothing was removed; give a longer substring or the exact line. Candidates:\n  {}",
            aref,
            typed,
            n,
            hits.iter().map(|&i| lines[i].as_str()).collect::<Vec<_>>().join("\n  ")
        ),
    }
}

pub fn set(store: &Store, key: &str, fields: &[String], note: Option<String>) -> Result<SetOutcome> {
    let all = store.load_all()?;
    let mut node = store.find(&all, key)?.clone();
    let mut from_status: Option<String> = None;
    let mut new_acceptance: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    // The logged fields carry the FULL resolved line for every removal —
    // the log stores what was actually removed, never what was typed.
    let mut logged_fields: Vec<String> = Vec::new();
    for f in fields {
        let (k, v) = f
            .split_once('=')
            .ok_or_else(|| anyhow!("expected field=value, got '{}'", f))?;
        let mut logged = f.clone();
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
                crate::surface::retitle(&mut node, v.into());
            }
            "kind" => node.front.kind = Some(v.into()),
            "method" => node.front.method = Some(v.into()),
            "path" => node.front.path = Some(v.into()),
            "acceptance+" => {
                node.front.acceptance.push(v.into());
                new_acceptance.push(v.into());
            }
            // Removal by line (it-ds6b): the -= value resolves against the
            // lines as they stand at this point in the act, so replace is
            // both fields in one call — "acceptance-=<old>" then
            // "acceptance+=<new>" — one act, one log event, one bump.
            "acceptance-" => {
                let idx = resolve_acceptance_removal(&node, v)?;
                let line = node.front.acceptance.remove(idx);
                logged = format!("acceptance-={}", line);
                removed.push(line);
            }
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
        logged_fields.push(logged);
    }
    // The witness pen at authoring (dc-mpg8): acceptance authored from a
    // witness seat passes the line check (no register names) and the
    // sequence check (never after this seat met the gate's refusal on the
    // item), then the lines are marked — the design wake's review channel
    // carries them until the user's ratify-or-amend. A bail here discards
    // the in-memory clone; nothing has been saved.
    if !new_acceptance.is_empty() && node.front.ty == "item" {
        if let Some(seat) = crate::coord::witness_seat(store) {
            witness_line_check(&seat, &new_acceptance)?;
            witness_sequence_check(store, &node, &seat)?;
            node.front.witness.extend(witness_marks(&seat, &new_acceptance));
        }
    }
    // The witness pen at removal (it-ds6b, dc-mpg8): authoring-by-
    // subtraction is authoring, so a witness seat's removal is marked and
    // rides the same review channel until the user ratifies; design seats
    // mutate freely. No line check — removal takes names out of the
    // register, never in — and the sequence check is vacuous here (the
    // gate only refuses acceptance-less items, which carry nothing to
    // remove).
    let mut witness_removals = 0usize;
    if !removed.is_empty() && node.front.ty == "item" {
        if let Some(seat) = crate::coord::witness_seat(store) {
            let marks = witness_removal_marks(&seat, &removed);
            witness_removals = marks.len();
            node.front.witness.extend(marks);
        }
    }
    // The acceptance gate's construction point (dc-p6z4): ready is stored
    // intent, and intent without acceptance names is unmeasurable — the
    // flip hard-refuses until the contract is stated. Checked after every
    // field lands, so authoring acceptance and flipping ready in one act
    // passes in either order. The refusal faces the shaper and teaches the
    // authoring command; demotions stay free.
    if from_status.is_some()
        && node.front.ty == "item"
        && node.front.status == "ready"
        && node.front.acceptance.is_empty()
    {
        if let Some(fs) = from_status {
            node.front.status = fs;
        }
        bail!(
            "acceptance gate (dc-p6z4): {} has no acceptance lines — ready is stored intent, and intent without acceptance names is unmeasurable. State what done means first: q set {} acceptance+=\"<outcome>\" (repeatable, one line per outcome), then flip ready.",
            crate::surface::atom_ref(&crate::surface::atom(&[], &node)),
            node.front.id
        );
    }
    // The it-ds6b rider on dc-p6z4, at mutation time: an edit that strips
    // a readied item's last acceptance line loudly demotes ready to shaped
    // in this verb's own act — the ready feed carries only items whose
    // contract is stated. In-flight demotes the same way, mirroring the
    // fire-time backstop's arm: an acceptance-less item past the gate is a
    // breach state, never left standing. One act, one log event, one bump.
    let demoted_from = if !removed.is_empty()
        && node.front.ty == "item"
        && node.front.acceptance.is_empty()
        && matches!(node.front.status.as_str(), "ready" | "in-flight")
    {
        let from = node.front.status.clone();
        node.front.status = "shaped".into();
        Some(from)
    } else {
        None
    };
    // The logged fields carry resolved removals; the removed lines land
    // whole on the event besides — who and why ride actor and --note, in
    // the unlink spirit. Unlike unlink this BUMPS (via bump below): the
    // contract is content, not bookkeeping, so citers' stamps go behind.
    let mut ev = json!({"op": "set", "fields": logged_fields});
    if let Some(fs) = &from_status {
        ev.as_object_mut().unwrap().insert("from_status".into(), json!(fs));
    }
    if !removed.is_empty() {
        ev.as_object_mut()
            .unwrap()
            .insert("removed_acceptance".into(), json!(removed));
    }
    if let Some(df) = &demoted_from {
        ev.as_object_mut().unwrap().insert(
            "demoted".into(),
            json!({"from": df, "to": "shaped", "cause": "acceptance gate (dc-p6z4): last acceptance line stripped"}),
        );
    }
    bump(store, &mut node, ev, note)?;
    Ok(SetOutcome {
        node,
        removed,
        demoted_from,
        witness_removals,
    })
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
            "C3: thread {} is user-provenance; an assistant decision cannot settle it. Pass --by user when recording the user's own ruling.",
            crate::surface::atom_ref(&crate::surface::atom(&[], &thread))
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
    kind: Option<String>,
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
    // The species split at mint (dc-6gn9): --kind carries any species
    // string — the engine stays kindless-but-kind-aware (dc-yd9s), never
    // validates. A method-carrying claim minted kindless draws the species
    // prompt at the mint surface, a prompt, never a gate.
    args.kind = kind;
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

/// What a landing assayed (dc-drr6): the arc's freshly ratified claims and
/// which path ratified them — harvest (badge mints, the dispatcher's hand)
/// or solo (a session's own mints, self-ratified on the record).
pub struct LandingAssay {
    pub solo: bool,
    pub ratified: Vec<Node>,
}

/// The assay office (dc-drr6): the landing act ratifies the arc's vein and
/// feature mints. A dispatched item's landing ratifies the claims minted
/// under its badge — the dispatcher's landing judgment already verified
/// them against the diff, and ratification records that act by the
/// dispatcher's own hand. A never-dispatched item landing under the session
/// that worked it self-ratifies the session's own mints across the arc
/// (lease since, else the last in-flight flip) — the ratifier is always
/// stamped, so one mind stays legible against two-minds-against-evidence.
/// Ratified never means true: refute and blast stand. Returns None when the
/// arc leaves nothing to assay.
pub fn ratify_landing(
    store: &Store,
    item_id: &str,
    session: Option<&str>,
) -> Result<Option<LandingAssay>> {
    let all = store.load_all()?;
    let log = store.read_log()?;
    let (ids, solo) = if crate::queries::item_was_dispatched(&log, item_id) {
        (crate::queries::badge_claim_mints(&log, item_id), false)
    } else {
        let Some(sess) = session else { return Ok(None) };
        let since = crate::coord::load_leases(store)
            .iter()
            .find(|l| l.item == item_id && l.session == sess)
            .map(|l| l.since.clone())
            .or_else(|| {
                log.iter()
                    .rev()
                    .find(|ev| {
                        ev.get("node").and_then(|v| v.as_str()) == Some(item_id)
                            && ev.get("op").and_then(|v| v.as_str()) == Some("set")
                            && ev
                                .get("fields")
                                .and_then(|v| v.as_array())
                                .map_or(false, |fs| {
                                    fs.iter().any(|f| f.as_str() == Some("status=in-flight"))
                                })
                    })
                    .and_then(|ev| ev.get("ts").and_then(|v| v.as_str()).map(String::from))
            });
        let Some(since) = since else { return Ok(None) };
        (crate::queries::session_claim_mints(&log, sess, &since), true)
    };
    let claims: Vec<Node> =
        crate::queries::assayable(&all, &ids).into_iter().cloned().collect();
    if claims.is_empty() {
        return Ok(None);
    }
    let actor = Store::actor();
    let mut ratified = Vec::new();
    for mut n in claims {
        n.front.status = "ratified".into();
        n.front.ratified = Some(Ratified {
            by: actor.clone(),
            date: Store::today(),
        });
        // No version bump: an assay records who stood behind the claim, not
        // new content — the affirm-no-bump rationale (th-uvu9), so citers of
        // a freshly ratified vein never go behind over good news. The act is
        // loud in the log instead: op ratify, the assayer stamped, the
        // landing item as cause — a fallen claim shows its assayer (dc-drr6).
        store.save(&n)?;
        store.log_event(json!({
            "ts": Store::now(), "node": n.front.id, "v": n.front.v,
            "op": "ratify", "from": "asserted", "cause": item_id,
            "assay": if solo { "solo" } else { "harvest" }, "actor": actor
        }))?;
        ratified.push(n);
    }
    Ok(Some(LandingAssay { solo, ratified }))
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
            bail!("{} is not archived", crate::surface::atom_ref(&crate::surface::atom(&all, &node)));
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
        bail!("{} is already archived", crate::surface::atom_ref(&crate::surface::atom(&all, &node)));
    }
    match node.front.ty.as_str() {
        "area" => bail!("areas never archive — they are the map; retire one with status=retired"),
        "doc" => bail!("docs and journals never archive — they are the record"),
        _ => {}
    }
    // A reading settles by species, not by ladder (dc-6gn9): it is done
    // serving the moment it lands, so any ladder status archives.
    let reading =
        node.front.ty == "claim" && node.front.kind.as_deref() == Some("reading");
    if !reading
        && !matches!(
            node.front.status.as_str(),
            "done" | "dropped" | "resolved" | "refuted" | "superseded"
        )
    {
        bail!(
            "only settled statuses archive; {} — archive follows status, never age",
            crate::surface::atom_ref(&crate::surface::atom(&all, &node))
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
            "{} has {} live child(ren) — parents index their offspring; archive the leaves first ({})",
            crate::surface::atom_ref(&crate::surface::atom(&all, &node)),
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

/// Archive-on-consumption (dc-6gn9): a reading is done serving the moment
/// its last live consumer settles — when only one end exists after a
/// condition, automating the transition removes a dependency on agent
/// discipline and attention budget (the user's principle, kin to
/// enforcement-beats-doctrine). Consumers are the nodes that lean on the
/// reading, in blast's own vocabulary (queries::blast): inbound depends-on,
/// source, and builds-on edges, plus the targets of its own supports edges
/// (the decisions and items it informed). Inbound supports and refutes are
/// evidence and attack, never a lean. A consumer counts as
/// settled when it is archived or its status is terminal for its type —
/// items done/dropped, threads resolved/dropped, decisions in-force or
/// superseded (an in-force ruling has consumed its inputs), claims
/// refuted/superseded (a live claim still leans), docs always (registered
/// is the record). A consumerless reading never sweeps — nothing consumed
/// it, so nothing settles it. Returns the readings this sweep archived;
/// callers say what was hidden.
pub fn consume_readings(store: &Store) -> Result<Vec<Node>> {
    let all = store.load_all()?;
    let mut swept = Vec::new();
    for n in &all {
        if n.front.ty != "claim"
            || n.front.archived
            || n.front.kind.as_deref() != Some("reading")
        {
            continue;
        }
        let mut consumer_ids: Vec<&str> = all
            .iter()
            .filter(|m| {
                m.front.id != n.front.id
                    && m.front.edges.iter().any(|e| {
                        matches!(e.rel.as_str(), "depends-on" | "source" | "builds-on")
                            && e.to == n.front.id
                    })
            })
            .map(|m| m.front.id.as_str())
            .collect();
        for e in &n.front.edges {
            if e.rel == "supports" && !consumer_ids.contains(&e.to.as_str()) {
                consumer_ids.push(&e.to);
            }
        }
        let consumers: Vec<&Node> = consumer_ids
            .iter()
            .filter_map(|id| all.iter().find(|m| &m.front.id == id))
            .collect();
        if consumers.is_empty() || !consumers.iter().all(|c| consumer_settled(c)) {
            continue;
        }
        let mut node = n.clone();
        node.front.archived = true;
        store.save(&node)?;
        store.log_event(json!({
            "ts": Store::now(), "node": node.front.id, "v": node.front.v,
            "op": "archive", "cause": "consumption", "actor": Store::actor()
        }))?;
        swept.push(node);
    }
    Ok(swept)
}

/// Settledness for consumption (dc-6gn9): terminal by type, archived
/// always. Unknown types (areas among them) hold their readings live.
fn consumer_settled(c: &Node) -> bool {
    if c.front.archived {
        return true;
    }
    match c.front.ty.as_str() {
        "item" => matches!(c.front.status.as_str(), "done" | "dropped"),
        "thread" => matches!(c.front.status.as_str(), "resolved" | "dropped"),
        "decision" => matches!(c.front.status.as_str(), "in-force" | "superseded"),
        "claim" => matches!(c.front.status.as_str(), "refuted" | "superseded"),
        "doc" => true,
        _ => false,
    }
}

#[derive(Debug)]
pub struct DispatchOutcome {
    pub item_id: String,
    pub item_title: String,
    pub globs: Vec<String>,
    /// Re-dispatch: this session already held the lease and kept it.
    pub reused_lease: bool,
    /// A cross-chat steal: the (holder key, session) the dispatch was taken
    /// from — reported loud by the caller.
    pub stolen_from: Option<(String, String)>,
    /// The single-use join token minted for this hand-off.
    pub token: String,
    /// The one-line spawn prompt (dc-zbxj): the hand-off is a fetch — the
    /// agent runs q join and the brief renders fresh from the graph, so a
    /// stale or dispatcher-mangled copy is impossible.
    pub spawn: String,
    /// The model stamped into the badge (it-xcvb), verbatim as the
    /// dispatcher typed it. None when the fire named none.
    pub model: Option<String>,
    /// What this arc's graph writes will file under, WHERE THE FIRE CAN KNOW
    /// IT (it-xwpw). `Some` is the stamped model through `safe_actor`, which
    /// the hook prefers over every derived answer, so a stamped fire is
    /// certain of it. `None` is an unstamped fire, and there is nothing here
    /// to name: the agent does not exist yet, and its model resolves from the
    /// harness's own record of it at its own first fire (cl-jp4q). This field
    /// used to fall back to the dispatching chat's actor, which is exactly
    /// the statement it-6ekf made false.
    pub arc_actor: Option<String>,
}

/// The acceptance gate's fire-time backstop (dc-p6z4): reserve and dispatch
/// hard-refuse an acceptance-less item, and the refusal loudly un-readies
/// it back to shaped so the ready feed stays true — catching pre-gate items
/// and any future strip. Teaching is station-appropriate: the dispatcher is
/// not a design station and never authors acceptance, so its refusal
/// teaches the return-to-design move; the solo path is design-capable and
/// is taught the authoring command directly. No inline authoring flags on
/// either station — authoring acceptance is always a shaping act, never a
/// firing act.
pub fn acceptance_backstop(store: &Store, item: &Node, solo: bool) -> Result<()> {
    if item.front.ty != "item" || !item.front.acceptance.is_empty() {
        return Ok(());
    }
    let demoted = matches!(item.front.status.as_str(), "ready" | "in-flight");
    if demoted {
        set(
            store,
            &item.front.id,
            &["status=shaped".to_string()],
            Some("acceptance gate (dc-p6z4): un-readied at fire — no acceptance lines".into()),
        )?;
    }
    let mut named = item.clone();
    if demoted {
        named.front.status = "shaped".into();
    }
    let aref = crate::surface::atom_ref(&crate::surface::atom(&[], &named));
    let demote_line = if demoted {
        format!(
            " UN-READIED: {} → [shaped], loudly (logged) — the ready feed carries only items whose contract is stated.",
            item.front.id
        )
    } else {
        String::new()
    };
    // The gate refusal is the witness pen's memory (dc-mpg8): the sequence
    // check refuses this seat's later authoring on this item against this
    // event. Logged before either bail; log_event stamps session and badge.
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "gate-refusal", "solo": solo, "actor": Store::actor(),
        "refused_key": crate::coord::acting_key(
            crate::coord::current_agent().as_deref(),
            crate::coord::current_chat().as_deref(),
            crate::coord::current_session().as_deref(),
        ),
    }))?;
    if solo {
        // The solo station is design-capable by default (dc-p6z4) and is
        // taught the authoring command — unless this seat holds the witness
        // pen (dc-mpg8), where that teach would be a trap: the sequence
        // check now bars this seat's authoring on this item.
        if let Some(seat) = crate::coord::witness_seat(store) {
            bail!(
                "acceptance gate (dc-p6z4): {} has no acceptance lines — nothing states what done means, so there is nothing to fire against.{} This seat (session {}, kind {}) holds the witness pen (dc-mpg8), and the gate has now refused it here: the contract comes from design (authored there, it re-readies), or the plea rides a thread — evidence onto the thread it informs, or q new thread \"<the plea>\" --about <area>.",
                aref, demote_line, seat.session, seat.kind
            );
        }
        bail!(
            "acceptance gate (dc-p6z4): {} has no acceptance lines — nothing states what done means, so there is nothing to fire against.{} Authoring acceptance is a shaping act: q set {} acceptance+=\"<outcome>\" (repeatable, one line per outcome), then q set {} status=ready and fire again.",
            aref, demote_line, item.front.id, item.front.id
        );
    }
    bail!(
        "acceptance gate (dc-p6z4): {} has no acceptance lines — nothing states what done means, so there is nothing to dispatch against.{} The pen stays with design: a dispatcher never authors acceptance, and after this refusal the witness pen (dc-mpg8) refuses this seat's authoring on this item by sequence. Return it to the design session that shapes this work; acceptance authored there re-readies it. A gap this seat witnessed is a plea: evidence onto the thread it informs, or q new thread \"<the plea>\" --about <area>.",
        aref, demote_line
    );
}

/// `q dispatch <item>`: one act — brief logged (C8's substance; the TEXT
/// renders at q join, fresh), lease, in-flight, machine-local badge with a
/// single-use join token, one-line spawn prompt. Never a landing: the item
/// comes back through `q harvest` by the dispatcher's own hand.
#[allow(clippy::too_many_arguments)]
pub fn dispatch(
    store: &Store,
    key: &str,
    files: Vec<String>,
    shared: bool,
    steal: bool,
    reason: Option<&str>,
    model: Option<&str>,
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
            "{} is already [{}] — dispatch moves live work. If the arc truly resumes, reopen it deliberately first: q set {} status=ready",
            crate::surface::atom_ref(&crate::surface::atom(&[], &item)),
            item.front.status, item.front.id
        );
    }
    // The write-set shape floor (it-x4bb), ahead of everything that mutates:
    // a comma-joined --files value leases one dead glob whose static prefix
    // runs across the commas, so only the first path in it would ever match
    // and the agent is denied its own write-set mid-arc. Refusing the FLAG
    // here — not only inside reserve — keeps a mistyped fire free of side
    // effects: no gate-refusal event, no brief event, no lease, no in-flight
    // flip, nothing to undo before the corrected command re-runs. The other
    // road into the same lease, the fallback to the item's recorded
    // write-set, inherits the floor at `coord::reserve` below, where it
    // refuses beside the foreign-lease refusal and like it.
    crate::coord::check_glob_shapes(&files)?;
    // The fire-time backstop (dc-p6z4), ahead of ownership and lease logic:
    // no brief event, no lease, no in-flight flip on a contract-less item —
    // and the re-dispatch path (lease kept) is covered by sitting here, not
    // inside reserve.
    acceptance_backstop(store, &item, false)?;
    // Per-item ownership (dc-qyr5): a chat holds ANY number of live
    // dispatches — the same-chat second-dispatch refusal is deleted;
    // fire-them-all-off from one chat is literal. What refuses is
    // dispatching an item ANOTHER chat already has live — or steals it
    // whole, loud and logged, with the required reason.
    let dkey = crate::coord::dispatch_key(session);
    let stolen_from = match crate::coord::dispatch_for_item(store, &item.front.id) {
        // Same-chat re-dispatch stays free: the documented recovery flow —
        // a fresh token is minted below and the old one dies with the
        // replaced entry.
        Some(d) if d.holder == dkey => None,
        Some(d) => {
            if !steal {
                bail!(
                    "\"{}\" ({}) is already dispatched — held by {} (session {}, since {}). An item belongs to one chat (dc-qyr5); the holder harvests (q harvest {}) or re-dispatches. Taking it over is a steal, loud and logged: q dispatch {} --steal --reason \"why\"",
                    d.item_title, d.item, d.holder, d.session, d.since, d.item, d.item
                );
            }
            let Some(r) = reason else {
                bail!(
                    "--steal takes {}'s dispatch of \"{}\" ({}) over whole and demands its reason — the required response is the proof of engagement, and it lands on the logged steal event where the holder reads it. Re-run adding: --reason \"why\"",
                    d.holder, d.item_title, d.item
                );
            };
            // The steal takes the dispatch WHOLE (the lease steal pattern
            // applied to dispatches): the held entry moves below, the old
            // token dies, and the acting associations clear here so a
            // stolen-from agent's later acts stop stamping into an arc it
            // no longer works. Loud and logged, reason on the event.
            store.log_event(json!({
                "ts": Store::now(), "node": item.front.id, "v": item.front.v,
                "op": "steal", "from_chat": d.holder, "from_session": d.session,
                "from_joined": d.joined, "actor": actor, "session": session,
                "reason": r
            }))?;
            crate::coord::clear_dispatch(store, &item.front.id);
            Some((d.holder, d.session))
        }
        None => None,
    };
    // The brief act is logged up front (C8: the lease follows a brief); the
    // TEXT renders at q join, after the lease is taken, so the brief's
    // write-set section shows the contract the agent actually works under.
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "brief", "actor": actor, "session": session
    }))?;
    let existing = crate::coord::load_leases(store)
        .into_iter()
        .find(|l| l.item == item.front.id);
    // The arc's own return (it-3prx): the RETURN spec demands a report and
    // the homework prints its registration command, but a write-set names
    // the CODE the work touches — so until this line the guard denied the
    // one artifact the contract mandates, and only the dispatcher's hand
    // could land it. Derived once here, at the station that CAN lease it;
    // the brief names it and harvest prints it back from the lease.
    let report = crate::coord::arc_report_path(
        store,
        &item.front.id,
        crate::surface::title_raw(&item),
    );
    let (mut globs, reused_lease) = match existing {
        Some(l) if l.session == session => (l.globs, true),
        // The steal takes the lease with the dispatch: re-homed under the
        // stealing session, globs intact.
        Some(l) if stolen_from.is_some() => {
            crate::coord::rehome_lease(store, &item, session, actor)?;
            (l.globs, false)
        }
        Some(l) => bail!(
            "{} is leased by session {} ({:?}) — a dispatch would double-hold. Coordinate with the holder, or they harvest/release first.",
            crate::surface::atom_ref(&crate::surface::atom(&[], &item)),
            l.session, l.globs
        ),
        None => {
            let globs = if !files.is_empty() { files } else { item.front.write_set.clone() };
            // The empty check reads the write-set the dispatcher NAMED, and
            // the arc report joins below it: a report path is the contract's
            // artifact, never a write-set, and a dispatch leasing nothing
            // else would be a research dispatch that stopped saying so.
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
    // The report path joins AFTER the lease is taken — deliberately outside
    // `reserve`'s C7 overlap test. Inside it, any arc whose write-set covers
    // `docs/**` would refuse against every other live arc's report path, and
    // docs-touching work would become unfireable while any dispatch flies.
    // There is nothing to contend for: the path is unique to this arc and
    // names a file that does not exist yet, so no co-writer can be standing
    // on it. Every arm re-points here — the reused and stolen leases were
    // taken for a PRIOR arc, and a re-dispatch is a new arc with its own
    // return.
    if crate::coord::set_arc_report(&mut globs, &report) {
        crate::coord::set_lease_globs(store, &item.front.id, &globs)?;
    }
    if item.front.status != "in-flight" {
        set(store, &item.front.id, &["status=in-flight".to_string()], None)?;
    }
    let cursor = store.read_log().map(|l| l.len()).unwrap_or(0) as u64;
    let now = Store::now();
    // The join token: single-use, minted fresh on every dispatch (a
    // re-dispatch is a new hand-off — the old token dies with the replaced
    // entry). The spawn prompt collapses to one line; the manual-export
    // instruction is gone — identity is structural, never discipline
    // (dc-zbxj).
    let token = crate::protocol::mint_token_n(10);
    crate::coord::save_dispatch(
        store,
        &crate::coord::DispatchState {
            item: item.front.id.clone(),
            item_title: crate::surface::title_raw(&item).to_string(),
            session: session.to_string(),
            holder: dkey,
            globs: globs.clone(),
            acceptance: item.front.acceptance.clone(),
            since: now.clone(),
            cursor,
            checked: now,
            token: Some(token.clone()),
            joined: None,
            // The model stamp (it-xcvb), minted with the badge and spent by
            // the session hook at the joined agent's shells. Re-stamped on
            // every fire by construction — a re-dispatch or a steal builds a
            // fresh entry, so an arc handed to a different model carries the
            // model it was handed to.
            model: model.map(|m| m.trim().to_string()).filter(|m| !m.is_empty()),
        },
    )?;
    store.log_event(json!({
        "ts": Store::now(), "node": item.front.id, "v": item.front.v,
        "op": "dispatch", "actor": actor, "session": session, "globs": globs,
        "model": model
    }))?;
    // The spawn line stamps the canonical graph root beside the token
    // (dc-g5x5): join consumes the pin explicitly, then plants it for the
    // identity so every later verb and hook resolves the same locale — a
    // worktree fork's cwd never decides where acts land. It therefore names
    // NO working directory (it-rmqy): under the pin, cwd decides nothing,
    // so directing the agent anywhere can only misdirect — an agent the
    // harness sat in a worktree fork, obeying an "in <root>" clause, leaves
    // its isolation and lands file work in the canonical tree (or edits in
    // the fork while its stamps hash canon content it never wrote, the work
    // root following cwd). Dispatch cannot know where the agent will sit;
    // followed verbatim this line is correct from the canonical tree and
    // from a fork alike, and where the agent actually stands is read from
    // the real cwd at join and stated there.
    let spawn = format!(
        "You are dispatched: run: q join {token} --store {root} — then follow what it prints.",
        root = store.root.display(),
        token = token
    );
    let stamped = model.map(|m| m.trim().to_string()).filter(|m| !m.is_empty());
    Ok(DispatchOutcome {
        item_id: item.front.id.clone(),
        item_title: crate::surface::title_raw(&item).to_string(),
        globs,
        reused_lease,
        stolen_from,
        token,
        spawn,
        arc_actor: stamped.as_deref().map(crate::coord::safe_actor),
        model: stamped,
    })
}

#[derive(Debug)]
pub struct JoinOutcome {
    pub item_id: String,
    pub item_title: String,
    /// The identity key newly bound (None on an idempotent re-join).
    pub bound: Option<String>,
    pub rejoined: bool,
    /// The brief, rendered fresh from the graph at join time — opened by the
    /// where-you-stand banner when the join happens in a fork (it-rmqy).
    pub brief: String,
    /// The actor this arc's graph writes file under (it-xcvb) — resolved
    /// HERE, at the one seat that has the agent's own id in hand, rather
    /// than predicted at the fire. Stated at the join because the joining
    /// agent is the ONE party that knows its own model for certain and can
    /// say so in its report when this reads wrong.
    pub arc_actor: String,
    /// Which of the three roads answered — the join says which rather than
    /// asserting one of them (it-xwpw).
    pub actor_source: ArcActorSource,
}

/// Where a join's attribution answer came from (it-xwpw). Three real states,
/// and they read differently to the agent: one is certain, one is derived
/// from the harness's own record of this very agent, and one is a fallback
/// that may not be the joining agent's model at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcActorSource {
    /// `q dispatch --model` stamped the badge. The hook prefers the stamp
    /// over the harness record (cl-jp4q), so this is what the arc files
    /// under whether or not it is what the agent actually runs.
    Stamp,
    /// The harness's OWN record of this agent id, read at this seat — the
    /// structural answer, owing nothing to the dispatcher's memory.
    Record,
    /// Neither answered: the actor this shell was injected with stands. For
    /// a subagent whose record could not be reached that is the dispatching
    /// chat's model; for a chat- or session-keyed joiner it is its own.
    Injected,
}

/// The where-you-stand banner (it-rmqy): `Some(line)` when the working
/// checkout this process sits in is not the tree the graph lives in — a
/// worktree fork acting under the badge's store pin (dc-g5x5). The spawn
/// line names no directory because dispatch cannot know where the harness
/// will sit the agent; the orientation is therefore owed at join, where the
/// real cwd is in hand, and it opens the brief.
///
/// Composed at the join rather than inside `render::brief` deliberately: the
/// brief's own second line promises everything below it is derived from the
/// graph at render time, and this fact is read from the process's cwd, not
/// from the graph.
pub fn fork_banner(store: &Store) -> Option<String> {
    if store.work_root == store.root {
        return None;
    }
    Some(format!(
        "WHERE YOU STAND: your working checkout is {work} — a fork of the tree this graph lives in ({root}). Work where you stand: read, edit, build, and test HERE, and never cd to the canonical tree to run a verb — that takes your file work out of its isolation, and your blob stamps would hash content you never wrote. Your q acts, those stamps, and every hook-observed write land at the canonical graph under the badge's store pin whatever your cwd (dc-g5x5); the fork's own graph/ copy stays inert and merges clean.",
        work = store.work_root.display(),
        root = store.root.display()
    ))
}

/// `q join <token>`: the fetch half of the dc-zbxj hand-off. Consumes the
/// single-use token minted at dispatch, binds the acting identity (agent →
/// chat → session, hook-injected; resolved by the caller) to the badge in
/// the association map, logs the join under the badge, and renders the
/// brief fresh — the brief doctrine (derived, never hand-carried) applied
/// to the hand-off itself. Re-join by the same identity is idempotent; an
/// identity already bound to a DIFFERENT live badge refuses before the
/// token is spent — one agent, one badge (it-tanf), the refusal down in
/// coord::consume_join_token naming the live badge and the roads out.
pub fn join(store: &Store, token: &str, identity: Option<String>) -> Result<JoinOutcome> {
    // Identity precedes consumption: a join that can bind nothing refuses
    // WITHOUT spending the token, so the retry (from a covered shell, or
    // with the env override) still finds it live.
    let Some(id_key) = identity else {
        bail!(
            "no identity reached this q process — the session hook injects QUARRY_AGENT/QUARRY_CHAT for shells in this repo, so run q join from such a shell. Outside hook coverage, skip join and export the badge by hand (QUARRY_DISPATCH=<item id>, from your dispatcher) — the env override survives exactly for that case."
        );
    };
    // The witness executor check (dc-mpg8), BEFORE consumption: the
    // authoring badge cannot join the item it authored, and the refusal
    // must leave the single-use token live for the right agent. Keys only —
    // a joined subagent legitimately inherits the dispatcher's session env,
    // so the session never bars the join road.
    if let Some(d) = crate::coord::dispatch_for_token(store, token) {
        witness_execution_check(store, &d.item, &id_key, None)?;
    }
    let bind = crate::coord::consume_join_token(store, token, &id_key)?;
    let (d, bound, rejoined) = match bind {
        crate::coord::JoinBind::Bound(d) => (d, Some(id_key.clone()), false),
        crate::coord::JoinBind::Rejoined(d) => (d, None, true),
    };
    // The store pin follows the bind (dc-g5x5): this identity's q acts and
    // hook-observed writes resolve to THIS store from here on, wherever cwd
    // sits — the session hook reads the pin to inject QUARRY_STORE into
    // badged shells, and hook processes read it directly. Recorded on
    // re-join too: an idempotent read that restores a lost pin.
    crate::store::pin_identity(&id_key, &store.root, &d.item);
    // The badge's model stamp reaches this process one act too late to ride
    // the hook (it-xcvb): the PreToolUse firing that injected QUARRY_ACTOR
    // into THIS shell ran before the bind existed, so the env still carries
    // whatever the hook could resolve without a badge. Every later shell
    // resolves the badge and gets it right; the join event is the whole of
    // the window, so it is stamped here from the badge directly. Agent-keyed
    // only, matching `coord::badge_model` — a chat- or session-keyed joiner
    // is a real chat whose own model is already recorded.
    let stamped = d
        .model
        .as_deref()
        .filter(|m| !m.trim().is_empty())
        .filter(|_| id_key.starts_with("agent:"))
        .map(crate::coord::safe_actor);
    // BENEATH THE STAMP, THE AGENT'S OWN RECORD, READ HERE (it-xwpw). Before
    // it-6ekf an unstamped arc genuinely inherited the dispatching chat's
    // model and the join said so; it no longer does, and repeating the old
    // sentence made the join state a falsehood to the one mind that could
    // check it. This is also the seat that can do better than the hook did:
    // the hook that injected QUARRY_ACTOR into this very shell was the arc's
    // FIRST fire, the one window where the agent's own turns may not have
    // reached disk yet and `agent_model` falls back to the sidecar's coarse
    // spawn alias (cl-jp4q's named grain). By the time this line composes,
    // that turn is on disk, so re-deriving here answers with the resolved
    // model where the injected value carries the alias — and the join event
    // below files under it.
    let recorded = match stamped {
        Some(_) => None,
        None => crate::coord::agent_actor_here(
            crate::coord::current_chat().as_deref(),
            id_key.strip_prefix("agent:"),
        ),
    };
    let (arc_actor, actor_source) = match (stamped, recorded) {
        (Some(m), _) => (m, ArcActorSource::Stamp),
        (None, Some(m)) => (m, ArcActorSource::Record),
        (None, None) => (Store::actor(), ArcActorSource::Injected),
    };
    if bound.is_some() {
        // Logged once, at the bind: the join is the arc's first badged act.
        let v = store
            .load_all()
            .ok()
            .and_then(|all| store.find(&all, &d.item).ok().map(|n| n.front.v))
            .unwrap_or(0);
        store.log_event(json!({
            "ts": Store::now(), "node": d.item, "v": v, "op": "join",
            "actor": arc_actor, "dispatch": d.item, "joined": id_key
        }))?;
    }
    // The brief opens with where the agent actually stands (it-rmqy): a
    // join whose working checkout differs from the store root says so before
    // a derived line renders — work where you stand; the graph half of the
    // arc lands canonically regardless.
    let rendered = crate::render::brief(store, &d.item)?;
    let brief = match fork_banner(store) {
        Some(banner) => format!("{}\n\n{}", banner, rendered),
        None => rendered,
    };
    // The brief IS a delivery of the item's areas' record (backdrop per
    // area), and attention rides the actor (dc-pwyd): record the badge's
    // area reads at this log position so the arc's first write doesn't
    // re-gate on ground the join just delivered. Badge-scoped — the
    // holding session's own watermarks stay exactly where they were
    // (it-csm3), and its first write into an area its own eyes never read
    // still gates.
    if let Ok(all) = store.load_all() {
        if let Ok(item) = store.find(&all, &d.item) {
            let reader = crate::coord::badge_attention_key(&d.item);
            for e in item.front.edges.iter().filter(|e| e.rel == "about") {
                if all.iter().any(|n| n.front.id == e.to && n.front.ty == "area") {
                    crate::coord::record_area_read(store, &reader, &e.to);
                }
            }
        }
    }
    Ok(JoinOutcome {
        item_id: d.item,
        item_title: d.item_title,
        bound,
        rejoined,
        brief,
        actor_source,
        arc_actor,
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
    // A path-backed doc re-affirms its own file — under `--to`, only when
    // the scope names that file (it-awhz). Unscoped, this rides along as
    // it always has; scoped, it used to restamp regardless of the filter,
    // so a `--to <some edge>` on a drifted doc restamped the doc's blob and
    // reported the count as if the named edge had moved. A scoped affirm
    // acts on its scope alone, or the count it returns cannot be read. The
    // homework line that advertises this restamp spells the target
    // `file:<path>` (render::homework, print_homework), so that is the
    // spelling the scope answers to.
    let mut new_blob = None;
    if node.front.ty == "doc" {
        if let (Some(p), Some(old)) = (&node.front.path, &node.front.blob) {
            let in_scope = match &only_to {
                None => true,
                Some(f) => f == &format!("file:{}", p),
            };
            if in_scope {
                if let Ok(nb) = store.blob(p) {
                    if &nb != old {
                        new_blob = Some(nb);
                        count += 1;
                    }
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

/// `q witness <item> --mark`: mark an EXISTING acceptance line as
/// witness-authored, transcribing the seat that authored it — the road for
/// lines that predate the pen (the three inaugural instances seed the
/// channel through it, dc-mpg8). Safe by direction: a mark only ADDS
/// review pressure; clearing is the user's act alone. The normal mark is
/// automatic at authoring; this road refuses a line the item does not
/// carry verbatim, and a line already marked. No version bump: a mark is
/// bookkeeping (the affirm rationale), loud in the log instead.
pub fn witness_mark(
    store: &Store,
    item_key: &str,
    line: &str,
    author_session: &str,
) -> Result<Node> {
    let all = store.load_all()?;
    let mut node = store.find(&all, item_key)?.clone();
    if node.front.ty != "item" {
        bail!(
            "{} is a {}, not an item — witness marks ride item acceptance lines",
            node.front.id, node.front.ty
        );
    }
    if !node.front.acceptance.iter().any(|a| a == line) {
        bail!(
            "no acceptance line on {} matches that text verbatim — the mark rides the exact line. What stands:\n  {}",
            crate::surface::atom_ref(&crate::surface::atom(&all, &node)),
            node.front.acceptance.join("\n  ")
        );
    }
    // Only authoring marks block a re-mark: a removal mark on the same text
    // records a different act (a since-removed twin), never this line.
    if node.front.witness.iter().any(|m| m.line == line && !m.removed) {
        bail!(
            "that line already carries a witness mark on {} — q witness {} shows the channel",
            node.front.id, node.front.id
        );
    }
    let kind = crate::coord::load_sessions(store)
        .get(author_session)
        .and_then(|p| p.kind.clone());
    node.front.witness.push(WitnessMark {
        line: line.to_string(),
        by: format!("session:{}", author_session),
        session: Some(author_session.to_string()),
        kind,
        date: Store::today(),
        ratified: None,
        removed: false,
    });
    store.save(&node)?;
    store.log_event(json!({
        "ts": Store::now(), "node": node.front.id, "v": node.front.v,
        "op": "witness-mark", "line": line, "author_session": author_session,
        "actor": Store::actor()
    }))?;
    Ok(node)
}

/// `q witness <item> --ratify --by user`: the user's ratification of the
/// item's witness-authored acceptance lines (dc-mpg8: review rides the
/// design wake UNTIL user-ratified or amended). Only the user's word
/// clears the flag — the q rule --by user channel: the agent transcribes,
/// never decides. Stamps every unratified mark; the mark stays as record.
/// No version bump: ratification is bookkeeping, never content (dc-2wes),
/// so citers never go behind over good news.
pub fn witness_ratify(store: &Store, item_key: &str, by: Option<&str>) -> Result<(Node, usize)> {
    if by != Some("user") {
        bail!(
            "witness pen (dc-mpg8): only the user's word clears a witness flag — re-run with --by user when the user has ratified the line(s), their word transcribed (the q rule --by user channel). The amend arm is the design seat's: q set <item> \"acceptance-=<line>\" \"acceptance+=<amended>\" retires or replaces the line (it-ds6b); an unratified line stays on the design wake's review channel until either word lands."
        );
    }
    let all = store.load_all()?;
    let mut node = store.find(&all, item_key)?.clone();
    let mut count = 0usize;
    for m in &mut node.front.witness {
        if m.ratified.is_none() {
            m.ratified = Some(Ratified { by: "user".into(), date: Store::today() });
            count += 1;
        }
    }
    if count == 0 {
        bail!(
            "{} carries no unratified witness marks — nothing to ratify (q witness {} shows the record)",
            crate::surface::atom_ref(&crate::surface::atom(&all, &node)),
            node.front.id
        );
    }
    store.save(&node)?;
    store.log_event(json!({
        "ts": Store::now(), "node": node.front.id, "v": node.front.v,
        "op": "witness-ratify", "restamped": count, "actor": Store::actor()
    }))?;
    Ok((node, count))
}
