use quarry::model::At;
use quarry::ops::{self, NewArgs};
use quarry::queries;
use quarry::store::Store;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_store() -> Store {
    std::env::set_var("QUARRY_ACTOR", "test-user");
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("quarry-test-{}-{}-{}", std::process::id(), n, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    Store::init(&dir).unwrap()
}

#[test]
fn create_and_reload() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    assert!(a.front.id.starts_with("ar-"));
    assert_eq!(a.front.v, 1);
    let mut args = NewArgs::bare("item", "S11 spike");
    args.kind = Some("slice".into());
    args.acceptance = vec!["halo measured".into()];
    let i = ops::new_node(&s, args).unwrap();
    assert!(i.front.id.starts_with("it-"));
    assert_eq!(i.front.status, "sketch");

    let all = s.load_all().unwrap();
    assert_eq!(all.len(), 2);
    let found = s.find(&all, "water").unwrap();
    assert_eq!(found.front.id, a.front.id);
    let by_slug = s.find(&all, "s11-spike").unwrap();
    assert_eq!(by_slug.front.id, i.front.id);
}

#[test]
fn stamped_edges_go_behind_and_affirm() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    // user-provenance claim needs no source (C2 exemption)
    let c = ops::claim(
        &s,
        "the halo is bounded",
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    assert_eq!(c.front.edges.len(), 1);
    assert_eq!(c.front.edges[0].at, At::V(1));

    // bump the area; the claim's edge is now behind
    ops::set(&s, &area.front.id, &["title=hydrology".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let behind = queries::behind(&s, &all);
    assert_eq!(behind.len(), 1);
    assert_eq!(behind[0].src_id, c.front.id);
    assert_eq!(behind[0].severity, 3);

    // affirm restamps
    let n = ops::affirm(&s, &c.front.id, None).unwrap();
    assert_eq!(n, 1);
    let all = s.load_all().unwrap();
    assert!(queries::behind(&s, &all).is_empty());
}

#[test]
fn c3_denies_assistant_settling_user_thread() {
    let s = temp_store();
    let mut t = NewArgs::bare("thread", "ocean: finite or pinned?");
    t.provenance = Some("user".into());
    let t = ops::new_node(&s, t).unwrap();

    let mut d = NewArgs::bare("decision", "pinned");
    d.provenance = Some("assistant".into());
    let d = ops::new_node(&s, d).unwrap();

    let err = ops::link(&s, &d.front.id, "settles", &t.front.id, false, None).unwrap_err();
    assert!(err.to_string().contains("C3"), "got: {}", err);

    // rule with actor test-user derives user provenance and succeeds
    let dec = ops::rule(&s, &t.front.id, "pinned: level set by the world", Some("user".into()), None).unwrap();
    assert_eq!(dec.front.provenance, "user");
    assert!(dec.front.ratified.is_some());
    let all = s.load_all().unwrap();
    let t2 = s.find(&all, &t.front.id).unwrap();
    assert_eq!(t2.front.status, "resolved");
    // the decision inherits nothing here (thread had no about), but settles edge exists
    let dec2 = s.find(&all, &dec.front.id).unwrap();
    assert!(dec2.front.edges.iter().any(|e| e.rel == "settles" && e.to == t.front.id));
}

#[test]
fn refute_blast_and_c5() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let src_doc = ops::new_node(&s, NewArgs::bare("doc", "S11 results")).unwrap();
    let evidence = ops::new_node(&s, NewArgs::bare("doc", "S14 remeasurement")).unwrap();

    let c = ops::claim(
        &s,
        "halo is 4-11 cells",
        vec![area.front.id.clone()],
        Some(src_doc.front.id.clone()),
        Some("ring differencing".into()),
        None,
        None,
    )
    .unwrap();
    assert_eq!(c.front.provenance, "measured");

    let mut d = NewArgs::bare("decision", "persist bodies, derive voxels");
    d.provenance = Some("user".into());
    let d = ops::new_node(&s, d).unwrap();
    ops::link(&s, &c.front.id, "supports", &d.front.id, false, None).unwrap();

    let mut it = NewArgs::bare("item", "water body graph");
    it.status = Some("ready".into());
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &d.front.id, false, None).unwrap();

    let (c2, blast) = ops::refute(&s, &c.front.id, &evidence.front.id, None).unwrap();
    assert_eq!(c2.front.status, "refuted");
    let ids: Vec<&str> = blast.iter().map(|n| n.front.id.as_str()).collect();
    assert!(ids.contains(&d.front.id.as_str()), "decision in blast: {:?}", ids);
    assert!(ids.contains(&it.front.id.as_str()), "item in blast: {:?}", ids);

    // C5: citing the refuted claim now requires --acknowledge
    let c5 = ops::link(&s, &src_doc.front.id, "supports", &c2.front.id, false, None).unwrap_err();
    assert!(c5.to_string().contains("C5"), "got: {}", c5);
    ops::link(&s, &src_doc.front.id, "supports", &c2.front.id, true, None).unwrap();
}

#[test]
fn ready_respects_thread_blockers() {
    let s = temp_store();
    let mut it = NewArgs::bare("item", "attitude recording");
    it.status = Some("ready".into());
    let it = ops::new_node(&s, it).unwrap();

    let all = s.load_all().unwrap();
    assert_eq!(queries::ready(&all).len(), 1);

    let mut th = NewArgs::bare("thread", "fact or interpretation?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &th.front.id, false, None).unwrap();

    let all = s.load_all().unwrap();
    assert!(queries::ready(&all).is_empty(), "item blocked by open thread");
    assert_eq!(queries::queue(&all).len(), 1);

    ops::rule(&s, &th.front.id, "recorded in deeptime", Some("user".into()), None).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(queries::ready(&all).len(), 1, "ruling unblocks the item");
    assert!(queries::queue(&all).is_empty());
}

#[test]
fn thread_blocked_on_spike_leaves_queue() {
    let s = temp_store();
    let mut th = NewArgs::bare("thread", "which storage layout?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();

    let mut spike = NewArgs::bare("item", "spike: measure both layouts");
    spike.status = Some("in-flight".into());
    let spike = ops::new_node(&s, spike).unwrap();
    ops::link(&s, &th.front.id, "depends-on", &spike.front.id, false, None).unwrap();

    let all = s.load_all().unwrap();
    assert!(queries::queue(&all).is_empty(), "thread waits on its spike");

    ops::set(&s, &spike.front.id, &["status=done".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(queries::queue(&all).len(), 1, "spike landed; thread answerable");
}

#[test]
fn hook_guard_denies_graph_writes_only() {
    let windows_path = "{\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"B:\\\\repos\\\\x\\\\graph\\\\nodes\\\\claim\\\\cl-1.md\"}}";
    let deny = quarry::teach::guard(windows_path);
    assert!(deny.is_some());
    assert!(deny.unwrap().contains("C6"));
    let log = quarry::teach::guard(
        r#"{"tool_name":"Write","tool_input":{"file_path":"B:/repos/x/graph/log/2026-08.jsonl"}}"#,
    );
    assert!(log.is_some());
    let src = quarry::teach::guard(
        r#"{"tool_name":"Edit","tool_input":{"file_path":"B:/repos/x/src/main.rs"}}"#,
    );
    assert!(src.is_none());
    let read = quarry::teach::guard(
        r#"{"tool_name":"Read","tool_input":{"file_path":"B:/repos/x/graph/nodes/a.md"}}"#,
    );
    assert!(read.is_none());
    let garbage = quarry::teach::guard("not json");
    assert!(garbage.is_none());
}

#[test]
fn view_renders_every_node() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let mut th = NewArgs::bare("thread", "finite or pinned?");
    th.provenance = Some("user".into());
    th.about = vec![a.front.id.clone()];
    let th = ops::new_node(&s, th).unwrap();
    let html = quarry::view::render(&s).unwrap();
    assert!(html.contains(&a.front.id));
    assert!(html.contains(&th.front.id));
    assert!(html.contains("finite or pinned?"));
    assert!(html.contains("__QUARRY_DATA__") == false);
}

#[test]
fn affirm_only_restamps_does_not_bump() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let c = ops::claim(
        &s,
        "halo bounded",
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    ops::set(&s, &area.front.id, &["title=hydrology".to_string()], None).unwrap();
    let v_before = c.front.v;
    let n = ops::affirm(&s, &c.front.id, None).unwrap();
    assert_eq!(n, 1);
    let all = s.load_all().unwrap();
    let c2 = s.find(&all, &c.front.id).unwrap();
    assert_eq!(c2.front.v, v_before, "affirm-only restamp must not bump v");
    assert!(queries::behind(&s, &all).is_empty());
}

#[test]
fn homework_helpers() {
    let s = temp_store();
    let mut th = NewArgs::bare("thread", "which layout?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();
    let mut it = NewArgs::bare("item", "build the layout");
    it.status = Some("ready".into());
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &th.front.id, false, None).unwrap();

    // bumping the thread puts the item's stamp behind
    ops::set(&s, &th.front.id, &["title=which storage layout?".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let behind = queries::citers_behind(&all, &th.front.id);
    assert_eq!(behind.len(), 1);
    assert_eq!(behind[0].0.front.id, it.front.id);
    // thread not resolved: nothing unblocked yet
    assert!(queries::unblocked_by(&all, &th.front.id).is_empty());

    ops::rule(&s, &th.front.id, "flat files", Some("user".into()), None).unwrap();
    let all = s.load_all().unwrap();
    let un = queries::unblocked_by(&all, &th.front.id);
    assert_eq!(un.len(), 1);
    assert_eq!(un[0].front.id, it.front.id);
}

#[test]
fn doc_markdown_content_embeds_in_view() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::write(s.root.join("NOTES.md"), "# Heading One

| a | b |
|---|---|
| 1 | 2 |
").unwrap();
    let mut d = NewArgs::bare("doc", "notes");
    d.path = Some("NOTES.md".into());
    ops::new_node(&s, d).unwrap();
    let html = quarry::view::render(&s).unwrap();
    assert!(html.contains("Heading One"), "md content embedded");
    assert!(html.contains("doc_content"));
}

#[test]
fn glob_overlap_heuristic() {
    use quarry::coord::globs_overlap;
    assert!(globs_overlap("crates/**", "crates/dc-sim/**"));
    assert!(globs_overlap("crates/dc-sim/**", "crates/**"));
    assert!(!globs_overlap("crates/dc-worldgen/**", "crates/dc-sim/**"));
    assert!(globs_overlap("docs/design/*.md", "docs/**"));
    assert!(globs_overlap("*.rs", "src/main.rs"), "bare wildcard is conservative");
    assert!(!globs_overlap("docs/a/**", "docs/b/**"));
}

#[test]
fn leases_exclusive_shared_steal() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("item", "geo pass work")).unwrap();
    let b = ops::new_node(&s, NewArgs::bare("item", "bodies gait work")).unwrap();
    let c = ops::new_node(&s, NewArgs::bare("item", "sdk docs from geo")).unwrap();
    let d = ops::new_node(&s, NewArgs::bare("item", "sdk docs from bodies")).unwrap();

    quarry::coord::reserve(&s, &a, "geo", "t", vec!["crates/dc-worldgen/**".into()], false, false).unwrap();
    // disjoint: fine
    quarry::coord::reserve(&s, &b, "bodies", "t", vec!["crates/dc-sim/body/**".into()], false, false).unwrap();
    // overlapping exclusive from another session: denied, names holder
    let mut e = NewArgs::bare("item", "bodies wants worldgen");
    e.status = Some("sketch".into());
    let e = ops::new_node(&s, e).unwrap();
    let err = quarry::coord::reserve(&s, &e, "bodies", "t", vec!["crates/dc-worldgen/deep/**".into()], false, false)
        .unwrap_err();
    assert!(err.to_string().contains("C7"), "got: {}", err);
    assert!(err.to_string().contains("geo"));
    // shared + shared coexist with visibility
    quarry::coord::reserve(&s, &c, "geo", "t", vec!["docs/sdk/**".into()], true, false).unwrap();
    let out = quarry::coord::reserve(&s, &d, "bodies", "t", vec!["docs/sdk/**".into()], true, false).unwrap();
    assert_eq!(out.co_holders.len(), 1);
    assert_eq!(out.co_holders[0].session, "geo");
    // steal is allowed and reported
    let out = quarry::coord::reserve(&s, &e, "bodies", "t", vec!["crates/dc-worldgen/deep/**".into()], false, true).unwrap();
    assert_eq!(out.stolen.len(), 1);
    assert_eq!(out.stolen[0].session, "geo");
    // release: own lease only
    let err = quarry::coord::release(&s, &b, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("bodies"));
    quarry::coord::release(&s, &b, "bodies", "t").unwrap();
}

#[test]
fn purview_scoping() {
    let s = temp_store();
    let geo = ops::new_node(&s, NewArgs::bare("area", "worldgen passes")).unwrap();
    let bod = ops::new_node(&s, NewArgs::bare("area", "bodies")).unwrap();
    let mut i1 = NewArgs::bare("item", "erosion pass");
    i1.about = vec![geo.front.id.clone()];
    i1.status = Some("ready".into());
    ops::new_node(&s, i1).unwrap();
    let mut i2 = NewArgs::bare("item", "gait clip");
    i2.about = vec![bod.front.id.clone()];
    i2.status = Some("ready".into());
    ops::new_node(&s, i2).unwrap();

    quarry::coord::save_session(&s, "geo", vec![geo.front.id.clone()], None).unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert!(reg.contains_key("geo"));
    let all = s.load_all().unwrap();
    let ids: Vec<&str> = reg["geo"].areas.iter().map(|x| x.as_str()).collect();
    let mine: Vec<_> = queries::ready(&all)
        .into_iter()
        .filter(|n| quarry::coord::in_purview(n, &ids))
        .collect();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].front.title, "erosion pass");
}

#[test]
fn c2_grounding_forms() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::write(s.root.join("evidence.rs"), "fn observed() {}
").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();

    // free-floating assistant claim: denied
    let err = ops::claim(&s, "vibes", vec![area.front.id.clone()], None, None, Some("assistant".into()), None)
        .unwrap_err();
    assert!(err.to_string().contains("C2"), "got: {}", err);
    // method alone grounds it
    ops::claim(&s, "measured thing", vec![area.front.id.clone()], None,
        Some("log inspection".into()), None, None).unwrap();
    // a file: source grounds it (matrix widened), blob-stamped
    let c = ops::claim(&s, "read off the code", vec![area.front.id.clone()],
        Some("file:evidence.rs".into()), None, Some("assistant".into()), None).unwrap();
    assert!(c.front.edges.iter().any(|e| e.rel == "source" && e.to == "file:evidence.rs"));
}
