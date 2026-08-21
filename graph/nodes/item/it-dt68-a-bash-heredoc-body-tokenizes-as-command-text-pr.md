---
id: it-dt68
type: item
title: 'a bash heredoc body tokenizes as command text: prose inside it can mint a plausible false positive'
v: 3
status: sketch
provenance: assistant
created: 2026-08-21T09:20:15Z
actor: claude-fable-5
kind: bug
acceptance:
- 'prose inside a bash heredoc never mints a touched path: the body is consumed whole to its line-initial terminator, in the quoted, bare, and tab-stripped forms alike, and a real write beside the heredoc still parses'
witness:
- line: 'prose inside a bash heredoc never mints a touched path: the body is consumed whole to its line-initial terminator, in the quoted, bare, and tab-stripped forms alike, and a real write beside the heredoc still parses'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Flagged from the agent seat at the it-ap3x landing, 2026-08-21, as the residue that fix could not reach — recorded in cl-zj2c's closing paragraph and lifted here so it is work, not a footnote.

teach::shell_tokens now swallows a PowerShell here-string whole, but a bash heredoc has no delimiter tracking at all: the body of << DELIM ... DELIM tokenizes as ordinary command text. The << itself pushes a boundary, so the desync the PowerShell form suffered does not repeat — but every line of the body is still read as words. The plausibility floor landed at it-ap3x catches debris SHAPES (metacharacters, control chars, sigil-only tokens), so heredoc prose usually dies there. What it cannot catch is prose that looks like a command: a heredoc line reading 'touch foo' mints foo as a perfectly plausible touched path, and the harvester cannot tell it from a real write.

This matters because the dispatcher's own commit road runs through a heredoc — this session switched to git commit -F - <<'EOF' precisely after the PowerShell here-string form produced the it-ap3x debris. So the channel most likely to carry long prose through the parser is the one still unguarded.

Same class as it-ap3x, one channel over, and the cure is the move already made: track the delimiter in shell_tokens and consume the body to its line-initial terminator as one opaque token. The quoted form (<<'EOF') and the bare form (<<EOF) both want it; <<- strips leading tabs from the terminator and wants the same allowance.
