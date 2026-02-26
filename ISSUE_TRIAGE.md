# Fresh Editor - Open Issue Triage

**Date:** 2026-02-26 (updated from 2026-02-06)
**Total open issues (excluding PRs):** ~97
**Focus:** Low-complexity actionable items, duplicates, fixes vs. debatable changes

---

## Changes Since Last Triage (2026-02-06)

- **Total issues grew from ~72 to ~97** (~35 new issues filed, ~10 closed)
- **Closed from previous low-complexity list:** #915 (custom themes), #514 (cursor in whitespace), #482 (CMD+S Mac — superseded by #1036), #542 (auto save), #716 (move line up/down)
- **Possibly resolved:** #1054 (Python chars on Windows) — reporter confirmed fix in v0.2.5, should be closed
- **Active new reporter:** comesuccingfuccsloot filed 4 UI/settings bugs (#1118-#1122) in one day — all well-scoped
- **New maintainer-filed bugs:** #1112, #1113, #1115 — settings UX, session key handling, package manager

---

## Duplicate / Overlapping Issues

| Group | Issues | Recommendation |
|-------|--------|----------------|
| Whitespace visibility | #893 + #664 (if still open) | Merge. Both request configurable whitespace rendering. |
| WASM version | #534 + PR #596 (Design WASM editor) | Close #534. Design doc exists in `docs/wasm.md`. |
| Diff view | #229 + #197 + #432 | #229 is umbrella. Link others as sub-tasks. |
| Mac keybindings | #1036 (default Mac keybindings wrong) + #727 (Ctrl+Delete/Option+Right Mac) + #482 (closed) | #1036 supersedes #482. Combine #1036 + #727 into a Mac keymap audit. |
| Ignored/flaky tests | #184 + #330 | Keep separate, cross-reference. |
| Settings descriptions | #1119 (descriptions truncated) + #1118 (width format mismatch) | Both are settings UI text issues. Could fix together. |
| Code folding | #1122 (big files can't fold) + #900 (code folding feature, if still open) | #1122 may be a consequence of #900 not being implemented. |
| PuTTY issues | #780 (copy/paste) + #1023 (delete char/word bindings) | Both PuTTY terminal escape sequence handling. May share root cause. |
| Column highlight | #1073 (highlight cursor column) relates to #779 (lines after EOF) | Different features but both viewport rendering enhancements. |

---

## Low Complexity - Recommended Fixes (Bugs)

### Tier 1: Highest confidence, smallest scope

| # | Title | Why it's low complexity |
|---|-------|------------------------|
| **#1114** | Cursor position bleeds through drop-down menus | Z-ordering/layering bug. Cursor/selection renders on top of dropdown overlay. Affects all themes. Likely a render order issue in the overlay system. |
| **#1121** | Can't scroll main buffer while "Open file" panel is open | Mouse scroll events are captured by the bottom panel regardless of cursor position. Event routing / focus issue. |
| **#1120** | Keybinding style list doesn't show currently selected value | Missing visual indicator for active selection in Menu > View > Keybinding Style. Small rendering fix. |
| **#1119** | Settings descriptions end abruptly (truncated text) | Text overflow in settings UI — descriptions cut off mid-sentence. Layout/wrapping issue at high resolutions. |
| **#1118** | Settings File Explorer Width: value format doesn't match description | Mismatch between displayed format and actual value format. Documentation or rendering fix. |
| **#938** | Go to line shows wrong line numbers in large files | Indexing bug in `:` command palette prefix. Lines 1-2 correct, 3+ increasingly wrong. Reproducible in 100K+ line files. Likely related to line wrapping calculation. |
| **#899** | JavaScript syntax highlighting bug | TextMate grammar issue. Arrow function with template literal in class property breaks highlighting for rest of file. Fix is in `.tmLanguage` grammar. |
| **#1039** | Comment delimiters don't use comment color in themes | Comment text is correctly colored but delimiter chars (`#`, `//`) use default text color. Syntax theme token scope issue. |
| **#566** | git log can't use j/k | UI hints say "j/k: navigate" but read-only buffer mode blocks input. Key events eaten before navigation handler. |
| **#1113** | Ctrl+Enter in session writes `[13;5u]` as text | Terminal escape sequence for Ctrl+Enter not handled in session attach mode. Kitty keyboard protocol sequence leaking as literal text. Maintainer-filed. |

### Tier 2: Well-defined but may require more investigation

| # | Title | Notes |
|---|-------|-------|
| **#1112** | Settings UI has mouse/keyboard editing UX issues | Two sub-bugs: (1) mouse click offset in scrolled LSP list, (2) Tab Width input field doesn't respond to Enter/Tab/+/- buttons. Maintainer-filed. |
| **#1115** | Package manager navigation issues (macOS) | Can't navigate packages with arrows; Enter inserts newlines in virtual buffer instead of installing. Maintainer-filed. |
| **#851** | Blinking bar cursor only works at line ending | Cursor blink applies only to primary cursor at EOL. At other positions, character highlighting obscures the blink. Partial fix possible with hardware cursor for primary. |
| **#1012** | Scrollbar flashing / nearly invisible | Scrollbar handle changes color when cursor hovers over trough, becoming invisible. Chromebook/Debian terminal. |
| **#1068** | Tab size always 8 | Per-language tab size overrides global setting. Go defaults to 8. UX problem: users can't easily change per-language defaults. Partially fixed but UX concern remains. |
| **#722** | LSP inlay hints sometimes rendered in wrong place | Positioning calculation error. Maintainer-filed. |
| **#431** | Auto-indent creates staircase code (Windows Terminal) | Indent detection fails for some code block patterns, causing cumulative indentation. |
| **#865** | Empty line at bottom of editor wastes space | Status bar area takes a line even when no prompt is active. Could reclaim for content. |
| **#699** | Find Previous with Shift-F3 does not work | Keybinding issue. F3 works, menu-based Find Previous works, but Shift-F3 doesn't trigger. (May still be open.) |
| **#653** | Line numbers out of sync in large file mode | Rendering desync after scrolling. Maintainer-filed. (May still be open.) |
| **#692** | Hover dismissed after mouse exit + re-entry click | Mouse state tracking bug. (May still be open.) |

---

## Low Complexity - Recommended Improvements (Enhancements)

Clear improvements that are non-debatable and well-scoped.

| # | Title | Why it's clear-cut |
|---|-------|--------------------|
| **#1081** | Support `path:line:column` in command palette / open file | Standard format from compiler errors. Ctrl+P should parse `file.rs:319:48` and jump to location. Well-defined behavior. |
| **#546** | Standard y/n keys for exit-without-saving prompt | Currently requires typing + Enter. Standard UX is single keypress y/n. Small prompt input change. |
| **#744** | Add i18n in CLI help message | Project already has full i18n infrastructure (`locales/`). Wire up CLI help strings. |
| **#619** | Add a `.desktop` file | Standard Linux packaging artifact. Just create the file. (May still be open.) |
| **#779** | Display lines after EOF | Show tilde lines or blank space below last line, like Vim/VS Code. |
| **#833** | Suggested changes to PKGINFO | Packaging metadata fix. Small change. |
| **#465** | Add Winget release action | CI/CD addition for Windows distribution. |
| **#875** | Menu shortcuts should use i18n-dependent keys | German "Alt D" instead of "Alt F" based on localized menu labels. Already has i18n system. |
| **#1073** | Highlight current cursor line and column | Standard editor feature. Line highlight likely exists; column highlight is the new part. |

---

## Medium Complexity

Worth doing but require more design or broader changes.

| # | Title | Notes |
|---|-------|-------|
| #959 | Respect `.editorconfig` files | Well-defined spec, needs parsing library + integration with indent settings. |
| #926 | Recent files feature | Needs persistence (file history) + UI in file menu. |
| #1036 | Default macOS keybindings wrong (Cmd vs Ctrl) | Needs platform-aware modifier system (like VS Code's CtrlCmd). Broader than single keybinding. |
| #836 | Syntax highlighting in reference panel | Wire syntect into reference/hover panel renderer. |
| #867 | Keybinding editing UX in settings editor | No easy way to edit keybindings currently. 2 thumbs up. Maintainer-filed. |
| #878 | Add Move file functionality | File operations in explorer. |
| #868 | Buffer-based autocompletion | Complete from current buffer content when no LSP. |
| #611 | Buffer sometimes empty after switching file in explorer | Race condition between file loading and display. |
| #973 | Auto wrap line inconsistent across languages | Line wrapping behavior varies per language. |
| #1057 | Paste in column mode | Column/block paste like Notepad++. Needs multi-cursor paste logic. |

---

## High Complexity / Large Features

| # | Title | Notes |
|---|-------|-------|
| #900 | Code folding | Major feature (if still open). |
| #909 | magit-style git support | Large plugin feature. |
| #826 | Helix mode | Entire modal editing paradigm. |
| #1086 | Persistent Vi mode | Related to #826 but specifically Vi, not Helix. |
| #478 | Neovim plugin compatibility layer | Massive scope. |
| #140 | Three-way merge (IntelliJ-style) | Complex diff + merge UI. |
| #229 | Diff view (full) | Multi-phase implementation (docs exist). |
| #534 | WASM version | ~3-4 week effort per docs. Design exists. |
| #186 | Rendering optimizations | Broad performance work. |
| #160 | Plugin installation UX | Needs package registry/discovery design. |
| #988 | Support DAP (Debug Adapter Protocol) | Full debug integration. |
| #1026 | Support code lens | LSP code lens feature. |
| #1111 | Support vsix (VS Code extensions) | Massive compatibility layer. |

---

## Debatable / Needs Discussion

| # | Title | Why debatable |
|---|-------|---------------|
| #351 | Config format: JSON to YAML/HJSON | Breaking change. Could just add JSON5/comments support. |
| #348 | Use `ty` as Python LSP | Speculative, ty not stable. |
| #570 | Taskfile support | Niche. Questionable ROI. |
| #381 | WakaTime plugin support | Plugin system should handle this natively. |
| #460 | Terminal triggers CrowdStrike alert | Likely false positive. Documentation fix. |
| #528 | Rust edition 2024 | Mechanical but may surface issues. |
| #554 | Nerd font icons in file tree | Detection/fallback is tricky. |
| #236 | Cursor position keybinding in bottom line | UX preference. |
| #1066 | Auto start preview for Tinymist LSP | LSP-specific auto-preview. Niche. |
| #1053 | Opening remote terminal | Vague scope — SSH? Container? |
| #1051 | Rainbow brackets | Debatable visual feature. Popular but polarizing. |

---

## Questions / Support Requests (Not Bugs)

| # | Title | Recommendation |
|---|-------|----------------|
| #473 | How to configure Python LSP (pyright) | Documentation gap. Add to docs/wiki then close. |
| #490 | How to move/copy files in Explorer view | Feature request disguised as question. Convert to enhancement. |
| #1090 | How to configure C++ syntax highlighting with tree-sitter | Documentation/support. Answer and close. |
| #889 | How does search and replace with regex work? | Documentation gap. |
| #1054 | Python chars on Windows 11 | Reporter confirmed fix in v0.2.5. **Should be closed.** |

---

## Platform-Specific Issues

| # | Platform | Title |
|---|----------|-------|
| #780 | PuTTY/Windows→Oracle Linux | Copy/paste not working |
| #1023 | PuTTY | Delete char/word keybindings |
| #376 | SecureCRT | Mouse support not working |
| #477 | macOS SSH | Cannot copy to system clipboard |
| #586 | KDE | Middle-mouse paste not working |
| #784 | Windows ARM | Build support |
| #989 | Android/Termux | LSP fails to autostart |
| #1054 | Windows 11 | Python chars disappear (likely fixed) |

PuTTY issues (#780, #1023) likely share a root cause (terminal escape sequence handling).

---

## New Feature Requests (not previously triaged)

| # | Title | Complexity |
|---|-------|------------|
| #1117 | Markdown handling doesn't engage on language change | Low-Med — file type detection issue |
| #1116 | Markdown de-indent should cycle bullet type in reverse | Low — small enhancement to existing markdown mode |
| #1080 | Gentoo support | Low — packaging/ebuild |
| #1070 | Command to repaint entire console | Low — force redraw command |
| #1038 | Add binary cache to flake nixConfig | Low — Nix packaging |
| #966 | Svelte syntax highlighting | Low — add .tmLanguage |
| #463 | Syntax highlighting: templ | Low — add .tmLanguage |
| #971 | Set multiple LSP servers per language | Medium — config + LSP orchestration |
| #950 | Sidebar with file outline / TOC | Medium — needs tree-sitter/LSP symbols |
| #953 | LSP pop-up not appearing | Medium — needs investigation |
| #1031 | nushell language support | Low — syntax + LSP config |
| #995 | Installation through conda | Low — packaging |
| #789 | Add flatpak to flathub | Low-Med — packaging + review process |
| #394 | Shellcheck integration | Medium — LSP-like integration for shell scripts |
| #391 | Improve Live Grep | Medium — search infrastructure |

---

## Recommended Priority Order for Low-Complexity Work

### Bugs (fix first)
1. **#1114** - Cursor bleeds through dropdowns (visual layering, easy to verify)
2. **#1121** - Can't scroll buffer with open-file panel (event routing)
3. **#938** - Go to line wrong numbers in large files (calculation fix, high engagement)
4. **#1039** - Comment delimiters wrong color (theme token scoping)
5. **#899** - JS syntax highlighting broken (TextMate grammar)
6. **#1120** - Keybinding list missing selection indicator (small rendering fix)
7. **#1119** - Settings descriptions truncated (text overflow)
8. **#1113** - Ctrl+Enter writes escape sequence in sessions (input handling)
9. **#566** - git log j/k navigation broken (read-only buffer input)
10. **#1118** - Settings width format mismatch (display fix)

### Enhancements (then improve)
1. **#1081** - Support `path:line:column` in open file (high utility, standard format)
2. **#546** - y/n exit confirmation (small UX win)
3. **#744** - i18n CLI help messages (infrastructure exists)
4. **#875** - i18n-dependent menu shortcuts (infrastructure exists)
5. **#779** - Lines after EOF display (viewport rendering)
