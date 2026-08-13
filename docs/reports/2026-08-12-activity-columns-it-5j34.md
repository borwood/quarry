# Dispatch report: recent activity carries type and area (it-5j34)

Agent report, registered verbatim by the dispatcher at harvest.

## What changed and the mechanism

Write-set respected: only src/view.rs.

1. In the TEMPLATE's `viewMap()` recent-activity block: header row `['When','Node','Event']` → `['When','Type','Node','Area','Event']`. Each row gains a `tyCell` — `<td class="ty">` rendering `n.type` plus `· n.kind` when present, character-for-character the same register `nodeRow()` uses for the uniform tables — and an `areaCell` — `<td class="areacol">` rendering the node's `about` edges resolved through the existing `areasOf()` helper (already filtered to `type==='area'` targets), each area title as a link to its node, multiple areas joined with ` · `. A missing node (dangling event) or a node with no areas yields empty cells; the Node cell's raw-id fallback is unchanged.
2. One CSS rule added: `.areacol { color: var(--ink2); font-size: 12px; }` — muted metadata register matching the page's `.evd`/crumb tone.
3. Event selection and ordering untouched: same `filter(e => e.ts).sort(ts desc).slice(0, 8)`; only cells added.

## Build and tests

`cargo build --release`: Finished `release` profile in 16.36s, no warnings.

`cargo test` raw result lines (all five binaries):

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.36s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

53 passed, 0 failed total; surface_lint clean — the new cells live in TEMPLATE JS rendering from the data blob, per the dc-nnf5 carve-out. No test pinned the old strip markup — no landing homework.

## Acceptance outcome

"Recent activity rows show type and area columns": met. Page regenerated under the badge; verified by executing the exact new row logic headlessly against the embedded DATA blob — all 8 current rows render Type (`item · feature`, `doc · report`) and Area (`cli`) cells resolved from live about-edges. Not committed, no status flipped, lease intact.

## Reflections (agent's own words)

- Column order was a judgment call the brief left open: Type before Node (mirroring the uniform register, where type leads title), Area between Node and Event, When first since the strip is time-ordered, Event last as the wide prose column. If the strip should lead with Type exactly like NODE_HD, that's a two-minute reorder.
- The Type cell includes `· kind` because that's what the uniform register actually renders. If bare type is wanted, strip the kind clause.
- `.areacol` deliberately omits `white-space: nowrap` (unlike `.ty`/`.rel`) so a multi-area node wraps instead of stretching the table — untested against real multi-area rows since every current recent row has exactly one area.
- The eyeball check is a regex-extraction of DATA plus a re-implementation of the cell expressions, not a browser render — it proves data resolution, not pixel layout. A human glance at the map view is the honest final confirmation.
