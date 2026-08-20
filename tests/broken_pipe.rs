//! The broken-pipe proof (it-8tcy): a closed stdout is a normal fate for
//! CLI output (`head -1` takes its line and leaves; a terminal dies) and
//! must never panic a verb or skip the state work sequenced after prints.
//! These tests spawn the real binary with stdout piped and drop the read
//! end immediately — on Windows every later write to the pipe fails with
//! ERROR_NO_DATA (os error 232), the incident's exact shape. The verb must
//! complete ALL its mutations and exit successfully; the homework the dead
//! pipe swallowed must re-derive at `q query homework <node>`.
//!
//! (The drop races the child's first write: process spawn plus graph load
//! takes milliseconds while the drop is immediate, so in practice the pipe
//! is always dead before the first print. Even in the losing race the
//! assertions still hold — the test just exercises the happy path once in
//! a great while.)

use quarry::ops::{self, NewArgs};
use quarry::store::Store;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Spawn the real q binary: env scrubbed of every QUARRY_* the harness or a
/// dev shell might carry, then the pin-file home applied (the
/// worktree_proof pattern).
fn q_cmd(qhome: &Path, dir: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_q"));
    c.current_dir(dir)
        .env_remove("QUARRY_SESSION")
        .env_remove("QUARRY_DISPATCH")
        .env_remove("QUARRY_CHAT")
        .env_remove("QUARRY_AGENT")
        .env_remove("QUARRY_STORE")
        .env_remove("QUARRY_HOME")
        .env("QUARRY_HOME", qhome)
        .env("QUARRY_ACTOR", "pipe-proof-actor")
        .args(args);
    c
}

/// Run a verb with stdout closed before the child can print: pipe the
/// child's stdout, drop our read end at once, capture stderr for the
/// diagnosis. Returns (success, stderr).
fn run_with_dead_stdout(qhome: &Path, dir: &Path, args: &[&str]) -> (bool, String) {
    let mut child = q_cmd(qhome, dir, args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take()); // the read end dies here — head has left
    let out = child.wait_with_output().unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stderr).to_string())
}

fn run_ok(qhome: &Path, dir: &Path, args: &[&str]) -> String {
    let out = q_cmd(qhome, dir, args).output().unwrap();
    assert!(
        out.status.success(),
        "q {:?} failed: {}{}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn temp_base(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base =
        std::env::temp_dir().join(format!("quarry-pipe-{}-{}-{}", tag, std::process::id(), nanos));
    std::fs::create_dir_all(&base).unwrap();
    base
}

/// The sequencing proof: real state work rides AFTER prints in the verb
/// paths (readings sweep, ratification, view regeneration behind the
/// done-flip's confirmation line) — a dead pipe must not skip any of it.
#[test]
fn a_closed_pipe_never_panics_and_state_after_the_print_still_lands() {
    let base = temp_base("state");
    let repo = base.join("repo");
    let qhome = base.join("qhome");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&qhome).unwrap();
    let s = Store::init(&repo).unwrap();
    std::env::set_var("QUARRY_ACTOR", "test-user");
    for v in ["QUARRY_SESSION", "QUARRY_DISPATCH", "QUARRY_CHAT", "QUARRY_AGENT", "QUARRY_STORE"] {
        std::env::remove_var(v);
    }

    let area = ops::new_node(&s, NewArgs::bare("area", "pipe country")).unwrap();
    let mut it = NewArgs::bare("item", "the pipe proof slice");
    it.status = Some("ready".into());
    it.acceptance = vec!["the pipe holds".into()];
    it.about = vec![area.front.id.clone()];
    let it = ops::new_node(&s, it).unwrap();
    // A reading whose only consumer is the item: the done-flip's
    // sweep_readings — sequenced after several prints — must archive it.
    let reading = ops::claim(
        &s,
        "a one-off reading the landing consumes",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    ops::link(&s, &reading.front.id, "supports", &it.front.id, false, None).unwrap();

    let (ok, stderr) = run_with_dead_stdout(
        &qhome,
        &repo,
        &["set", &it.front.id, "status=done"],
    );
    assert!(ok, "the verb must exit quietly with stdout dead, never panic: {}", stderr);
    assert!(
        !stderr.contains("panicked"),
        "no panic residue on stderr: {}",
        stderr
    );

    let all = s.load_all().unwrap();
    let it2 = s.find(&all, &it.front.id).unwrap();
    assert_eq!(it2.front.status, "done", "the flip itself landed");
    let r2 = s.find(&all, &reading.front.id).unwrap();
    assert!(
        r2.front.archived,
        "the readings sweep — sequenced after the done-flip's prints — still ran"
    );
    assert!(
        repo.join("graph").join("view").join("index.html").exists(),
        "view regeneration — sequenced after prints — still ran"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// The incident's shape end to end: the edit lands, the homework print dies
/// with the pipe — and the homework re-derives at the query, because the
/// act-time print is a delivery of a derivable surface, not the only copy.
#[test]
fn homework_lost_to_a_dead_pipe_rederives_at_the_query() {
    let base = temp_base("hw");
    let repo = base.join("repo");
    let qhome = base.join("qhome");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&qhome).unwrap();
    let s = Store::init(&repo).unwrap();
    std::env::set_var("QUARRY_ACTOR", "test-user");
    for v in ["QUARRY_SESSION", "QUARRY_DISPATCH", "QUARRY_CHAT", "QUARRY_AGENT", "QUARRY_STORE"] {
        std::env::remove_var(v);
    }

    let mut th = NewArgs::bare("thread", "which pipe layout?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();
    let mut it = NewArgs::bare("item", "build the pipe layout");
    it.status = Some("ready".into());
    it.acceptance = vec!["the layout stands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &th.front.id, false, None).unwrap();

    // The mutation lands; its homework print (the item now behind) goes to
    // a pipe that is already dead.
    let (ok, stderr) = run_with_dead_stdout(
        &qhome,
        &repo,
        &["set", &th.front.id, "title=the pipe layout question"],
    );
    assert!(ok, "the verb must exit quietly with stdout dead: {}", stderr);
    let all = s.load_all().unwrap();
    let th2 = s.find(&all, &th.front.id).unwrap();
    assert!(
        quarry::surface::title_raw(th2).contains("the pipe layout question"),
        "the edit itself landed"
    );

    // The pull: the same derivation the dead pipe swallowed, on demand.
    let out = run_ok(&qhome, &repo, &["query", "homework", &th.front.id]);
    assert!(
        out.contains("behind:")
            && out.contains(&format!("q affirm {} --to {}", it.front.id, th.front.id)),
        "the homework re-derives with the affirm command in hand: {}",
        out
    );
    let _ = std::fs::remove_dir_all(&base);
}
