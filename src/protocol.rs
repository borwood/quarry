//! The protocol layer: project-authored house rules as graph content.
//! A protocol entry is a doc node (kind=protocol) whose body is the
//! instruction and whose extra fields pick the trigger and delivery:
//!   on=<verb> · node_type=<type> · node_kind=<kind> · tier=inline|gate
//! inline rides the verb's confirmation output; gate intercepts the first
//! attempt per (reader, rule), delivers the context, saves the intent
//! under a one-time token, and executes on `q resume <token>`.
//!
//! The READER is the acting identity's attention key (dc-pwyd: attention
//! rides the actor), never the inherited env session — a gate answers
//! has-this-reader-seen-the-teach, and a joined agent's eyes are its own
//! (it-nngn, the it-csm3 fix shape applied to the protocol memo).

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
        .map(|n| (crate::surface::atom_ref(&crate::surface::atom(all, n)), n.body.clone()))
        .collect()
}

#[derive(Serialize, Deserialize, Default)]
struct IntentsFile {
    #[serde(default)]
    delivered: Vec<Delivered>,
    #[serde(default)]
    intents: Vec<StoredIntent>,
}

/// One spent memo: this READER has been shown this rule. The `session`
/// alias reads the pre-it-nngn rows, whose key was the env session name —
/// identical to the new key for any BOUND session, so every real memo
/// carries over; only the unbound fallback moved ("default" → "unbound",
/// unifying with coord::session_key), re-delivering once, machine-local.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
struct Delivered {
    #[serde(alias = "session")]
    reader: String,
    rule: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredIntent {
    pub token: String,
    pub verb: String,
    pub args: serde_json::Value,
    /// The attention key that saved this intent — the same reader the memo
    /// is spent by, so the saved act and its delivery name one identity.
    #[serde(alias = "session")]
    pub reader: String,
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

/// Machine-local one-time token mint (gate intents, join tokens): lowercase
/// unambiguous alphabet, nanos-seeded LCG — unguessable-enough for a
/// same-machine hand-off, never a cryptographic credential.
pub fn mint_token_n(len: usize) -> String {
    const AB: &[u8] = b"23456789abcdefghjkmnpqrstuvwxyz";
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    (0..len)
        .map(|_| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            AB[(seed >> 33) as usize % AB.len()] as char
        })
        .collect()
}

fn mint_token() -> String {
    mint_token_n(6)
}

pub struct Gate {
    pub rules: Vec<(String, String)>,
    pub token: String,
}

/// If any gate-tier protocol matches this act and has not yet been delivered
/// to this READER, save the intent, mark the rules delivered, and return
/// the gate brief. The caller prints it and exits without executing.
///
/// The reader is coord::attention_key (dc-pwyd, it-nngn): badge-scoped for
/// a joined arc, session otherwise. A joined agent tripping the gate spends
/// its OWN memo — it never marks the rule delivered to the holding session
/// whose eyes never saw the teach, and never inherits one that session
/// already consumed.
pub fn gate_if_needed(
    store: &Store,
    all: &[Node],
    verb: &str,
    node_type: Option<&str>,
    node_kind: Option<&str>,
    args: serde_json::Value,
) -> Result<Option<Gate>> {
    let reader = crate::coord::attention_key(store);
    let mut f = load(store);
    let pending: Vec<&Node> = matching(all, verb, node_type, node_kind)
        .into_iter()
        .filter(|n| tier(n) == "gate")
        .filter(|n| {
            !f.delivered.contains(&Delivered {
                reader: reader.clone(),
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
            reader: reader.clone(),
            rule: n.front.id.clone(),
        });
    }
    f.intents.push(StoredIntent {
        token: token.clone(),
        verb: verb.to_string(),
        args,
        reader,
        created: Store::now(),
    });
    save(store, &f)?;
    Ok(Some(Gate {
        rules: pending
            .into_iter()
            .map(|n| (crate::surface::atom_ref(&crate::surface::atom(all, n)), n.body.clone()))
            .collect(),
        token,
    }))
}

/// Save an intent outside the protocol-rule flow (engine-native gates, e.g.
/// the area-first-touch gate) and return its one-time token. Recorded under
/// the same reader the gate that raised it keys on — the area gate already
/// gates on coord::attention_key (it-csm3), so both halves name one identity.
pub fn save_intent(store: &Store, verb: &str, args: serde_json::Value) -> Result<String> {
    let reader = crate::coord::attention_key(store);
    let mut f = load(store);
    let token = mint_token();
    f.intents.push(StoredIntent {
        token: token.clone(),
        verb: verb.to_string(),
        args,
        reader,
        created: Store::now(),
    });
    save(store, &f)?;
    Ok(token)
}

/// Drop every memo a reader has spent — a badge's protocol deliveries die
/// with the badge (coord::clear_dispatch), the cl-b2z2 invariant applied to
/// this surface: a re-dispatched item's next agent meets the rule with its
/// own eyes rather than inheriting the replaced agent's consumption. Saved
/// intents are one-time tokens on a 1h TTL, not attention, and are left
/// alone: an orphaned token still resumes, and its rule re-gates.
pub fn clear_delivered(store: &Store, reader: &str) {
    let mut f = load(store);
    let before = f.delivered.len();
    f.delivered.retain(|d| d.reader != reader);
    if f.delivered.len() != before {
        let _ = save(store, &f);
    }
}

/// Consume a token: single-use, ~1h TTL. Expired or unknown tokens re-gate
/// gracefully (the memo already stands, so the plain retry executes).
///
/// The re-run advice speaks to the READER holding the token, never to "this
/// session" (it-nngn): the memo the retry rides is the acting identity's, so
/// a joined agent's retry executes on the agent's own spent delivery.
pub fn take_intent(store: &Store, token: &str) -> Result<StoredIntent> {
    let mut f = load(store);
    let Some(pos) = f.intents.iter().position(|i| i.token == token) else {
        bail!(
            "unknown token '{}' — if it expired, just re-run the original command (the protocol is already delivered to you, so it will execute)",
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
            "token '{}' expired (1h) — re-run the original command; the protocol is already delivered to you",
            token
        );
    }
    Ok(intent)
}
