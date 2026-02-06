# Fresh Editor - Open Issue Triage

**Date:** 2026-02-06
**Total open issues (excluding PRs):** ~72
**Focus:** Low-complexity actionable items, duplicates, fixes vs. debatable changes

---

## Duplicate / Overlapping Issues

| Group | Issues | Recommendation |
|-------|--------|----------------|
| Whitespace visibility | #893 (add settings for whitespace visibility) + #664 (add more options for whitespace visibility) | Merge into one. Both request configurable whitespace rendering. |
| WASM version | #534 (crazy idea: wasm version) + PR #596 (Design WASM editor) | #534 is a discussion; design work already exists in `docs/wasm.md` and PR #596. Close #534 in favor of the PR/design doc. |
| Diff view | #229 (Diff view) + #197 (line-diff highlighting) + #432 (Diff keyboard shortcuts - IntelliJ compat) | All diff-related. #229 is the umbrella; #197 and #432 are sub-tasks. Link them and close duplicates or convert to checklist items under #229. |
| Ignored/flaky tests | #184 (fix ignored tests) + #330 (flaky tests) | Different root causes but related. Keep separate but cross-reference. |
| Mac keybindings | #482 (CMD+S on Mac) + #727 (Ctrl+Delete / Option+Right on Mac) | Both are Mac-specific keybinding issues. Could be addressed together in a Mac keymap audit. |

---

## Low Complexity - Recommended Fixes (Bugs)

These are clear bugs with well-defined scope, likely fixable with small, targeted changes.

### Tier 1: Highest confidence, smallest scope

| # | Title | Why it's low complexity |
|---|-------|------------------------|
| **#938** | Go to line shows wrong line numbers | Off-by-one or indexing bug in command palette `:` prefix handler. Lines 1-2 work, 3+ are wrong. Likely a single calculation error. |
| **#899** | JavaScript syntax highlighting bug (arrow functions in class properties) | TextMate grammar issue. Arrow function with template literal in class property causes rest of file to highlight as string. Reproducible with `--no-plugins`. Fix is in the JS `.tmLanguage` grammar file. |
| **#699** | Find Previous with Shift-F3 does not work | Keybinding issue. F3 (Find Next) works, menu-based Find Previous works, but Shift-F3 doesn't trigger. Likely a missing or incorrect keybinding entry. |
| **#566** | git log can't use j/k | UI shows "j/k: navigate" hint but buffer is marked "editing disabled" which blocks input. Likely the read-only buffer mode is eating the key events before the navigation handler. |
| **#692** | Hover dismissed after mouse moves outside then clicks inside | Mouse event handling bug. If mouse never leaves hover, clicking inside works. If mouse exits then re-enters and clicks, hover is dismissed. Event state tracking issue. |

### Tier 2: Well-defined but may require more investigation

| # | Title | Notes |
|---|-------|-------|
| **#915** | Custom themes in "Edit Theme" but not "Select Theme" | Theme loading paths differ between the two commands. The select-theme picker likely doesn't scan `~/.config/fresh/themes/`. |
| **#514** | Cursor disappears when moving into whitespace | Cursor rendering in blank lines. Reopened issue (was fixed once, regressed). Windows 11 / PowerShell. |
| **#653** | Line numbers out of sync in large file mode | Rendering bug where line number column desynchronizes from content after scrolling. Filed by maintainer with screenshots. |
| **#722** | LSP inlay hints sometimes rendered in wrong place | Positioning calculation error for inlay hint overlays. Filed by maintainer. |
| **#677** | Can't scroll to the end (split terminal) | Scroll bounds calculation may not account for split terminal dimensions correctly. |
| **#431** | Auto-indent creates staircase code (Windows Terminal) | Indent detection likely fails for certain code block patterns, causing cumulative indentation. |

---

## Low Complexity - Recommended Improvements (Enhancements)

Clear improvements that are non-debatable and well-scoped.

| # | Title | Why it's clear-cut |
|---|-------|--------------------|
| **#619** | Add a `.desktop` file | Standard Linux packaging artifact. Just create the file (reference: Neovim's). |
| **#716** | Move current line/selection up and down | Standard editor feature (VS Code Alt+Up/Down). Well-defined behavior, maintainer-filed. |
| **#546** | Standard y/n keys for exit-without-saving prompt | Currently requires typing + Enter. Standard UX is single keypress y/n. Small change to prompt input handling. |
| **#744** | Add i18n in CLI help message | Project already has full i18n infrastructure (`locales/`). Just wire up CLI help strings. |
| **#779** | Display lines after EOF | Small visual enhancement: show tilde lines or blank space below last line of file, like Vim/VS Code. |
| **#833** | Suggested changes to PKGINFO | Packaging metadata fix. Likely a few-line change. |
| **#465** | Add Winget release action | CI/CD addition for Windows package distribution. |
| **#482** | CMD+S should use Save on Mac | Platform-standard keybinding. May be part of a broader Mac keymap file. |

---

## Medium Complexity

Worth doing but require more design or broader changes.

| # | Title | Notes |
|---|-------|-------|
| #959 | Respect `.editorconfig` files | Well-defined spec, but needs parsing library + integration with tab/indent settings. |
| #926 | Recent files feature | Needs persistence (file history) + UI in file menu. |
| #542 | Auto save | Needs timer + dirty-buffer tracking + configuration. Session persistence work may overlap. |
| #836 | Syntax highlighting in reference panel | Requires wiring syntect into the reference/hover panel renderer. |
| #702 | Search state per-buffer instead of global | Architecture change: move SearchState from global to per-buffer. |
| #700 | Testing macros (record/playback broken) | Macro system bugs. Needs investigation of the recording/playback pipeline. |
| #620 | Multi-select not consistent with cursor | Multi-cursor behavior inconsistencies. Needs careful UX decisions. |
| #611 | Buffer sometimes empty after switching file in explorer | Race condition between file loading and buffer display. |
| #371 | macOS path expansion while saving | Path handling (`~` expansion, etc.) in save dialog on macOS. |

---

## High Complexity / Large Features

Significant effort, architecture changes, or debatable design decisions.

| # | Title | Notes |
|---|-------|-------|
| #900 | Code folding | Major feature: fold regions, persistence, depth-based folding. |
| #909 | magit-style git support | Large plugin feature. |
| #826 | Helix mode | Entire modal editing paradigm. |
| #478 | Neovim plugin compatibility layer | Massive scope. |
| #140 | Three-way merge (IntelliJ-style) | Complex diff + merge UI. |
| #229 | Diff view (full) | Multi-phase implementation (docs exist). |
| #534 | WASM version | ~3-4 week effort per docs. Design exists. |
| #186 | Rendering optimizations | Broad performance work. |
| #160 | Plugin installation UX | Needs package registry/discovery design. |

---

## Debatable / Needs Discussion

Issues where the right approach is unclear or subjective.

| # | Title | Why debatable |
|---|-------|---------------|
| #351 | Change config format from JSON to YAML/HJSON | Breaking change. JSON is standard, HJSON/YAML add dependencies. Could just add comments support. |
| #348 | Use `ty` as Python LSP | `ty` not yet stable. Speculative. |
| #570 | Taskfile support | Niche build system. Questionable ROI. |
| #381 | WakaTime plugin support | Third-party time-tracking integration. Plugin system should handle this. |
| #460 | Terminal opening triggers CrowdStrike alert | Likely a CrowdStrike false positive, not a Fresh bug. May need documentation. |
| #528 | Upgrade to Rust edition 2024 | Mechanical but may surface issues. Needs testing. |
| #554 | Nerd font icons in file tree | Nice-to-have, but detection/fallback is tricky. |
| #236 | Cursor position keybinding in bottom line | UX preference. |

---

## Questions / Support Requests (Not Bugs)

These could potentially be closed or converted to documentation improvements.

| # | Title | Recommendation |
|---|-------|----------------|
| #473 | How to configure Python LSP (pyright) | Documentation gap. Add to docs/wiki then close. |
| #490 | How to move/copy files in Explorer view | Feature request disguised as question. If not supported, convert to enhancement. |
| #554 | Nerd font icons in file tree? | Question. Answer and close or convert to feature request. |

---

## Platform-Specific Issues

| # | Platform | Title |
|---|----------|-------|
| #783 | PuTTY/Oracle Linux | CTRL+END not working |
| #780 | PuTTY/Windows→Oracle Linux | Copy/paste not working |
| #376 | SecureCRT | Mouse support not working |
| #477 | macOS SSH | Cannot copy to system clipboard |
| #586 | KDE | Middle-mouse paste not working |
| #784 | Windows ARM | Build support |

These are mostly terminal emulator compatibility issues. PuTTY issues (#783, #780) may share a root cause (terminal escape sequence handling).

---

## Recommended Priority Order for Low-Complexity Work

1. **#938** - Go to line bug (clear calculation error, user-facing, 8 comments = high engagement)
2. **#899** - JS syntax highlighting (TextMate grammar fix, well-reproduced)
3. **#699** - Shift-F3 Find Previous (keybinding fix, 6 comments)
4. **#566** - git log j/k navigation (input handling in read-only buffer)
5. **#619** - Add .desktop file (packaging, standalone)
6. **#716** - Move line up/down (standard feature, maintainer-requested)
7. **#915** - Custom theme selection (theme path scanning)
8. **#546** - y/n exit confirmation (small UX fix)
9. **#692** - Hover dismiss on re-entry click (mouse state bug)
10. **#653** - Line number desync in large files (rendering bug, maintainer-filed)
