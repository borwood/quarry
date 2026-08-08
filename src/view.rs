//! `q view` — a single self-contained static HTML render of the whole graph.
//! The file is a derived view, never truth: regenerate at will, gitignored.
//! Landing view is the architecture map (areas, two tenses: built vs
//! becoming) plus a recent-activity strip; drill-down runs area → item →
//! file ref; the state panels are the queue/ready/shaping/behind queries in
//! page form. Every row list is a uniform table: type · title · status ·
//! v · updated (derived per node from the event log).

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

    // Markdown bodies of path-backed docs, embedded for in-app reading.
    // Only .md files — source code is deliberately not viewable in-app.
    let mut doc_content = serde_json::Map::new();
    for n in &all {
        if n.front.ty == "doc" {
            if let Some(p) = &n.front.path {
                if p.to_lowercase().ends_with(".md") {
                    if let Ok(text) = std::fs::read_to_string(store.root.join(p)) {
                        let capped: String = text.chars().take(200_000).collect();
                        doc_content.insert(p.clone(), json!(capped));
                    }
                }
            }
        }
    }

    let root_name = store
        .root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".into());

    // Presentation config from the protocol layer: a doc node kind=protocol
    // with on=view may declare collapse=<N> (default 8).
    let collapse = crate::protocol::matching(&all, "view", None, None)
        .iter()
        .find_map(|n| {
            n.front
                .extra
                .get("collapse")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<usize>().ok())
        })
        .unwrap_or(8);

    let data = json!({
        "generated": Store::now(),
        "root": root_name,
        "nodes": nodes,
        "file_current": file_current,
        "doc_content": doc_content,
        "events": events,
        "config": { "collapse": collapse },
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
code, .id, .ty { font-family: ui-monospace, "Cascadia Mono", Consolas, monospace; }
code, .id { font-size: 12px; }
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
  overflow: auto; min-width: 360px; box-shadow: 0 4px 16px rgba(0,0,0,.12); }
#results a { display: block; padding: 6px 12px; border-bottom: 1px solid var(--grid); }
main { max-width: 1240px; margin: 0 auto; padding: 20px; }
.tiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(400px, 1fr)); gap: 16px; }
.tile, .panel, .node { background: var(--surface); border: 1px solid var(--border);
  border-radius: 10px; padding: 14px 16px; }
.panel { margin-bottom: 16px; }
.tile h2 { margin: 0 0 2px; font-size: 15px; }
.panel h2 { font-size: 13px; letter-spacing: .05em; text-transform: uppercase;
  color: var(--ink2); margin: 0 0 8px; }
.charter { color: var(--ink2); font-size: 12.5px; margin: 0 0 10px; }
.sect { margin-top: 12px; }
.sect .hd { font-size: 11px; letter-spacing: .06em; text-transform: uppercase;
  color: var(--muted); margin-bottom: 6px; display: inline-block;
  padding-bottom: 2px; border-bottom: 1px solid var(--grid); }
.becoming .hd { border-bottom-style: dashed; border-bottom-color: var(--muted); color: var(--ink2); }
.becoming .hd::before { content: '▹ '; color: var(--muted); }
table.tbl { border-collapse: collapse; width: 100%; }
.tbl th { text-align: left; font-size: 10.5px; letter-spacing: .06em;
  text-transform: uppercase; color: var(--muted); font-weight: 600;
  padding: 2px 12px 4px 0; border-bottom: 1px solid var(--grid); white-space: nowrap; }
.tbl td { padding: 4px 12px 4px 0; border-bottom: 1px solid var(--grid);
  font-size: 13px; vertical-align: top; }
.tbl tr:last-child td { border-bottom: none; }
.ty { color: var(--muted); font-size: 11px; white-space: nowrap; }
.vcol { color: var(--muted); font-size: 12px; font-variant-numeric: tabular-nums; white-space: nowrap; }
.when { color: var(--muted); font-size: 12px; white-space: nowrap; font-variant-numeric: tabular-nums; }
.when.fresh { color: var(--ink); font-weight: 600; }
.bubble { display: inline-block; font-size: 11px; border-radius: 999px;
  padding: 0 8px; border: 1px solid var(--grid); color: var(--ink2); white-space: nowrap; }
.b-built  { background: rgba(12,163,12,.12);  border-color: rgba(12,163,12,.40); }
.b-flight { background: rgba(250,178,25,.16); border-color: rgba(250,178,25,.55); }
.b-ready  { background: rgba(137,135,129,.18); border-color: var(--muted); font-weight: 600; }
.b-dead   { background: rgba(208,59,59,.10);  border-color: rgba(208,59,59,.40);
  color: var(--muted); text-decoration: line-through; }
.dead-title { color: var(--muted); text-decoration: line-through; }
.dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%;
  margin-right: 6px; vertical-align: baseline; }
.sev1 .dot { background: var(--critical); } .sev2 .dot { background: var(--serious); }
.sev3 .dot { background: var(--warn); } .sev4 .dot { background: var(--warn); }
.ok .dot { background: var(--good); }
.coupling { margin-top: 10px; font-size: 12px; color: var(--ink2); }
.node { padding: 18px 22px; }
.node h2 { margin: 0; font-size: 17px; }
.meta { color: var(--muted); font-size: 12.5px; margin: 4px 0 0; }
.body { margin: 14px 0; color: var(--ink); background: var(--page);
  border: 1px solid var(--grid); border-radius: 8px; padding: 12px 14px;
  max-width: 76ch; white-space: pre-wrap; }
.rel { color: var(--muted); font-size: 12px; white-space: nowrap; }
.delta { color: var(--muted); font-size: 12px; padding-left: 14px; }
h3.part { font-size: 12px; letter-spacing: .06em; text-transform: uppercase;
  color: var(--ink2); margin: 22px 0 6px; }
h3.part .arrow { color: var(--muted); font-weight: 400; }
.empty { color: var(--muted); font-style: italic; }
.crumb { color: var(--muted); font-size: 12.5px; margin-bottom: 10px; display: block; }
.evd { color: var(--ink2); font-size: 12.5px; }
.mdview { max-width: 84ch; color: var(--ink); font-size: 13.5px; }
.mdview h2, .mdview h3, .mdview h4, .mdview h5 { margin: 18px 0 6px; line-height: 1.3; }
.mdview h2 { font-size: 16px; } .mdview h3 { font-size: 14.5px; } .mdview h4, .mdview h5 { font-size: 13.5px; }
.mdview p { margin: 6px 0; }
.mdview ul { margin: 6px 0; padding-left: 20px; }
.mdview pre { background: var(--page); border: 1px solid var(--grid); border-radius: 8px;
  padding: 10px 12px; overflow-x: auto; font-size: 12.5px; font-family: ui-monospace, Consolas, monospace; }
.mdview blockquote { border-left: 3px solid var(--grid); margin: 8px 0; padding: 2px 12px; color: var(--ink2); }
.mdview hr { border: 0; border-top: 1px solid var(--grid); margin: 14px 0; }
.mdview table { margin: 8px 0; }
.mdview td { font-size: 12.5px; }
tr.arch { opacity: .55; }
.morebar { font-size: 12px; color: var(--muted); margin-top: 5px; }
.morebar a { color: var(--ink2); text-decoration: underline; }
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
const lastTs = {};
DATA.events.forEach(ev => {
  if (ev.node && ev.ts && (!lastTs[ev.node] || ev.ts > lastTs[ev.node])) lastTs[ev.node] = ev.ts;
});
const TENSE = {
  built: ['done','in-force','measured','ratified','resolved','registered','active'],
  becoming: ['sketch','shaped','ready','in-flight','open','queued'],
  dead: ['dropped','refuted','superseded'], parked: ['parked']
};
function tense(s){ for (const t in TENSE) if (TENSE[t].includes(s)) return t; return 'built'; }
function esc(s){ return String(s??'').replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])); }
function areasOf(n){ return (n.edges||[]).filter(e => e.rel==='about' && byId[e.to] && byId[e.to].type==='area').map(e => e.to); }
function updatedOf(n){ return lastTs[n.id] || n.created; }
function relTime(ts){
  const ms = Date.now() - Date.parse(ts);
  if (isNaN(ms)) return ts;
  const m = Math.floor(ms/60000);
  if (m < 1) return 'just now';
  if (m < 60) return m+'m ago';
  const h = Math.floor(m/60);
  if (h < 24) return h+'h ago';
  const d = Math.floor(h/24);
  if (d < 14) return d+'d ago';
  return ts.split('T')[0];
}
function whenCell(ts){
  const fresh = (Date.now() - Date.parse(ts)) < 86400000;
  return '<td class="when'+(fresh?' fresh':'')+'" title="'+esc(ts)+'">'+esc(relTime(ts))+'</td>';
}
function bubbleClass(s){
  if (TENSE.dead.includes(s)) return 'b-dead';
  if (s === 'in-flight') return 'b-flight';
  if (s === 'ready') return 'b-ready';
  if (TENSE.built.includes(s)) return 'b-built';
  return '';
}
function bubble(s){ return '<span class="bubble '+bubbleClass(s)+'">'+esc(s)+'</span>'; }
function titleLink(n){
  const dead = TENSE.dead.includes(n.status) ? ' class="dead-title"' : '';
  return '<a'+dead+' href="#/n/'+esc(n.id)+'">'+esc(n.title)+'</a>';
}
function tbl(headers, rows){
  if (!rows.length) return '<p class="empty">—</p>';
  return '<table class="tbl"><thead><tr>'+headers.map(h => '<th>'+h+'</th>').join('')
    + '</tr></thead><tbody>'+rows.join('')+'</tbody></table>';
}
window.VS = window.VS || {arch: {}, exp: {}};
/// Collapsible row group: shows at most config.collapse live rows, always
/// COUNTS what it hides (more + archived) with reach-them toggles — no
/// surface hides content silently.
function collapsible(key, rowObjs, headers){
  const lim = (DATA.config && DATA.config.collapse) || 8;
  const archN = rowObjs.filter(r => r.a).length;
  const showA = !!VS.arch[key];
  let list = showA ? rowObjs : rowObjs.filter(r => !r.a);
  let hidden = 0;
  if (!VS.exp[key] && list.length > lim) { hidden = list.length - lim; list = list.slice(0, lim); }
  let html = tbl(headers, list.map(r => r.tr));
  const bits = [];
  if (hidden) bits.push('<a href="#" onclick="VS.exp[&quot;'+key+'&quot;]=1;route();return false">show '+hidden+' more</a>');
  if (archN && !showA) bits.push('<a href="#" onclick="VS.arch[&quot;'+key+'&quot;]=1;route();return false">show '+archN+' archived</a>');
  if (archN && showA) bits.push('<a href="#" onclick="delete VS.arch[&quot;'+key+'&quot;];route();return false">hide archived</a>');
  if (bits.length) html += '<div class="morebar">'+bits.join(' · ')+'</div>';
  return html;
}
function rowObj(n, extraCell){ return {a: !!n.archived, tr: nodeRow(n, extraCell)}; }
const NODE_HD = ['Type','Title','Status',
  '<span title="Node version — bumps on every content edit (affirm-only restamps excepted)">V</span>',
  '<span title="Last event in this node’s log; bold = within 24h">Updated</span>'];
const STAMP_HD = '<span title="What this ref was written against — the target’s version (or file blob) at link time. Behind = the target moved since; review, then affirm.">Stamp</span>';
function nodeRow(n, extraCell){
  return '<tr'+(n.archived?' class="arch" title="archived — settled and demoted from default surfaces; still fully linked"':'')+'><td class="ty">'+esc(n.type)+(n.kind?' · '+esc(n.kind):'')+'</td>'
    + '<td>'+titleLink(n)+'</td><td>'+bubble(n.status)+'</td>'
    + '<td class="vcol">v'+n.v+'</td>'+whenCell(updatedOf(n))
    + (extraCell !== undefined ? '<td>'+extraCell+'</td>' : '') + '</tr>';
}
function byUpdatedDesc(a,b){ return updatedOf(b).localeCompare(updatedOf(a)); }
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
  if (cur !== e.at) return {sev: 4, label: 'drifted ('+e.at+' → '+cur+')'};
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
function statusDot(sev){ return '<span class="'+(sev===0?'ok':'sev'+sev)+'"><span class="dot"></span></span>'; }
function mdInline(s){
  return s.replace(/`([^`]+)`/g, '<code>$1</code>')
          .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
          .replace(/\*([^*]+)\*/g, '<em>$1</em>');
}
function md(src){
  const lines = esc(src).split('\n');
  let out = [], inCode = false, inList = false, inTable = false;
  const close = () => {
    if (inList) { out.push('</ul>'); inList = false; }
    if (inTable) { out.push('</tbody></table>'); inTable = false; }
  };
  for (const ln of lines) {
    if (ln.startsWith('```')) { close(); out.push(inCode ? '</pre>' : '<pre>'); inCode = !inCode; continue; }
    if (inCode) { out.push(ln); continue; }
    if (ln.trim().startsWith('|')) {
      if (inList) { out.push('</ul>'); inList = false; }
      const cells = ln.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map(c => c.trim());
      if (cells.every(c => /^:?-{2,}:?$/.test(c) || c === '')) continue;
      if (!inTable) { out.push('<table class="tbl"><tbody>'); inTable = true; }
      out.push('<tr>' + cells.map(c => '<td>'+mdInline(c)+'</td>').join('') + '</tr>');
      continue;
    }
    let m;
    if (m = ln.match(/^(#+) (.*)/)) {
      close();
      const lvl = Math.min(m[1].length + 1, 5);
      out.push('<h'+lvl+'>'+mdInline(m[2])+'</h'+lvl+'>');
      continue;
    }
    if (/^\s*[-*] /.test(ln)) {
      if (inTable) { out.push('</tbody></table>'); inTable = false; }
      if (!inList) { out.push('<ul>'); inList = true; }
      out.push('<li>'+mdInline(ln.replace(/^\s*[-*] /, ''))+'</li>');
      continue;
    }
    if (/^> ?/.test(ln) && ln.startsWith('>')) { close(); out.push('<blockquote>'+mdInline(ln.replace(/^> ?/, ''))+'</blockquote>'); continue; }
    if (/^---+\s*$/.test(ln)) { close(); out.push('<hr>'); continue; }
    if (/^\s*$/.test(ln)) { close(); continue; }
    close();
    out.push('<p>'+mdInline(ln)+'</p>');
  }
  close();
  if (inCode) out.push('</pre>');
  return out.join('\n');
}
function evDetail(ev){
  if (ev.op==='create') return 'create "'+esc(ev.title||'')+'"';
  if (ev.op==='link') return 'link -['+esc(ev.rel||'?')+']→ '+esc(ev.to||'?');
  if (ev.op==='set') return 'set '+esc(ev.fields ? ev.fields.join(', ') : (ev.field ? ev.field+'→'+ev.to : ''));
  if (ev.op==='affirm') return 'affirm ×'+(ev.restamped||0);
  if (ev.op==='body') return 'body edited';
  return esc(ev.op||'?');
}

function viewMap(){
  let html = '';
  const recent = DATA.events.slice().filter(e => e.ts).sort((a,b) => b.ts.localeCompare(a.ts)).slice(0, 8);
  html += '<div class="panel"><h2>Recent activity</h2>'
    + tbl(['When','Node','Event'], recent.map(ev => {
        const n = byId[ev.node];
        return '<tr>'+whenCell(ev.ts)+'<td>'+(n ? titleLink(n) : '<span class="id">'+esc(ev.node||'?')+'</span>')
          + '</td><td class="evd">'+evDetail(ev)+(ev.note ? ' — '+esc(ev.note) : '')+'</td></tr>';
      }))
    + '</div>';
  const areas = DATA.nodes.filter(n => n.type==='area' && n.status!=='retired')
    .sort((a,b) => a.title.localeCompare(b.title));
  html += '<div class="tiles">';
  for (const a of areas) {
    const attached = DATA.nodes.filter(n => n.type!=='area' && areasOf(n).includes(a.id));
    const dec = attached.filter(n => n.type==='decision' && n.status==='in-force').sort(byUpdatedDesc);
    const claims = attached.filter(n => n.type==='claim' && !TENSE.dead.includes(n.status));
    const itemsB = attached.filter(n => n.type==='item' && TENSE.becoming.includes(n.status)).sort(byUpdatedDesc);
    const itemsD = attached.filter(n => n.type==='item' && n.status==='done');
    const threads = attached.filter(n => n.type==='thread' && ['open','queued','parked'].includes(n.status)).sort(byUpdatedDesc);
    const docs = attached.filter(n => n.type==='doc');
    const coupling = {};
    attached.forEach(n => (n.edges||[]).forEach(e => {
      if (e.rel!=='depends-on' && e.rel!=='supports') return;
      const t = byId[e.to]; if (!t) return;
      areasOf(t).forEach(a2 => { if (a2!==a.id) coupling[a2]=(coupling[a2]||0)+1; });
    }));
    html += '<div class="tile"><h2><a href="#/n/'+a.id+'">'+esc(a.title)+'</a></h2>'
      + '<p class="charter">'+esc((a.body||'').split('\n')[0])+'</p>';
    const builtRows = dec.concat(claims).concat(itemsD).sort(byUpdatedDesc);
    html += '<div class="sect"><div class="hd" title="What is settled in this area: decisions in force, live claims, shipped work.">Built — decisions in force ('+dec.length
      +') · claims ('+claims.length+') · shipped ('+itemsD.length+')</div>'
      + collapsible('built-'+a.id, builtRows.map(n => rowObj(n)), NODE_HD)+'</div>';
    if (itemsB.length || threads.length) {
      html += '<div class="sect becoming"><div class="hd" title="What is in motion or planned here: work items by status, and threads awaiting the user. Dashed = not yet settled.">Becoming</div>'
        + collapsible('bec-'+a.id, itemsB.concat(threads).map(n => rowObj(n)), NODE_HD)+'</div>';
    }
    const cp = Object.entries(coupling);
    if (cp.length) html += '<div class="coupling"><span title="Derived coupling: work filed in this area holds depends-on or supports edges into nodes filed in these areas. ×N = how many such edges. Computed from real edges, never hand-drawn.">leans on</span>: '
      + cp.map(([id,c]) => '<a href="#/n/'+id+'">'+esc(byId[id].title)+'</a> ×'+c).join(' · ')+'</div>';
    if (docs.length) html += '<div class="coupling"><span title="Registered prose attached to this area — journal entries, spike reports, design docs. Click to see what cites them.">docs</span>: '
      + docs.map(d => '<a href="#/n/'+d.id+'">'+esc(d.title)+'</a>').join(' · ')+'</div>';
    html += '</div>';
  }
  const unfiled = DATA.nodes.filter(n => n.type!=='area' && areasOf(n).length===0);
  if (unfiled.length) {
    html += '<div class="tile"><h2>unfiled</h2><p class="charter">nodes with no area attachment — the map cannot place what nobody filed</p>'
      + collapsible('unfiled', unfiled.sort(byUpdatedDesc).map(n => rowObj(n)), NODE_HD)+'</div>';
  }
  html += '</div>';
  return html;
}

function eventsFor(id, afterV){
  return DATA.events.filter(ev => ev.node===id && (ev.v||0) > afterV)
    .map(ev => 'v'+ev.v+' · '+relTime(ev.ts||'')+' · '+evDetail(ev)+(ev.note ? ' — '+esc(ev.note) : ''));
}

function viewNode(id){
  const n = byId[id];
  if (!n) return '<p class="empty">no node '+esc(id)+'</p>';
  const areas = areasOf(n);
  let html = '';
  if (areas.length) html += '<span class="crumb">'+areas.map(a => '<a href="#/n/'+a+'">'+esc(byId[a].title)+'</a>').join(' · ')+'</span>';
  html += '<div class="node"><h2>'+esc(n.title)+'</h2>'
    + '<p class="meta"><span class="id">'+esc(n.id)+'</span> · '+esc(n.type)+' · v'+n.v
    + ' · '+bubble(n.status)+' · '+esc(n.provenance)
    + (n.ratified ? ' · RATIFIED by '+esc(n.ratified.by)+' '+esc(n.ratified.date) : '')
    + (n.kind ? ' · '+esc(n.kind) : '') + (n.method ? ' · method: '+esc(n.method) : '')
    + (n.path ? ' · <code>'+esc(n.path)+'</code>' : '')
    + '<br>created '+esc(n.created.split('T')[0])+' · updated '+esc(relTime(updatedOf(n)))
    + ' · actor '+esc(n.actor)+'</p>';
  if ((n.acceptance||[]).length) html += '<h3 class="part">Acceptance</h3><ul style="margin:0;padding-left:18px">'
    + n.acceptance.map(a => '<li>'+esc(a)+'</li>').join('')+'</ul>';
  if (n.body) html += '<div class="body">'+esc(n.body)+'</div>';
  if (n.archived) html += '<p class="meta">⚑ ARCHIVED — settled and demoted from default surfaces; every edge and query still reaches it.</p>';
  if (n.type === 'doc' && n.path && DATA.doc_content[n.path] !== undefined) {
    html += '<h3 class="part">Document — <code>'+esc(n.path)+'</code></h3>'
      + '<div class="mdview">'+md(DATA.doc_content[n.path])+'</div>';
  }
  const bl = blockers(n);
  if (bl.length) html += '<h3 class="part">Blocked on</h3>'+tbl(NODE_HD, bl.map(x => nodeRow(x)));
  if ((n.edges||[]).length) {
    html += '<h3 class="part">Edges <span class="arrow">→</span></h3>';
    const rows = [];
    for (const e of n.edges) {
      const s = edgeState(e);
      const stamp = statusDot(s.sev)+'<span class="rel">'+esc(s.label)+'</span>';
      if (e.to.startsWith('file:')) {
        rows.push({a: false, tr: '<tr><td class="rel">'+esc(e.rel)+'</td><td class="ty">file</td>'
          + '<td colspan="2"><code>'+esc(e.to.slice(5))+'</code></td><td></td><td>'+stamp+'</td></tr>'});
      } else {
        const t = byId[e.to];
        if (t) {
          let tr = '<tr'+(t.archived?' class="arch"':'')+'><td class="rel">'+esc(e.rel)+'</td><td class="ty">'+esc(t.type)+'</td>'
            + '<td>'+titleLink(t)+'</td><td>'+bubble(t.status)+'</td>'+whenCell(updatedOf(t))
            + '<td>'+stamp+'</td></tr>';
          if (s.sev===1 || s.sev===3) {
            eventsFor(e.to, e.at).forEach(d =>
              tr += '<tr'+(t.archived?' class="arch"':'')+'><td></td><td colspan="5" class="delta">· '+d+'</td></tr>');
          }
          rows.push({a: !!t.archived, tr});
        } else {
          rows.push({a: false, tr: '<tr><td class="rel">'+esc(e.rel)+'</td><td class="ty">?</td>'
            + '<td colspan="3"><span class="id">'+esc(e.to)+'</span> (missing)</td><td>'+stamp+'</td></tr>'});
        }
      }
    }
    html += collapsible('edges-'+n.id, rows, ['Rel','Type','Target','Status','Updated',STAMP_HD]);
  }
  const inbound = backlinks[n.id]||[];
  if (inbound.length) {
    html += '<h3 class="part"><span class="arrow">←</span> Backlinks</h3>';
    const rows = inbound.map(b => {
      const m = byId[b.from];
      const stale = (typeof b.at==='number' && n.v > b.at)
        ? statusDot(3)+'<span class="rel">cited at v'+b.at+', this is v'+n.v+'</span>'
        : statusDot(0)+'<span class="rel">current</span>';
      return {a: !!m.archived, tr: '<tr'+(m.archived?' class="arch"':'')+'><td class="rel">'+esc(b.rel)+' ←</td><td class="ty">'+esc(m.type)+'</td>'
        + '<td>'+titleLink(m)+'</td><td>'+bubble(m.status)+'</td>'+whenCell(updatedOf(m))
        + '<td>'+stale+'</td></tr>'};
    });
    html += collapsible('back-'+n.id, rows, ['Rel','Type','From','Status','Updated',
      '<span title="Whether the citing node’s stamp still matches this node’s version — stale means the citer has not reviewed this node’s newer state.">Citation</span>']);
  }
  html += '</div>';
  return html;
}

function panelT(title, headers, rows, empty){
  return '<div class="panel"><h2>'+title+'</h2>'
    + (rows.length ? tbl(headers, rows) : '<p class="empty">'+empty+'</p>') + '</div>';
}

function viewState(){
  const threadsQ = DATA.nodes.filter(n => n.type==='thread' && n.status==='queued');
  const queue = threadsQ.filter(n => blockers(n).length===0).sort(byUpdatedDesc);
  const blockedQ = threadsQ.length - queue.length;
  const ready = DATA.nodes.filter(n => n.type==='item' && n.status==='ready' && blockers(n).length===0).sort(byUpdatedDesc);
  const shaping = DATA.nodes.filter(n => n.type==='item' && ['sketch','shaped'].includes(n.status)).sort(byUpdatedDesc);
  const behind = behindAll();
  const sevLabel = {1:'upstream refuted/superseded',2:'dangling',3:'target changed',4:'file drifted'};
  let html = panelT('<span title="The complete list of what is owed by the user: queued threads whose prerequisites have landed. Agents cannot settle these (C3) — only a user ruling resolves them.">Owed to you — answerable now</span>'
      +(blockedQ ? ' ('+blockedQ+' more blocked on spikes)' : ''),
      NODE_HD, queue.map(n => nodeRow(n)), 'nothing awaits the user');
  html += panelT('Ready to dispatch', NODE_HD, ready.map(n => nodeRow(n)), 'nothing dispatchable');
  html += panelT('Shaping — upcoming work', NODE_HD.concat('Blocked on'),
    shaping.map(n => nodeRow(n, blockers(n).map(x => titleLink(x)).join(', ') || '—')), 'nothing sketched');
  html += panelT('Behind — stale refs', ['Sev','Source','Rel','Target','Issue'],
    behind.map(x =>
      '<tr><td>'+statusDot(x.s.sev)+'</td><td>'+titleLink(x.n)+'</td>'
      + '<td class="rel">'+esc(x.e.rel)+'</td>'
      + '<td>'+(x.e.to.startsWith('file:') ? '<code>'+esc(x.e.to.slice(5))+'</code>' : (byId[x.e.to] ? titleLink(byId[x.e.to]) : esc(x.e.to)))+'</td>'
      + '<td class="rel">'+esc(sevLabel[x.s.sev])+': '+esc(x.s.label)+'</td></tr>'), 'every ref current');
  const unver = DATA.nodes.filter(n => n.type==='claim' && n.provenance==='assistant' && n.status==='asserted' && !n.method).sort(byUpdatedDesc);
  html += panelT('Unverified assistant claims', NODE_HD, unver.map(n => nodeRow(n)), 'none');
  return html;
}

function viewAll(){
  const order = ['area','decision','claim','thread','item','doc'];
  let html = '';
  for (const ty of order) {
    const ns = DATA.nodes.filter(n => n.type===ty).sort(byUpdatedDesc);
    if (ns.length) html += '<div class="panel"><h2>'+ty+' ('+ns.length+')</h2>'
      + collapsible('all-'+ty, ns.map(n => rowObj(n)), NODE_HD) + '</div>';
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
{
  const owed = DATA.nodes.filter(n => n.type==='thread' && n.status==='queued' && blockers(n).length===0).length;
  if (owed > 0) {
    const nav = document.getElementById('nav-state');
    nav.innerHTML = 'State · <b>'+owed+' for you</b>';
    nav.title = owed+' thread(s) await your ruling — agents cannot settle them';
  }
}
document.getElementById('gen').textContent = 'generated '+DATA.generated+' · '+DATA.nodes.length+' nodes';
const searchEl = document.getElementById('search'), resultsEl = document.getElementById('results');
searchEl.addEventListener('input', () => {
  const q = searchEl.value.trim().toLowerCase();
  if (!q) { resultsEl.hidden = true; return; }
  const hits = DATA.nodes.filter(n =>
    n.id.includes(q) || n.title.toLowerCase().includes(q) || (n.body||'').toLowerCase().includes(q)
  ).slice(0, 20);
  resultsEl.innerHTML = hits.map(n => '<a href="#/n/'+n.id+'"'+(n.archived?' style="opacity:.55"':'')+'><span class="ty">'+esc(n.type)
    +'</span> <span class="id">'+esc(n.id)+'</span> '+esc(n.title)+' '+bubble(n.status)
    +(n.archived?' <span class="rel">[archived]</span>':'')+'</a>').join('')
    || '<a class="empty">no match</a>';
  resultsEl.hidden = false;
});
searchEl.addEventListener('blur', () => setTimeout(() => resultsEl.hidden = true, 200));
route();
</script>
</body>
</html>
"##;
