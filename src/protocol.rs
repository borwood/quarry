//! The protocol layer: project-authored house rules as graph content.
//! A protocol entry is a doc node (kind=protocol) whose body is the
//! instruction and whose extra fields pick the trigger and delivery:
//!   on=<verb> · node_type=<type> · node_kind=<kind> · tier=inline|gate
//! inline rides the verb's confirmation output; gate intercepts the first
//! attempt per (session, rule), delivers the context, saves the intent
//! under a one-time token, and executes on `q resume <token>`.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::model::Node;
use crate::store::Store;

fn field<'a>(n: &'a Node, k: &str) -> Option<&'a str> {
    n.front.extra.get(k).and_then(|v| v.as_str())
}

pub fn tier(n: &Node) -> &str {
    field(n, "tier").unwrap_or("inline")
}

/// Protocol entries whose trigger matches this act.
pub fn matching<'a>(
    all: &'a [Node],
    verb: &str,
    node_type: Option<&str>,
    node_kind: Option<&str>,
) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| {
            n.front.ty == "doc"
                && n.front.kind.as_deref() == Some("protocol")
                && n.front.status != "superseded"
        })
        .filter(|n| field(n, "on") == Some(verb))
        .filter(|n| field(n, "node_type").map_or(true, |t| Some(t) == node_type))
        .filter(|n| field(n, "node_kind").map_or(true, |t| Some(t) == node_kind))
        .collect()
}

pub fn inline_texts(
    all: &[Node],
    verb: &str,
    node_type: Option<&str>,
    node_kind: Option<&str>,
) -> Vec<(String, String)> {
    matching(all, verb, node_type, node_kind)
        .into_iter()
        .filter(|n| tier(n) == "inline")
        .map(|n| (n.front.title.clone(), n.body.clone()))
        .collect()
}

#[derive(Serialize, Deserialize, Default)]
struct IntentsFile {
    #[serde(default)]
    delivered: Vec<Delivered>,
    #[serde(default)]
    intents: Vec<StoredIntent>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
struct Delivered {
    session: String,
    rule: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredIntent {
    pub token: String,
    pub verb: String,
    pub args: serde_json::Value,
    pub session: String,
    pub created: String,
}

fn intents_path(store: &Store) -> PathBuf {
    store.root.join("graph").join(".intents.json")
}

fn load(store: &Store) -> IntentsFile {
    fs::read_to_string(intents_path(store))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(store: &Store, f: &IntentsFile) -> Result<()> {
    fs::write(intents_path(store), serde_json::to_string_pretty(f)? + "\n")?;
    Ok(())
}

fn mint_token() -> String {
    const AB: &[u8] = b"23456789abcdefghjkmnpqrstuvwxyz";
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    (0..6)
        .map(|_| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            AB[(seed >> 33) as usize % AB.len()] as char
        })
        .collect()
}

pub struct Gate {
    pub rules: Vec<(String, String)>,
    pub token: String,
}

/// If any gate-tier protocol matches this act and has not yet been delivered
/// to this session, save the intent, mark the rules delivered, and return
/// the gate brief. The caller prints it and exits without executing.
pub fn gate_if_needed(
    store: &Store,
    all: &[Node],
    verb: &str,
    node_type: Option<&str>,
    node_kind: Option<&str>,
    args: serde_json::Value,
) -> Result<Option<Gate>> {
    let sess = crate::coord::current_session().unwrap_or_else(|| "default".into());
    let mut f = load(store);
    let pending: Vec<&Node> = matching(all, verb, node_type, node_kind)
        .into_iter()
        .filter(|n| tier(n) == "gate")
        .filter(|n| {
            !f.delivered.contains(&Delivered {
                session: sess.clone(),
                rule: n.front.id.clone(),
            })
        })
        .collect();
    if pending.is_empty() {
        return Ok(None);
    }
    let token = mint_token();
    for n in &pending {
        f.delivered.push(Delivered {
            session: sess.clone(),
            rule: n.front.id.clone(),
        });
    }
    f.intents.push(StoredIntent {
        token: token.clone(),
        verb: verb.to_string(),
        args,
        session: sess,
        created: Store::now(),
    });
    save(store, &f)?;
    Ok(Some(Gate {
        rules: pending
            .into_iter()
            .map(|n| (n.front.title.clone(), n.body.clone()))
            .collect(),
        token,
    }))
}

/// Consume a token: single-use, ~1h TTL. Expired or unknown tokens re-gate
/// gracefully (the memo already stands, so the plain retry executes).
pub fn take_intent(store: &Store, token: &str) -> Result<StoredIntent> {
    let mut f = load(store);
    let Some(pos) = f.intents.iter().position(|i| i.token == token) else {
        bail!(
            "unknown token '{}' — if it expired, just re-run the original command (the protocol is already delivered to this session, so it will execute)",
            token
        );
    };
    let intent = f.intents.remove(pos);
    save(store, &f)?;
    let fresh = {
        use time::format_description::well_known::Rfc3339;
        let cutoff = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
        intent.created > cutoff.format(&Rfc3339).unwrap_or_default()
    };
    if !fresh {
        bail!(
            "token '{}' expired (1h) — re-run the original command; the protocol is already delivered to this session",
            token
        );
    }
    Ok(intent)
}
