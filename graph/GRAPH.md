# graph/

This directory is a quarry work graph. The files under `nodes/` and `log/`
are the artifact of record; any index or rendered view is derived.

Do not edit these files by hand — all writes go through the `q` verbs, which
enforce the schema constraints, bump per-node versions, and stamp edges with
the version of their target. (Host repos should wire a hook denying freehand
edits under this directory.)

Suggested .gitignore lines: `graph/.index/` and `graph/view/`.
