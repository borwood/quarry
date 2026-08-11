//! Prose ids (dc-wwnk): node bodies cite other nodes by bare immutable id;
//! everything here DERIVES — the render-time unpack, the mention index, the
//! authoring echo. Nothing is stored: a mention references; an edge leans.
//!
//! The closed shape is \b(ar|it|th|dc|cl|do)-[a-z0-9]{4}\b with backticked
//! code spans skipped. Hyphenated prose produces false positives by design
//! (th-read, do-over): an id-shape resolving to nothing is a question, never
//! an error — render leaves it as written, wrap lints it, mint/edit ask.

use crate::model::Node;

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn prefix_at(b: &[u8], i: usize) -> bool {
    let p = [b[i], b[i + 1]];
    matches!(&p, b"ar" | b"it" | b"th" | b"dc" | b"cl" | b"do")
}

/// The one scanner every surface rides: walk the text with backtick parity
/// (odd segments are code spans, skipped), find each id-shape at word
/// boundaries, and let `f` decide its replacement (None keeps it verbatim).
fn scan_replace<F: FnMut(&str) -> Option<String>>(text: &str, mut f: F) -> String {
    let mut segs: Vec<String> = Vec::new();
    for (i, seg) in text.split('`').enumerate() {
        if i % 2 == 1 {
            segs.push(seg.to_string());
            continue;
        }
        let b = seg.as_bytes();
        let mut out = String::with_capacity(seg.len());
        let mut flushed = 0usize;
        let mut pos = 0usize;
        while pos + 7 <= b.len() {
            let candidate = (pos == 0 || !is_word(b[pos - 1]))
                && prefix_at(b, pos)
                && b[pos + 2] == b'-'
                && b[pos + 3..pos + 7]
                    .iter()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                && (pos + 7 == b.len() || !is_word(b[pos + 7]));
            if candidate {
                if let Some(r) = f(&seg[pos..pos + 7]) {
                    out.push_str(&seg[flushed..pos]);
                    out.push_str(&r);
                    flushed = pos + 7;
                }
                pos += 7;
            } else {
                pos += 1;
            }
        }
        out.push_str(&seg[flushed..]);
        segs.push(out);
    }
    segs.join("`")
}

/// Distinct id-shapes cited in a text, in first-appearance order — shapes,
/// not nodes: a dangling shape is the caller's question to ask.
pub fn cited_ids(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    scan_replace(text, |id| {
        if !out.iter().any(|x| x == id) {
            out.push(id.to_string());
        }
        None
    });
    out
}

/// The status decoration a target carries into its unpack label: dead
/// statuses and the archived flag ride along; live targets stay bare.
fn label(t: &Node) -> String {
    let mut l = t.front.ty.clone();
    if matches!(t.front.status.as_str(), "refuted" | "superseded" | "dropped") {
        l.push_str(", ");
        l.push_str(&t.front.status);
    }
    if t.front.archived {
        l.push_str(", archived");
    }
    l
}

/// Render-time unpack for text surfaces (open, brief): each resolvable bare
/// id expands to `id [type: `title`]`, dead/archived targets labeled;
/// unresolved shapes stay as written (wrap lints them).
pub fn unpack(all: &[Node], text: &str) -> String {
    scan_replace(text, |id| {
        all.iter()
            .find(|n| n.front.id == id)
            .map(|t| format!("{} [{}: `{}`]", id, label(t), t.front.title))
    })
}

fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render-time unpack for the HTML view: same expansion, hyperlinked to the
/// node anchor. Escapes the whole text first (ids are ASCII and survive;
/// entities never form id-shapes), then expands.
pub fn unpack_html(all: &[Node], text: &str) -> String {
    let escaped = esc_html(text);
    scan_replace(&escaped, |id| {
        all.iter().find(|n| n.front.id == id).map(|t| {
            format!(
                "<a href=\"#/n/{id}\">{id}</a> [{}: <code>{}</code>]",
                label(t),
                esc_html(&t.front.title)
            )
        })
    })
}

/// The derived mention index, reversed: nodes whose bodies cite `id` —
/// backlinks that were never stored. Blast and behind stay real-edge-only;
/// a mention references, it does not lean.
pub fn mentioned_by<'a>(all: &'a [Node], id: &str) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| n.front.id != id)
        .filter(|n| cited_ids(&n.body).iter().any(|c| c == id))
        .collect()
}

/// Id-shapes in live bodies resolving to nothing — wrap's lint. Each is a
/// citation to fix or hyphenated prose to leave; the reader judges.
pub fn danglers<'a>(all: &'a [Node]) -> Vec<(&'a Node, String)> {
    let mut out = Vec::new();
    for n in all.iter().filter(|n| !n.front.archived) {
        for id in cited_ids(&n.body) {
            if !all.iter().any(|t| t.front.id == id) {
                out.push((n, id));
            }
        }
    }
    out
}
