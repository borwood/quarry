//! The teaching surfaces: the embedded judgment-layer guide (`q guide`, and
//! the generated Claude skill), the C6 guard hook, and the installer that
//! distributes them into a host repo. One authority per fact: syntax lives in
//! --help, constraints live in error messages, judgment lives here, state
//! lives in the graph.

use anyhow::{anyhow, Result};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub const GUIDE: &str = r#"QUARRY — THE JUDGMENT LAYER
(mechanics live in `q --help` and `q <verb> --help`; this is when and why)

WHAT THIS IS
A work graph: decisions, claims, threads, items, docs, areas — typed nodes
with version-stamped edges, stored under graph/ in this repo. Files there
are never edited by hand (a hook denies it); every write goes through a q
verb, which bumps versions, stamps refs, and enforces the constraints.

SESSION SHAPE
Open with the three orientation queries: queue (threads awaiting the user,
answerable now), ready (items dispatchable now), shaping (upcoming work and
its blockers). Before designing anything, `q open` the areas and items it
touches — the neighborhood brief IS the context payload, and it shows what
changed behind every stale ref. Close a session by reviewing behind and
affirming ONLY what you actually re-read: affirm is a recorded act of
review, never a way to silence a marker.

PROVENANCE HONESTY
Your analyses and proposals are assistant-provenance — the default. Mark
user provenance or ratification only for what the user actually said or
approved, in their words. The tool refuses an assistant decision that
settles or supersedes a user-provenance node (C3). That refusal is not an
obstacle: it is the queue telling you the call belongs to the user.

CLAIMS — EXTRACT, NEVER MINT WHILE WRITING
Prose stays prose: journals, spike reports, and design notes are docs,
written normally and registered whole. A claim node exists only when
something depends on a statement or kills it — extract it at that moment,
with its source doc and its subjects. If you are not building on it or
refuting it, it is not a claim yet. The graph's population rate should
track rulings and reuse, not writing volume.

THREADS — ANYTHING THAT NEEDS THE USER
A thread is a strand needing user input: a pick between options, a
discussion, a topic that needs a spike before it can be answered. Queue it
rather than asking ad hoc; give it depends-on edges to any intermediate
work it spawns, and it will surface only when answerable. Record the user's
ruling with the rule verb; the thread resolves and its dependents unblock.

ITEMS — SKETCH EARLY, DERIVE BLOCKAGE
Upcoming work enters as sketch the moment it is anticipated, with its
expected relationships as ordinary edges, and is refined as threads
resolve. Ready is a stored intent; whether anything still blocks an item is
always derived — never write "blocked" anywhere.

REFS AND STALENESS
Cite nodes and files with edges; stamps are automatic. behind is
information, not noise: severity 1 means something you cite was refuted or
superseded — read it before building further. When a claim falls, blast
enumerates everything leaning on it. That list IS the correction; there is
no sweep.

EDGE MATRIX (names only)
about (anything → area or file) · part-of (hierarchy) · depends-on
(item/thread → item/thread/decision) · settles (decision → thread) ·
supports (claim/doc → decision/item/claim) · refutes (claim/doc →
claim/decision) · supersedes (same type) · source (claim → doc)

ENVIRONMENT
Set QUARRY_ACTOR to your model/agent name so provenance derivation and the
event log stay honest.
"#;

const SKILL_FRONT: &str = "---\nname: quarry\ndescription: The work graph in this repo's graph/ directory — decisions, claims, threads, items, docs. Use at session start to get oriented (q query queue / ready / shaping), before design work (q open the relevant nodes), when recording a user ruling, extracting a claim, queueing a thread for the user, or closing a session (review behind, affirm what you re-read). All graph writes go through q verbs, never file edits.\n---\n\n";

/// C6, as a PreToolUse hook. Returns Some(denial) if the tool call should be
/// blocked, None to allow. Input is the hook's stdin JSON.
pub fn guard(input: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(input).ok()?;
    let tool = v.get("tool_name")?.as_str()?;
    if !matches!(tool, "Write" | "Edit" | "NotebookEdit") {
        return None;
    }
    let ti = v.get("tool_input")?;
    let path = ti
        .get("file_path")
        .or_else(|| ti.get("notebook_path"))?
        .as_str()?;
    let p = path.replace('\\', "/").to_lowercase();
    if p.contains("graph/nodes/") || p.contains("graph/log/") {
        Some(
            "C6: files under graph/ are the work graph and are written only through the q verbs — \
             a verb bumps the node's version, stamps every ref, and enforces the constraints; a \
             hand edit silently bypasses all three. Use q new / q set / q edit / q link / q rule / \
             q claim / q refute / q affirm instead (reading is unrestricted; `q open <node>` \
             renders the brief). Run `q guide` for the judgment layer."
                .into(),
        )
    } else {
        None
    }
}

/// Write the generated skill and wire the guard hook into the host repo's
/// .claude/settings.json. Returns a human summary of what happened.
pub fn install_claude(root: &Path) -> Result<Vec<String>> {
    let mut actions = Vec::new();

    let skill_dir = root.join(".claude").join("skills").join("quarry");
    fs::create_dir_all(&skill_dir)?;
    let skill = format!(
        "{}{}\n\n---\nGenerated by `q init --claude` (quarry v{}). Do not hand-edit — re-run \
         after upgrading the tool. Mechanics live in `q --help`.\n",
        SKILL_FRONT,
        GUIDE.trim_end(),
        env!("CARGO_PKG_VERSION")
    );
    fs::write(skill_dir.join("SKILL.md"), skill)?;
    actions.push(".claude/skills/quarry/SKILL.md written".into());

    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("q"));
    let cmd = format!("\"{}\" hook guard", exe.to_string_lossy().replace('\\', "/"));
    let settings_path = root.join(".claude").join("settings.json");
    let mut settings: serde_json::Value = if settings_path.exists() {
        serde_json::from_str(&fs::read_to_string(&settings_path)?)
            .map_err(|e| anyhow!(".claude/settings.json is not valid JSON: {}", e))?
    } else {
        json!({})
    };
    let obj = settings
        .as_object_mut()
        .ok_or_else(|| anyhow!(".claude/settings.json is not a JSON object"))?;
    let hooks = obj.entry("hooks").or_insert_with(|| json!({}));
    let pre = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow!("settings 'hooks' is not an object"))?
        .entry("PreToolUse")
        .or_insert_with(|| json!([]));
    let arr = pre
        .as_array_mut()
        .ok_or_else(|| anyhow!("settings 'hooks.PreToolUse' is not an array"))?;
    let already = arr
        .iter()
        .any(|e| serde_json::to_string(e).unwrap_or_default().contains("hook guard"));
    if already {
        actions.push("guard hook already present in .claude/settings.json".into());
    } else {
        arr.push(json!({
            "matcher": "Write|Edit|NotebookEdit",
            "hooks": [{
                "type": "command",
                "command": cmd,
                "timeout": 10,
                "statusMessage": "quarry: guarding graph/"
            }]
        }));
        fs::write(&settings_path, serde_json::to_string_pretty(&settings)? + "\n")?;
        actions.push(format!(".claude/settings.json: PreToolUse guard added ({})", cmd));
    }
    Ok(actions)
}
