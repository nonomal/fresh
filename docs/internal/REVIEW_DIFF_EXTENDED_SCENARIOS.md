# Review Diff — Extended UX Scenario Catalogue

Scenarios planned for the second pass of the NN/g usability study.
Scenarios already covered in the first report (`REVIEW_DIFF_NNG_USABILITY_EVAL.md`)
are not repeated here.

Format: each scenario has an **ID**, **Intent** (what we're testing),
**Setup**, **User actions**, and the **Heuristic(s) it probes**.

---

## A. Content / Encoding Edge Cases

### S1 — Unicode & emoji in diff
- **Intent:** Does the renderer mangle multi-byte glyphs or wide chars?
- **Setup:** File containing `日本語`, `🦀`, combining chars, RTL Hebrew.
- **Actions:** Open Review Diff → observe column alignment in both panes.
- **Heuristic:** H3 (standards), H5 (aesthetic).

### S2 — Very long line (2,000 chars on one line)
- **Intent:** Horizontal overflow / wrapping / scrolling behavior.
- **Setup:** Modify a file so a single changed line is 2,000 chars.
- **Actions:** Scroll horizontally; drill into side-by-side.
- **Heuristic:** H5, H1.

### S3 — CRLF vs LF line endings
- **Intent:** Does the diff highlight line-ending-only changes?
- **Setup:** Change file from LF to CRLF (core.autocrlf off).
- **Actions:** Open Review Diff.
- **Heuristic:** H1 (visibility).

### S4 — File with no trailing newline
- **Intent:** Does the "\ No newline at end of file" marker appear?
- **Setup:** Remove final newline from an existing file.
- **Actions:** Open Review Diff.
- **Heuristic:** H3 (standards).

### S5 — Binary file change
- **Intent:** Does the editor crash or show a graceful "binary" message?
- **Setup:** Modify a PNG / small binary file.
- **Actions:** Open Review Diff, drill into side-by-side.
- **Heuristic:** H9 (error handling).

### S6 — Only-additions file (pure new content in existing file)
- **Intent:** Header rendering when `-` count is zero.
- **Setup:** Append 10 lines to an existing file, no removals.
- **Actions:** Observe hunk header.
- **Heuristic:** H3.

### S7 — Only-deletions file
- **Intent:** Symmetric to S6.
- **Setup:** Remove 10 lines from an existing file.
- **Actions:** Observe hunk header, side-by-side alignment.
- **Heuristic:** H3.

---

## B. Repository / Git State Edge Cases

### S8 — Review Diff in non-git directory
- **Intent:** How does the feature fail when git is absent?
- **Setup:** `cd /tmp` (no `.git`), launch fresh, run Review Diff.
- **Actions:** Observe error path.
- **Heuristic:** H9 (error messages).

### S9 — Review Diff with zero changes
- **Intent:** Empty-state layout.
- **Setup:** Clean repo, no modifications.
- **Actions:** Run Review Diff.
- **Heuristic:** H1, H5.

### S10 — File rename detection
- **Intent:** Does git-style rename tracking show up?
- **Setup:** `git mv foo.py bar.py` without content change.
- **Actions:** Open Review Diff.
- **Heuristic:** H3.

### S11 — Chmod-only change
- **Intent:** Does a mode-only change render cleanly?
- **Setup:** `chmod +x file.txt` with no content change.
- **Actions:** Review.
- **Heuristic:** H3.

### S12 — Staged + unstaged changes on the same file
- **Intent:** Split-hunk behavior and grouping.
- **Setup:** Modify file, `git add`, modify again.
- **Actions:** Observe whether both appear and which is "current".
- **Heuristic:** H1.

### S13 — Merge conflict file
- **Intent:** Does Review Diff handle `<<<<<<` markers sanely?
- **Setup:** Force a merge conflict.
- **Actions:** Review.
- **Heuristic:** H9.

---

## C. Terminal / Layout

### S14 — Small terminal (80 × 24)
- **Intent:** Can the split-pane layout survive?
- **Setup:** Relaunch with `tmux new-session -x 80 -y 24`.
- **Actions:** Open Review Diff, toggle focus, drill in.
- **Heuristic:** H5.

### S15 — Very wide terminal (220 cols)
- **Intent:** Does whitespace explode? Is column width capped?
- **Setup:** 220×45 tmux.
- **Actions:** Review.
- **Heuristic:** H5.

### S16 — Resize mid-review
- **Intent:** Regression check for BUG-2.
- **Setup:** 160×45, open review, resize to 100×30.
- **Actions:** `tmux resize-window` or equivalent.
- **Heuristic:** H9.

### S17 — Review Diff tab next to editing buffers
- **Intent:** Tab-bar behavior; does Review Diff survive buffer switches?
- **Setup:** Open two files, then open Review Diff.
- **Actions:** Cycle buffers; return to Review Diff tab.
- **Heuristic:** H2 (freedom).

---

## D. Staging / Comment Workflows

### S18 — Stage then review
- **Intent:** A staged hunk should appear under "Staged", not "Changes".
- **Setup:** `git add happy.py`, then Review Diff.
- **Actions:** Observe section grouping.
- **Heuristic:** H1, H3.

### S19 — Discard hunk confirmation
- **Intent:** Safety of destructive key `d`.
- **Setup:** Open Review Diff with changes.
- **Actions:** Press `d` — expect a confirm dialog.
- **Heuristic:** H9 (error prevention).

### S20 — Add a line comment and verify persistence
- **Intent:** Round-trip test of `c` → inline render (BUG-6 regression).
- **Setup:** Standard changes.
- **Actions:** Focus diff pane, press `c`, type comment, Enter. Verify.
- **Heuristic:** H1.

### S21 — Edit / delete comment
- **Intent:** Undo path for comments.
- **Setup:** After S20, press `x`.
- **Actions:** Observe removal.
- **Heuristic:** H2.

### S22 — Export review to markdown
- **Intent:** Does export succeed and write a sensible file?
- **Setup:** After S20.
- **Actions:** Press `e`; inspect `.review/session.md`.
- **Heuristic:** H4.

### S23 — Export to JSON
- **Intent:** Second export path.
- **Setup:** Use palette → `Review: Export to JSON`.
- **Actions:** Inspect `.review/session.json`.
- **Heuristic:** H4.

---

## E. Navigation / Discoverability

### S24 — Arrow keys vs vim keys in files pane
- **Intent:** Are both bindings reachable? Any inconsistency?
- **Setup:** Any review session.
- **Actions:** Alternate `Up`/`Down` with `j`/`k`.
- **Heuristic:** H7 (flexibility).

### S25 — Home/End/PageUp/PageDown in files pane
- **Intent:** Boundary clamping.
- **Setup:** 5+ files.
- **Actions:** Spam `PageUp`, `End`, `Home`.
- **Heuristic:** H5.

### S26 — `N` (capital) vs `n` key collision
- **Intent:** Easy user mistake — does capital-N "Note" trigger instead of next-hunk?
- **Setup:** Focus diff pane.
- **Actions:** Press `n`, then `N`.
- **Heuristic:** H3, H9.

### S27 — Menu-bar discoverability of Review Diff
- **Intent:** Can a user find the feature without `Ctrl+P`?
- **Setup:** Launch, press `F10`.
- **Actions:** Browse menus (File/Edit/View/Selection/Go/LSP/Help).
- **Heuristic:** H6 (recognition over recall).

### S28 — Help screen
- **Intent:** Is there an in-app help with Review Diff keybindings?
- **Setup:** Press `F1` / check Help menu.
- **Actions:** Search for "review".
- **Heuristic:** H10 (help & documentation).

### S29 — Re-opening Review Diff after close
- **Intent:** Does state reset cleanly or crash?
- **Setup:** Open, close (`q`), reopen via palette.
- **Actions:** Verify listing matches current git state.
- **Heuristic:** H9.

---

## F. Theming / Accessibility

### S30 — NO_COLOR env var
- **Intent:** Does the feature degrade gracefully without color?
- **Setup:** `NO_COLOR=1 fresh`.
- **Actions:** Open Review Diff; verify `-`/`+` glyph still conveys change.
- **Heuristic:** Accessibility, H3.

### S31 — Alternate theme (dark→light or high-contrast)
- **Intent:** Do red/green remain distinguishable?
- **Setup:** Switch theme via settings.
- **Actions:** Review.
- **Heuristic:** Accessibility.

---

## G. Performance / Scale

### S32 — 50-file change set
- **Intent:** Files-pane scrolling and responsiveness with many rows.
- **Setup:** 50 modified files.
- **Actions:** Review, scroll files list.
- **Heuristic:** H4.

### S33 — Single 10,000-line hunk
- **Intent:** Pathological large-hunk rendering.
- **Setup:** Replace entire contents of a 10k-line file.
- **Actions:** Review, scroll.
- **Heuristic:** Performance.

---

# Execution order

The scenarios above number 33. To keep the second pass tractable,
execute the following **high-value subset** in one tmux session:

S9 (empty), S18 (staging), S20–S22 (comment round-trip + export),
S14 (small terminal), S16 (resize), S27 (menu discoverability),
S28 (help), S30 (NO_COLOR), S10 (rename), S6/S7 (pure-add / pure-del),
S32 (many files), S1 (unicode), S2 (long lines), S4 (no newline),
S29 (reopen), S19 (discard confirm), S26 (N/n collision).

Remaining scenarios are left for future expansion passes.
