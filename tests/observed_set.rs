//! The observed-set instrument (it-bj3b): a badged write must never escape
//! accounting. Three misses in the record (it-hapc 2026-08-14; it-swsy and
//! it-pgn9 2026-08-17) showed harvest reporting tests/** leased-but-untouched
//! while tests/basic.rs was genuinely modified under the badge. This harness
//! spawns the real binary through the hook seam — exactly the shape a tool
//! write arrives in — and pins: (1) Windows absolute tool-write paths in
//! mixed case and mixed separators land store-relative in the observed set;
//! (2) a badged write whose path fails store-relative resolution is recorded
//! raw and rendered at harvest, never silently discarded; (3) shell-made
//! writes parse through the session hook best-effort; (4) harvest states the
//! sight boundary.

use quarry::ops::{self, NewArgs};
use quarry::store::Store;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

/// Spawn the real q binary: env scrubbed of every QUARRY_* the harness or a
/// dev shell might carry, then the pin-file home and per-call env applied.
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
        .env("QUARRY_ACTOR", "obs-actor")
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
    args: &[&str],
    input: &str,
) -> (Option<i32>, String, String) {
    let mut child = q_cmd(qhome, dir, &[], args)
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

#[test]
fn badged_writes_land_or_are_recorded_never_dropped() {
    // In-process hygiene for the library-call setup (the spawned binaries
    // scrub their own env in q_cmd).
    std::env::set_var("QUARRY_ACTOR", "test-user");
    for v in ["QUARRY_SESSION", "QUARRY_DISPATCH", "QUARRY_CHAT", "QUARRY_AGENT", "QUARRY_STORE"] {
        std::env::remove_var(v);
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("quarry-obs-{}-{}", std::process::id(), nanos));
    let canon = base.join("canon");
    let qhome = base.join("qhome");
    fs::create_dir_all(&canon).unwrap();
    fs::create_dir_all(&qhome).unwrap();

    let cs = Store::init(&canon).unwrap();
    let area = ops::new_node(&cs, NewArgs::bare("area", "obs area")).unwrap();
    let mut ia = NewArgs::bare("item", "the observed set proof slice");
    ia.kind = Some("bug".into());
    ia.acceptance = vec!["every badged write lands or is recorded".into()];
    ia.about = vec![area.front.id.clone()];
    let item = ops::new_node(&cs, ia).unwrap();
    ops::set(&cs, &item.front.id, &["status=ready".to_string()], None).unwrap();
    fs::create_dir_all(canon.join("src")).unwrap();
    fs::write(canon.join("src").join("lib.rs"), "pub fn obs() {}\n").unwrap();
    let canon_s = canon.display().to_string();
    let item_id = item.front.id.clone();

    // ── dispatch and join: the badge binds agent obs-agent-1.
    let out = run(
        &qhome,
        &canon,
        &[("QUARRY_SESSION", "obs-dispatcher")],
        &["dispatch", &item_id, "--files", "src/**", "--files", "tests/**", "--solo"],
    );
    let marker = "run: q join ";
    let pos = out.find(marker).expect("spawn line present");
    let token: String =
        out[pos + marker.len()..].split_whitespace().next().unwrap().to_string();
    run(
        &qhome,
        &canon,
        &[("QUARRY_AGENT", "obs-agent-1")],
        &["join", &token, "--store", &canon_s],
    );

    let write_input = |path: &str| {
        serde_json::json!({
            "tool_name": "Edit",
            "tool_input": {"file_path": path},
            "session_id": "obs-chat-1",
            "agent_id": "obs-agent-1",
            "cwd": canon.display().to_string()
        })
        .to_string()
    };

    // ── (1) THE REGRESSION INSTRUMENT: a Windows-shaped absolute tool-write
    // path — every component case-mangled, separators mixed — resolves
    // store-relative and lands in the observed set under the badge.
    let mangled = format!("{}\\SRC/Lib.rs", canon_s.to_uppercase());
    let (code, _o, e) = run_hook(&qhome, &canon, &["hook", "guard"], &write_input(&mangled));
    assert_eq!(code, Some(0), "an in-write-set mangled path is allowed: {}", e);
    let touched = fs::read_to_string(canon.join("graph").join(".touched.jsonl")).unwrap();
    assert!(
        touched.contains(&format!("item:{}", item_id)) && touched.contains("src/lib.rs"),
        "the mangled tool write accrued store-relative under the badge: {}",
        touched
    );
    assert!(
        !touched.contains("unresolved"),
        "a resolvable path never records as unresolved: {}",
        touched
    );

    // ── (2) NO SILENT DISCARD: a badged tool write whose path resolves
    // nowhere (outside the store) is recorded raw, marked unresolved.
    let stray = base.join("elsewhere").join("checkout").join("src").join("teach.rs");
    let stray_s = stray.display().to_string();
    let (code, _o, _e) = run_hook(&qhome, &canon, &["hook", "guard"], &write_input(&stray_s));
    assert_eq!(code, Some(0), "an out-of-store write stays allowed — observed, never denied");
    let touched = fs::read_to_string(canon.join("graph").join(".touched.jsonl")).unwrap();
    assert!(
        touched.contains("\"unresolved\":true"),
        "the resolution failure is recorded, never dropped: {}",
        touched
    );

    // ── (3) THE SHELL CHANNEL: the incident shape — a heredoc write to
    // tests/basic.rs — parses through the session hook and accrues marked.
    let bash_input = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": {"command": "cat > tests/basic.rs <<'EOF'\nfn t() {}\nEOF"},
        "session_id": "obs-chat-1",
        "agent_id": "obs-agent-1",
        "cwd": canon.display().to_string()
    })
    .to_string();
    let (code, _o, _e) = run_hook(&qhome, &canon, &["hook", "session"], &bash_input);
    assert_eq!(code, Some(0));
    let touched = fs::read_to_string(canon.join("graph").join(".touched.jsonl")).unwrap();
    assert!(
        touched.contains("tests/basic.rs") && touched.contains("\"via\":\"shell\""),
        "the shell-made write parsed into the observed set: {}",
        touched
    );

    // ── (4) THE JUDGMENT SEAT: harvest renders all three channels and
    // states the sight boundary — a partial observed set cannot read as
    // complete, and the unresolved write faces the judge raw.
    let h = run(
        &qhome,
        &canon,
        &[("QUARRY_SESSION", "obs-dispatcher")],
        &["harvest", &item_id],
    );
    assert!(h.contains("src/lib.rs"), "the tool write renders: {}", h);
    assert!(
        h.contains("tests/basic.rs  (shell-parsed, best-effort)"),
        "the shell channel is named per file: {}",
        h
    );
    assert!(
        h.contains("unresolved under the badge (1)"),
        "the unresolved write renders at harvest: {}",
        h
    );
    assert!(
        h.contains("sight boundary:"),
        "the sight boundary is stated where observed-vs-leased renders: {}",
        h
    );
    assert!(
        !h.contains("leased but untouched: tests/**"),
        "the incident's lie is dead — the tests write no longer reads untouched: {}",
        h
    );

    let _ = fs::remove_dir_all(&base);
}
