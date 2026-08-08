//! `q view` — a single self-contained static HTML render of the whole graph.
//! The file is a derived view, never truth: regenerate at will, gitignored.
//! Landing view is the architecture map (areas, two tenses: built vs
//! becoming); drill-down runs area → item → file ref; the state panels are
//! the queue/ready/shaping/behind queries in page form.

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

use crate::model::At;
use crate::store::Store;

pub fn render(store: &Store) -> Result<String> {
    let all = store.load_all()?;
    let events = store.read_log()?;

    let mut file_current = serde_json::Map::new();
    let mut record_blob = |store: &Store, key: String| {
        if !file_current.contains_key(&key) {
            let f = key.strip_prefix("file:").unwrap_or(&key).to_string();
            let val = match store.blob(&f) {
                Ok(b) => json!(b),
                Err(_) => json!(null),
            };
            file_current.insert(key, val);
        }
    };
    for n in &all {
        for e in &n.front.edges {
            if let At::Blob(_) = &e.at {
                record_blob(store, e.to.clone());
            }
        }
        if n.front.ty == "doc" {
            if let Some(p) = &n.front.path {
                record_blob(store, format!("file:{}", p));
            }
        }
    }

    let nodes = all
        .iter()
        .map(|n| {
            let mut v = serde_json::to_value(&n.front)?;
            v["body"] = json!(n.body);
            v["slug"] = json!(n.slug());
            Ok(v)
        })
        .collect::<Result<Vec<_>>>()?;

    let root_name = store
        .root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".into());

    let data = json!({
        "generated": Store::now(),
        "root": root_name,
        "nodes": nodes,
        "file_current": file_current,
        "events": events,
    });
    let data_str = serde_json::to_string(&data)?.replace("</", "<\\/");
    Ok(TEMPLATE.replace("__QUARRY_DATA__", &data_str))
}

pub fn write(store: &Store) -> Result<PathBuf> {
    let html = render(store)?;
    let dir = store.root.join("graph").join("view");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("index.html");
    std::fs::write(&path, html)?;
    Ok(path)
}

const TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>quarry — work graph</title>
<style>
:root {
  --page: #f9f9f7; --surface: #fcfcfb; --ink: #0b0b0b; --ink2: #52514e;
  --muted: #898781; --grid: #e1e0d9; --border: rgba(11,11,11,0.10);
  --good: #0ca30c; --warn: #fab219; --serious: #ec835a; --critical: #d03b3b;
  --accent: #52514e;
}
@media (prefers-color-scheme: dark) {
  :root {
    --page: #0d0d0d; --surface: #1a1a19; --ink: #ffffff; --ink2: #c3c2b7;
    --muted: #898781; --grid: #2c2c2a; --border: rgba(255,255,255,0.10);
  }
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--page); color: var(--ink);
  font: 14px/1.5 system-ui, -apple-system, "Segoe UI", sans-serif; }
a { color: inherit; text-decoration: none; }
a:hover { text-decoration: underline; }
code, .id { font-family: ui-monospace, "Cascadia Mono", Consolas, monospace; font-size: 12px; }
.id { color: var(--muted); }
#topbar { position: sticky; top: 0; z-index: 5; background: var(--surface);
  border-bottom: 1px solid var(--grid); padding: 10px 20px;
  display: flex; align-items: center; gap: 18px; flex-wrap: wrap; }
#topbar h1 { font-size: 15px; margin: 0; }
#topbar .nav a { margin-right: 12px; color: var(--ink2); }
#topbar .nav a.active { color: var(--ink); font-weight: 600; }
#topbar .gen { color: var(--muted); font-size: 12px; margin-left: auto; }
#search { background: var(--page); color: var(--ink); border: 1px solid var(--grid);
  border-radius: 6px; padding: 5px 10px; width: 220px; }
#results { position: absolute; top: 44px; background: var(--surface);
  border: 1px solid var(--grid); border-radius: 8px; max-height: 320px;
  overflow: auto; min-width: 320px; box-shadow: 0 4px 16px rgba(0,0,0,.12); }
#results a { display: block; padding: 6px 12px; border-bottom: 1px solid var(--grid); }
main { max-width: 1200px; margin: 0 auto; padding: 20px; }
.tiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 16px; }
.tile { background: var(--surface); border: 1px solid var(--border);
  border-radius: 10px; padding: 14px 16px; }
.tile h2 { margin: 0 0 2px; font-size: 15px; }
.charter { color: var(--ink2); font-size: 12.5px; margin: 0 0 10px; }
.sect { margin-top: 10px; }
.sect .hd { font-size: 11px; letter-spacing: .06em; text-transform: uppercase;
  color: var(--muted); margin-bottom: 4px; }
.becoming { border-left: 2px dashed var(--grid); padding-left: 10px; }
ul.plain { list-style: none; margin: 0; padding: 0; }
ul.plain li { padding: 1.5px 0; }
.status { display: inline-block; font-size: 11px; color: var(--ink2);
  border: 1px solid var(--grid); border-radius: 999px; padding: 0 8px; margin-left: 6px; }
.dead { text-decoration: line-through; color: var(--muted); }
.dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%;
  margin-right: 6px; vertical-align: baseline; }
.sev1 .dot { background: var(--critical); } .sev2 .dot { background: var(--serious); }
.sev3 .dot { background: var(--warn); } .sev4 .dot { background: var(--warn); }
.ok .dot { background: var(--good); }
.coupling { margin-top: 10px; font-size: 12px; color: var(--ink2); }
.node { background: var(--surface); border: 1px solid var(--border);
  border-radius: 10px; padding: 18px 22px; }
.node h2 { margin: 0; font-size: 17px; }
.meta { color: var(--muted); font-size: 12.5px; margin: 4px 0 0; }
.body { margin: 14px 0; color: var(--ink2); max-width: 72ch; white-space: pre-wrap; }
table.edges { border-collapse: collapse; width: 100%; margin-top: 6px; }
table.edges td { padding: 4px 10px 4px 0; border-bottom: 1px solid var(--grid);
  vertical-align: top; font-size: 13px; }
.rel { color: var(--muted); font-size: 12px; white-space: nowrap; }
.delta { color: var(--muted); font-size: 12px; margin: 2px 0 2px 14px; }
h3.part { font-size: 12px; letter-spacing: .06em; text-transform: uppercase;
  color: var(--muted); margin: 22px 0 6px; }
.panel { background: var(--surface); border: 1px solid var(--border);
  border-radius: 10px; padding: 12px 16px; margin-bottom: 16px; }
.panel h2 { font-size: 13px; letter-spacing: .05em; text-transform: uppercase;
  color: var(--ink2); margin: 0 0 8px; }
.empty { color: var(--muted); font-style: italic; }
.crumb { color: var(--muted); font-size: 12.5px; margin-bottom: 10px; display: block; }
</style>
</head>
<body>
<div id="topbar">
  <h1><a href="#/">quarry · <span id="rootname"></span></a></h1>
  <span class="nav">
    <a href="#/" id="nav-map">Map</a>
    <a href="#/state" id="nav-state">State</a>
    <a href="#/all" id="nav-all">All nodes</a>
  </span>
  <span style="position:relative">
    <input id="search" placeholder="search nodes…" autocomplete="off">
    <div id="results" hidden></div>
  </span>
  <span class="gen" id="gen"></span>
</div>
<main id="main"></main>
<script>
const DATA = __QUARRY_DATA__;
const byId = {}; DATA.nodes.forEach(n => byId[n.id] = n);
const backlinks = {};
DATA.nodes.forEach(n => (n.edges||[]).forEach(e => {
  (backlinks[e.to] = backlinks[e.to] || []).push({from: n.id, rel: e.rel, at: e.at});
}));
const TENSE = {
  built: ['done','in-force','measured','ratified','resolved','registered','active'],
  becoming: ['sketch','shaped','ready','in-flight','open','queued'],
  dead: ['dropped','refuted','superseded'], parked: ['parked']
};
function tense(s){ for (const t in TENSE) if (TENSE[t].includes(s)) return t; return 'built'; }
function esc(s){ return String(s??'').replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])); }
function areasOf(n){ return (n.edges||[]).filter(e => e.rel==='about' && byId[e.to] && byId[e.to].type==='area').map(e => e.to); }
function edgeState(e){
  if (typeof e.at === 'number') {
    const t = byId[e.to];
    if (!t) return {sev: 2, label: 'dangling'};
    if (t.v > e.at) return TENSE.dead.includes(t.status)
      ? {sev: 1, label: 'cited v'+e.at+', now v'+t.v+' — '+t.status}
      : {sev: 3, label: 'cited v'+e.at+', now v'+t.v};
    return {sev: 0, label: 'v'+e.at+' current'};
  }
  const cur = DATA.file_current[e.to];
  if (cur === null || cur === undefined) return {sev: 2, label: 'file missing'};
  if (cur !== e.at) return {sev: 4, label: 'file drifted ('+e.at+' → '+cur+')'};
  return {sev: 0, label: 'blob current'};
}
function behindAll(){
  const out = [];
  DATA.nodes.forEach(n => (n.edges||[]).forEach(e => {
    const s = edgeState(e);
    if (s.sev > 0) out.push({n, e, s});
  }));
  DATA.nodes.filter(n => n.type==='doc' && n.path && n.blob).forEach(n => {
    const cur = DATA.file_current['file:'+n.path];
    if (cur !== undefined && cur !== null && cur !== n.blob)
      out.push({n, e: {rel:'path', to:'file:'+n.path, at:n.blob}, s:{sev:4, label:'registered doc drifted'}});
  });
  return out.sort((a,b) => a.s.sev - b.s.sev);
}
function blockers(n){
  return (n.edges||[]).filter(e => e.rel==='depends-on').map(e => byId[e.to]).filter(Boolean)
    .filter(t => t.type==='item' ? !['done','dropped'].includes(t.status)
      : t.type==='thread' ? t.status!=='resolved' : t.type==='decision' ? t.status!=='in-force' : false);
}
function nodeLink(id, cls){
  const n = byId[id];
  if (!n) return '<span class="id">'+esc(id)+'</span>';
  const dead = TENSE.dead.includes(n.status) ? ' dead' : '';
  return '<a class="'+(cls||'')+dead+'" href="#/n/'+esc(id)+'">'+esc(n.title)+'</a>'
    + '<span class="status">'+esc(n.status)+'</span>';
}
function li(n){ return '<li>'+nodeLink(n.id)+'</li>'; }
function statusDot(sev){ return '<span class="'+(sev===0?'ok':'sev'+sev)+'"><span class="dot"></span></span>'; }

function viewMap(){
  const areas = DATA.nodes.filter(n => n.type==='area' && n.status!=='retired')
    .sort((a,b) => a.title.localeCompare(b.title));
  let html = '<div class="tiles">';
  for (const a of areas) {
    const attached = DATA.nodes.filter(n => n.type!=='area' && areasOf(n).includes(a.id));
    const dec = attached.filter(n => n.type==='decision' && n.status==='in-force');
    const claims = attached.filter(n => n.type==='claim' && !TENSE.dead.includes(n.status));
    const itemsB = attached.filter(n => n.type==='item' && TENSE.becoming.includes(n.status));
    const itemsD = attached.filter(n => n.type==='item' && n.status==='done');
    const threads = attached.filter(n => n.type==='thread' && ['open','queued','parked'].includes(n.status));
    const docs = attached.filter(n => n.type==='doc');
    const coupling = {};
    attached.forEach(n => (n.edges||[]).forEach(e => {
      if (e.rel!=='depends-on' && e.rel!=='supports') return;
      const t = byId[e.to]; if (!t) return;
      areasOf(t).forEach(a2 => { if (a2!==a.id) coupling[a2]=(coupling[a2]||0)+1; });
    }));
    html += '<div class="tile"><h2><a href="#/n/'+a.id+'">'+esc(a.title)+'</a></h2>'
      + '<p class="charter">'+esc((a.body||'').split('\n')[0])+'</p>';
    html += '<div class="sect"><div class="hd">Built — decisions in force ('+dec.length
      +') · claims ('+claims.length+') · shipped ('+itemsD.length+')</div><ul class="plain">'
      + dec.map(li).join('') + '</ul></div>';
    if (itemsB.length || threads.length) {
      html += '<div class="sect becoming"><div class="hd">Becoming</div><ul class="plain">'
        + itemsB.sort((x,y)=>TENSE.becoming.indexOf(y.status)-TENSE.becoming.indexOf(x.status)).map(li).join('')
        + threads.map(li).join('') + '</ul></div>';
    }
    const cp = Object.entries(coupling);
    if (cp.length) html += '<div class="coupling">leans on: '
      + cp.map(([id,c]) => '<a href="#/n/'+id+'">'+esc(byId[id].title)+'</a> ×'+c).join(' · ')+'</div>';
    if (docs.length) html += '<div class="coupling">docs: '
      + docs.map(d => '<a href="#/n/'+d.id+'">'+esc(d.title)+'</a>').join(' · ')+'</div>';
    html += '</div>';
  }
  const unfiled = DATA.nodes.filter(n => n.type!=='area' && areasOf(n).length===0);
  if (unfiled.length) {
    html += '<div class="tile"><h2>unfiled</h2><p class="charter">nodes with no area attachment — the map cannot place what nobody filed</p><ul class="plain">'
      + unfiled.map(li).join('') + '</ul></div>';
  }
  html += '</div>';
  return html;
}

function eventsFor(id, afterV){
  return DATA.events.filter(ev => ev.node===id && (ev.v||0) > afterV)
    .map(ev => 'v'+ev.v+' '+ev.op+' '+esc(ev.fields ? ev.fields.join(', ') : (ev.field ? ev.field+'→'+ev.to : (ev.rel ? ev.rel+'→'+ev.to : ''))) + (ev.note ? ' — '+esc(ev.note) : ''));
}

function viewNode(id){
  const n = byId[id];
  if (!n) return '<p class="empty">no node '+esc(id)+'</p>';
  const areas = areasOf(n);
  let html = '';
  if (areas.length) html += '<span class="crumb">'+areas.map(a => '<a href="#/n/'+a+'">'+esc(byId[a].title)+'</a>').join(' · ')+'</span>';
  html += '<div class="node"><h2>'+esc(n.title)+'</h2>'
    + '<p class="meta"><span class="id">'+esc(n.id)+'</span> · '+esc(n.type)+' · v'+n.v
    + ' · <b>'+esc(n.status)+'</b> · '+esc(n.provenance)
    + (n.ratified ? ' · RATIFIED by '+esc(n.ratified.by)+' '+esc(n.ratified.date) : '')
    + (n.kind ? ' · '+esc(n.kind) : '') + (n.method ? ' · method: '+esc(n.method) : '')
    + (n.path ? ' · <code>'+esc(n.path)+'</code>' : '') + '</p>';
  if ((n.acceptance||[]).length) html += '<h3 class="part">Acceptance</h3><ul class="plain">'
    + n.acceptance.map(a => '<li>· '+esc(a)+'</li>').join('')+'</ul>';
  if (n.body) html += '<div class="body">'+esc(n.body)+'</div>';
  const bl = blockers(n);
  if (bl.length) html += '<h3 class="part">Blocked on</h3><ul class="plain">'+bl.map(li).join('')+'</ul>';
  if ((n.edges||[]).length) {
    html += '<h3 class="part">Edges</h3><table class="edges">';
    for (const e of n.edges) {
      const s = edgeState(e);
      let target;
      if (e.to.startsWith('file:')) target = '<code>'+esc(e.to.slice(5))+'</code>';
      else target = nodeLink(e.to);
      html += '<tr><td class="rel">'+esc(e.rel)+'</td><td>'+target+'</td>'
        + '<td>'+statusDot(s.sev)+'<span class="rel">'+esc(s.label)+'</span>';
      if (s.sev===1 || s.sev===3) {
        eventsFor(e.to, e.at).forEach(d => html += '<div class="delta">· '+d+'</div>');
      }
      html += '</td></tr>';
    }
    html += '</table>';
  }
  const inbound = backlinks[n.id]||[];
  if (inbound.length) {
    html += '<h3 class="part">Backlinks</h3><table class="edges">';
    for (const b of inbound) {
      const stale = (typeof b.at==='number' && n.v > b.at)
        ? statusDot(3)+'<span class="rel">cited at v'+b.at+', this is v'+n.v+'</span>'
        : statusDot(0)+'<span class="rel">current</span>';
      html += '<tr><td class="rel">'+esc(b.rel)+' ←</td><td>'+nodeLink(b.from)+'</td><td>'+stale+'</td></tr>';
    }
    html += '</table>';
  }
  html += '</div>';
  return html;
}

function panel(title, items, empty){
  return '<div class="panel"><h2>'+title+'</h2>'
    + (items.length ? '<ul class="plain">'+items.join('')+'</ul>' : '<p class="empty">'+empty+'</p>')
    + '</div>';
}

function viewState(){
  const threadsQ = DATA.nodes.filter(n => n.type==='thread' && n.status==='queued');
  const queue = threadsQ.filter(n => blockers(n).length===0);
  const blockedQ = threadsQ.length - queue.length;
  const ready = DATA.nodes.filter(n => n.type==='item' && n.status==='ready' && blockers(n).length===0);
  const shaping = DATA.nodes.filter(n => n.type==='item' && ['sketch','shaped'].includes(n.status));
  const behind = behindAll();
  const sevLabel = {1:'upstream refuted/superseded',2:'dangling',3:'target changed',4:'file drifted'};
  let html = panel('Queue — answerable now'+(blockedQ ? ' ('+blockedQ+' more blocked on spikes)' : ''),
      queue.map(li), 'nothing awaits the user');
  html += panel('Ready to dispatch', ready.map(li), 'nothing dispatchable');
  html += panel('Shaping — upcoming work', shaping.map(n => {
    const b = blockers(n);
    return '<li>'+nodeLink(n.id)+(b.length ? '<div class="delta">blocked on '+b.map(x => nodeLink(x.id)).join(', ')+'</div>' : '')+'</li>';
  }), 'nothing sketched');
  html += panel('Behind — stale refs', behind.map(x =>
    '<li>'+statusDot(x.s.sev)+nodeLink(x.n.id)+' <span class="rel">-['+esc(x.e.rel)+']→ '
    + (x.e.to.startsWith('file:') ? '<code>'+esc(x.e.to.slice(5))+'</code>' : nodeLink(x.e.to))
    + ' — '+esc(sevLabel[x.s.sev])+': '+esc(x.s.label)+'</span></li>'), 'every ref current');
  const unver = DATA.nodes.filter(n => n.type==='claim' && n.provenance==='assistant' && n.status==='asserted' && !n.method);
  html += panel('Unverified assistant claims', unver.map(li), 'none');
  return html;
}

function viewAll(){
  const order = ['area','decision','claim','thread','item','doc'];
  let html = '';
  for (const ty of order) {
    const ns = DATA.nodes.filter(n => n.type===ty).sort((a,b)=>a.title.localeCompare(b.title));
    if (ns.length) html += panel(ty+' ('+ns.length+')', ns.map(li), '');
  }
  return html;
}

function route(){
  const h = location.hash || '#/';
  document.querySelectorAll('.nav a').forEach(a => a.classList.remove('active'));
  let html;
  if (h.startsWith('#/n/')) html = viewNode(h.slice(4));
  else if (h === '#/state') { html = viewState(); document.getElementById('nav-state').classList.add('active'); }
  else if (h === '#/all') { html = viewAll(); document.getElementById('nav-all').classList.add('active'); }
  else { html = viewMap(); document.getElementById('nav-map').classList.add('active'); }
  document.getElementById('main').innerHTML = html;
  window.scrollTo(0,0);
}
window.addEventListener('hashchange', route);
document.getElementById('rootname').textContent = DATA.root;
document.getElementById('gen').textContent = 'generated '+DATA.generated+' · '+DATA.nodes.length+' nodes';
const searchEl = document.getElementById('search'), resultsEl = document.getElementById('results');
searchEl.addEventListener('input', () => {
  const q = searchEl.value.trim().toLowerCase();
  if (!q) { resultsEl.hidden = true; return; }
  const hits = DATA.nodes.filter(n =>
    n.id.includes(q) || n.title.toLowerCase().includes(q) || (n.body||'').toLowerCase().includes(q)
  ).slice(0, 20);
  resultsEl.innerHTML = hits.map(n => '<a href="#/n/'+n.id+'"><span class="id">'+esc(n.id)
    +'</span> '+esc(n.title)+'</a>').join('') || '<a class="empty">no match</a>';
  resultsEl.hidden = false;
});
searchEl.addEventListener('blur', () => setTimeout(() => resultsEl.hidden = true, 200));
route();
</script>
</body>
</html>
"##;
