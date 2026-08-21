//! The worktree proof (dc-g5x5, it-s789): a worktree dispatch exercises the
//! store-pin chain end to end — dispatch stamps the canonical root into the
//! spawn line, join binds and pins, badged mints and hook-observed writes
//! land at the canonical store while cwd sits in the fork, blob stamps hash
//! the fork's content against store-relative paths, and harvest clears the
//! pin. The fork's own graph/ copy is never written. This test IS the
//! instrument: it spawns the real binary with cwd in the fork, exactly the
//! shape a worktree-isolated agent runs in.

use quarry::model::At;
use quarry::ops::{self, NewArgs};
use quarry::store::Store;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git").current_dir(dir).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed: {}{}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_blob12(dir: &Path, rel: &str) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["hash-object", "--", rel])
        .output()
        .unwrap();
    assert!(out.status.success(), "git hash-object failed");
    String::from_utf8_lossy(&out.stdout).trim().chars().take(12).collect()
}

/// Spawn the real q binary: env scrubbed of every QUARRY_* the harness or a
/// dev shell might carry, then the pin-file home and per-call env applied —
/// env transport is per child process, so the threaded suite never sets
/// these vars in-process.
fn q_cmd(qhome: &Path, dir: &Path, envs: &[(&str, &str)], args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_q"));
    c.current_dir(dir)
        .env_remove("QUARRY_SESSION")
        .env_remove("QUARRY_DISPATCH")
        .env_remove("QUARRY_CHAT")
        .env_remove("QUARRY_AGENT")
        .env_remove("QUARRY_STORE")
        .env_remove("QUARRY_HOME")
        .env("QUARRY_HOME", qhome)
        .env("QUARRY_ACTOR", "proof-actor")
        .args(args);
    for (k, v) in envs {
        c.env(k, v);
    }
    c
}

fn run(qhome: &Path, dir: &Path, envs: &[(&str, &str)], args: &[&str]) -> String {
    let out = q_cmd(qhome, dir, envs, args).output().unwrap();
    assert!(
        out.status.success(),
        "q {:?} failed: {}{}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn run_hook(
    qhome: &Path,
    dir: &Path,
    envs: &[(&str, &str)],
    args: &[&str],
    input: &str,
) -> (Option<i32>, String, String) {
    let mut child = q_cmd(qhome, dir, envs, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// Recursive (relative path, content) snapshot — the fork-graph inertness
/// witness.
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for e in entries.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
                out.push((rel, fs::read(&p).unwrap_or_default()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Case-folded, separator-normalized — the same posture store::store_relative
/// takes: a Windows path round-tripped through the OS current-directory comes
/// back with its own idea of case and separators.
fn norm(s: &str) -> String {
    s.to_lowercase().replace('\\', "/")
}

fn md_count(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn the_store_pin_carries_a_worktree_dispatch_end_to_end() {
    // In-process hygiene for the library-call setup below (the spawned
    // binaries scrub their own env in q_cmd).
    std::env::set_var("QUARRY_ACTOR", "test-user");
    for v in ["QUARRY_SESSION", "QUARRY_DISPATCH", "QUARRY_CHAT", "QUARRY_AGENT", "QUARRY_STORE"] {
        std::env::remove_var(v);
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("quarry-wtproof-{}-{}", std::process::id(), nanos));
    let canon = base.join("canon");
    let fork = base.join("fork");
    let qhome = base.join("qhome");
    fs::create_dir_all(&canon).unwrap();
    fs::create_dir_all(&qhome).unwrap();

    // ── the canonical store: an area, a ready item with acceptance, a src
    // file — committed, then forked as a git worktree.
    let cs = Store::init(&canon).unwrap();
    let area = ops::new_node(&cs, NewArgs::bare("area", "proof area")).unwrap();
    let mut ia = NewArgs::bare("item", "the store pin proof slice");
    ia.kind = Some("slice".into());
    ia.acceptance = vec!["acts land at the canonical store".into()];
    ia.about = vec![area.front.id.clone()];
    let item = ops::new_node(&cs, ia).unwrap();
    ops::set(&cs, &item.front.id, &["status=ready".to_string()], None).unwrap();
    fs::create_dir_all(canon.join("src")).unwrap();
    fs::write(canon.join("src").join("lib.rs"), "pub fn canon() {}\n").unwrap();
    git(&canon, &["init", "-q"]);
    git(&canon, &["config", "user.email", "proof@test"]);
    git(&canon, &["config", "user.name", "proof"]);
    git(&canon, &["add", "-A"]);
    git(&canon, &["commit", "-q", "-m", "canon baseline"]);
    git(&canon, &["worktree", "add", "-q", fork.to_str().unwrap()]);
    assert!(fork.join("graph").join("nodes").is_dir(), "the fork carries a graph/ checkout");
    let fork_graph_before = snapshot(&fork.join("graph"));

    let canon_s = canon.display().to_string();
    let item_id = item.front.id.clone();

    // ── dispatch stamps the canonical root into the spawn line beside the
    // token (the badge-pin outcome, dispatcher side).
    let out = run(
        &qhome,
        &canon,
        &[("QUARRY_SESSION", "wt-dispatcher")],
        &["dispatch", &item_id, "--files", "src/**", "--solo"],
    );
    let marker = "run: q join ";
    let pos = out.find(marker).expect("spawn line present");
    let token: String = out[pos + marker.len()..]
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();
    assert!(
        out.contains(&format!("--store {}", canon_s)),
        "the spawn line stamps the canonical root beside the token: {}",
        out
    );
    // ── and it names NO working directory (it-rmqy): dispatch cannot know
    // where the harness will sit the agent, so an "in <dir>," clause could
    // only send a fork-isolated agent out of its isolation. Followed
    // verbatim the line must be correct from canon and from a fork alike.
    let spawn_line = out.lines().find(|l| l.contains(marker)).unwrap().trim();
    let (before_cmd, _) = spawn_line.split_once(marker).unwrap();
    assert!(
        !before_cmd.contains(&canon_s) && !before_cmd.contains(','),
        "no directing clause stands between the announcement and the command: {:?}",
        before_cmd
    );
    let pin_spent = spawn_line.replacen(&format!("--store {}", canon_s), "--store <pin>", 1);
    assert!(
        !pin_spent.contains(&canon_s),
        "the --store pin is the ONLY directory the spawn line names: {}",
        spawn_line
    );

    // ── the fork diverges: the agent's edit exists only in the worktree.
    fs::write(fork.join("src").join("lib.rs"), "pub fn fork_edit() {}\n").unwrap();
    let fork_blob = git_blob12(&fork, "src/lib.rs");
    let canon_blob = git_blob12(&canon, "src/lib.rs");
    assert_ne!(fork_blob, canon_blob, "fork content diverged from canon");

    // ── join from the fork, cwd in the fork: consumes the token at the
    // pinned store, binds the agent, plants the pin.
    let out = run(
        &qhome,
        &fork,
        &[("QUARRY_AGENT", "wt-agent-1")],
        &["join", &token, "--store", &canon_s],
    );
    assert!(out.contains("joined:"), "join bound the identity: {}", out);
    // ── the where-you-stand banner OPENS the brief (it-rmqy): the agent is
    // told where it stands before it reads a line of the work, and the
    // banner is the one place the split is stated (the redundant paragraph
    // that used to precede it is gone — a fork join said it twice).
    let banner_at = out
        .find("WHERE YOU STAND:")
        .unwrap_or_else(|| panic!("a fork join opens with the where-you-stand banner: {}", out));
    let brief_at = out.find("DISPATCH BRIEF").expect("the brief renders");
    assert!(
        banner_at < brief_at,
        "the banner opens the brief rather than trailing it: {}",
        out
    );
    let banner = &out[banner_at..brief_at];
    assert!(
        norm(banner).contains(&norm(&fork.display().to_string()))
            && banner.contains(&canon_s),
        "the banner names both roots — the fork it stands in and the graph its acts land at: {}",
        banner
    );
    let mut speaking = out.lines().filter(|l| !l.trim().is_empty());
    assert!(
        speaking.next().unwrap_or_default().contains("joined:"),
        "the bind is announced first: {}",
        out
    );
    assert!(
        speaking.next().unwrap_or_default().starts_with("WHERE YOU STAND:"),
        "the banner is the very next thing the join says — nothing stands between the bind and the orientation it opens the brief with: {}",
        out
    );
    assert_eq!(
        out.matches("WHERE YOU STAND:").count(),
        1,
        "the split is stated once — never a second paragraph saying the same thing: {}",
        out
    );
    let pins: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(qhome.join(".quarry").join("store-pins.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        pins["agent:wt-agent-1"]["root"].as_str(),
        Some(canon_s.as_str()),
        "the pin records identity → canonical root"
    );

    // ── a badged mint from the fork, resolved through the IDENTITY pin
    // alone (no QUARRY_STORE env — the hook-process road): the claim lands
    // in the canonical store, stamped with the badge, its source blob
    // hashing the FORK's content against the store-relative path.
    run(
        &qhome,
        &fork,
        &[("QUARRY_AGENT", "wt-agent-1")],
        &[
            "claim",
            "`store-pin`: badged acts land at the pinned graph",
            "--about",
            &area.front.id,
            "--source",
            "file:src/lib.rs",
            "--kind",
            "vein",
        ],
    );
    assert_eq!(md_count(&canon.join("graph").join("nodes").join("claim")), 1, "claim in canon");
    assert_eq!(md_count(&fork.join("graph").join("nodes").join("claim")), 0, "no claim in fork");
    let all = cs.load_all().unwrap();
    let claim = all.iter().find(|n| n.front.ty == "claim").unwrap();
    let src_edge = claim
        .front
        .edges
        .iter()
        .find(|e| e.rel == "source" && e.to == "file:src/lib.rs")
        .expect("source edge on the mint");
    assert_eq!(
        src_edge.at,
        At::Blob(fork_blob.clone()),
        "the blob stamp hashes the touched worktree file, not the canon copy"
    );
    let log = cs.read_log().unwrap();
    let mint_ev = log
        .iter()
        .rev()
        .find(|ev| ev.get("op").and_then(|v| v.as_str()) == Some("create"))
        .unwrap();
    assert_eq!(
        mint_ev.get("dispatch").and_then(|v| v.as_str()),
        Some(item_id.as_str()),
        "the mint stamps the badge through the joined association"
    );

    // ── the env-pin road (the injected-shell shape): a write verb with
    // QUARRY_STORE set lands canonically from the fork's cwd.
    run(
        &qhome,
        &fork,
        &[("QUARRY_STORE", &canon_s)],
        &["new", "thread", "the env pin road probe"],
    );
    assert_eq!(md_count(&canon.join("graph").join("nodes").join("thread")), 1, "thread in canon");
    assert_eq!(md_count(&fork.join("graph").join("nodes").join("thread")), 0, "no thread in fork");

    // ── the hook-observed write: the guard process (whose env the shell
    // injection never reaches) resolves the store from the input's identity
    // pin, judges the fork-relative path as the store-relative contract
    // path, and accrues the touch canonically.
    let write_input = |path: &Path| {
        serde_json::json!({
            "tool_name": "Write",
            "tool_input": {"file_path": path.display().to_string()},
            "session_id": "parent-chat-1",
            "agent_id": "wt-agent-1",
            "cwd": fork.display().to_string()
        })
        .to_string()
    };
    let (code, _o, _e) = run_hook(
        &qhome,
        &fork,
        &[],
        &["hook", "guard"],
        &write_input(&fork.join("src").join("lib.rs")),
    );
    assert_eq!(code, Some(0), "an in-write-set fork write is allowed");
    let touched = fs::read_to_string(canon.join("graph").join(".touched.jsonl")).unwrap();
    assert!(
        touched.contains(&format!("item:{}", item_id)) && touched.contains("src/lib.rs"),
        "the observed write accrues at the canonical store under the badge: {}",
        touched
    );
    assert!(
        !fork.join("graph").join(".touched.jsonl").exists(),
        "nothing accrues into the fork's graph copy"
    );
    // Outside the leased write-set: the contract holds across the boundary.
    let (code, _o, err) = run_hook(
        &qhome,
        &fork,
        &[],
        &["hook", "guard"],
        &write_input(&fork.join("docs").join("notes.md")),
    );
    assert_eq!(code, Some(2), "an out-of-write-set fork write denies: {}", err);
    // Freehand graph edits deny in the fork exactly as in the canon (C6).
    let (code, _o, _e) = run_hook(
        &qhome,
        &fork,
        &[],
        &["hook", "guard"],
        &write_input(&fork.join("graph").join("nodes").join("item").join("x.md")),
    );
    assert_eq!(code, Some(2), "the fork's graph copy refuses freehand edits");

    // ── the session hook injects the pin into badged shells (launcher env
    // winning), so every q act in the shell resolves the canonical store.
    let bash_input = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": {"command": "q query ready"},
        "session_id": "parent-chat-1",
        "agent_id": "wt-agent-1",
        "cwd": fork.display().to_string()
    })
    .to_string();
    let (code, out, _e) = run_hook(&qhome, &fork, &[], &["hook", "session"], &bash_input);
    assert_eq!(code, Some(0));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("hook envelope");
    let cmd = v["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert!(
        cmd.contains(&format!("export QUARRY_STORE='{}'", canon_s)),
        "the pin rides the identity injection channel: {}",
        cmd
    );
    let (_c, out, _e) = run_hook(
        &qhome,
        &fork,
        &[("QUARRY_STORE", &canon_s)],
        &["hook", "session"],
        &bash_input,
    );
    if let Some(cmd) = serde_json::from_str::<serde_json::Value>(out.trim())
        .ok()
        .and_then(|v| v["hookSpecificOutput"]["updatedInput"]["command"].as_str().map(String::from))
    {
        assert!(!cmd.contains("QUARRY_STORE"), "launcher env wins — no re-injection: {}", cmd);
    }

    // ── harvest by the dispatcher's hand, in the canon: the badge clears
    // and takes its pin with it.
    run(&qhome, &canon, &[("QUARRY_SESSION", "wt-dispatcher")], &["harvest", &item_id]);
    let pins_after = fs::read_to_string(qhome.join(".quarry").join("store-pins.json"))
        .unwrap_or_default();
    assert!(
        !pins_after.contains("agent:wt-agent-1"),
        "harvest clears the pin with the badge: {}",
        pins_after
    );
    assert!(
        !canon.join("graph").join(".dispatch.json").exists(),
        "the held entry and associations cleared"
    );

    // ── the fork's graph copy sat inert through the whole arc.
    let fork_graph_after = snapshot(&fork.join("graph"));
    assert_eq!(
        fork_graph_before, fork_graph_after,
        "the worktree's own graph/ copy is never written"
    );

    // Cleanup (best-effort; the worktree registration dies with the temp dir).
    let _ = Command::new("git")
        .current_dir(&canon)
        .args(["worktree", "remove", "--force", fork.to_str().unwrap()])
        .output();
    let _ = fs::remove_dir_all(&base);
}

/// The negative half of the where-you-stand banner (it-rmqy): a join whose
/// working checkout IS the tree the graph lives in says nothing about where
/// it stands — from the store root and from a subdirectory of it alike, so
/// the walk-up that finds the graph from a nested cwd never fakes a fork.
#[test]
fn a_canonical_join_says_nothing_about_where_it_stands() {
    std::env::set_var("QUARRY_ACTOR", "test-user");
    for v in ["QUARRY_SESSION", "QUARRY_DISPATCH", "QUARRY_CHAT", "QUARRY_AGENT", "QUARRY_STORE"] {
        std::env::remove_var(v);
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base =
        std::env::temp_dir().join(format!("quarry-canonjoin-{}-{}", std::process::id(), nanos));
    let canon = base.join("canon");
    let qhome = base.join("qhome");
    fs::create_dir_all(&canon).unwrap();
    fs::create_dir_all(&qhome).unwrap();
    fs::create_dir_all(canon.join("src")).unwrap();

    let cs = Store::init(&canon).unwrap();
    let area = ops::new_node(&cs, NewArgs::bare("area", "canon join area")).unwrap();
    let mut ia = NewArgs::bare("item", "the canonical join slice");
    ia.kind = Some("slice".into());
    ia.acceptance = vec!["the join opens with the work, not with directions".into()];
    ia.about = vec![area.front.id.clone()];
    let item = ops::new_node(&cs, ia).unwrap();
    ops::set(&cs, &item.front.id, &["status=ready".to_string()], None).unwrap();
    let canon_s = canon.display().to_string();
    let item_id = item.front.id.clone();

    let out = run(
        &qhome,
        &canon,
        &[("QUARRY_SESSION", "canon-dispatcher")],
        &["dispatch", &item_id, "--files", "src/**", "--solo"],
    );
    let marker = "run: q join ";
    let pos = out.find(marker).expect("spawn line present");
    let token: String = out[pos + marker.len()..]
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    // ── the join at the canonical root: work root and store root are one.
    let out = run(
        &qhome,
        &canon,
        &[("QUARRY_AGENT", "canon-agent-1")],
        &["join", &token, "--store", &canon_s],
    );
    assert!(out.contains("joined:"), "the canonical join bound: {}", out);
    assert!(
        !out.contains("WHERE YOU STAND"),
        "an agent standing in the graph's own tree is told nothing about where it stands: {}",
        out
    );

    // ── and from a SUBDIRECTORY: discovery walks up to the same root, so
    // the split the banner announces does not exist here either.
    let out = run(
        &qhome,
        &canon.join("src"),
        &[("QUARRY_AGENT", "canon-agent-1")],
        &["join", &token, "--store", &canon_s],
    );
    assert!(out.contains("already joined:"), "the re-join is an idempotent read: {}", out);
    assert!(
        !out.contains("WHERE YOU STAND"),
        "a nested cwd inside the store is not a fork — no false banner: {}",
        out
    );

    let _ = fs::remove_dir_all(&base);
}

/// The pin refuses loudly when it names a non-store: a stamped-but-moved
/// graph must never silently fall back to a fork's cwd discovery.
#[test]
fn a_dead_pin_refuses_instead_of_scattering() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base =
        std::env::temp_dir().join(format!("quarry-deadpin-{}-{}", std::process::id(), nanos));
    let repo = base.join("repo");
    let qhome = base.join("qhome");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&qhome).unwrap();
    Store::init(&repo).unwrap();
    let ghost: PathBuf = base.join("ghost");
    let out = q_cmd(
        &qhome,
        &repo,
        &[("QUARRY_STORE", ghost.to_str().unwrap())],
        &["query", "ready"],
    )
    .output()
    .unwrap();
    assert!(!out.status.success(), "a dead env pin is a loud error, not a fallback");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("QUARRY_STORE"), "the refusal names the pin: {}", err);
    let _ = fs::remove_dir_all(&base);
}
