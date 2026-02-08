# ColdFusion and Lucee CFML Support — Documentation & Implementation Gaps

This document identifies the gaps between Fresh's current capabilities and what would be needed to provide proper ColdFusion (CFML) and Lucee support.

## Background

CFML (ColdFusion Markup Language) is used by two major engines: **Adobe ColdFusion** (commercial) and **Lucee** (open-source). The language has two distinct syntax modes — tag-based and script-based — that can be intermixed, plus embedded languages (HTML, SQL, JavaScript, CSS) with `#expression#` interpolation throughout. This makes CFML one of the more complex languages for editor tooling.

### File extensions to support

| Extension | Purpose |
|-----------|---------|
| `.cfm`    | CFML template/page files (primary) |
| `.cfc`    | ColdFusion Component files (classes) |
| `.cfml`   | Alternative template extension |

---

## Gap 1: No Syntax Highlighting Grammar

**Status:** Not present. No `.sublime-syntax` or TextMate grammar for CFML is bundled or documented.

**What exists externally:**

- **jcberquist/sublimetext-cfml** — The actively maintained Sublime Text package with `.sublime-syntax` grammars. Provides two scopes: `text.html.cfm` (tag markup) and `source.cfscript` (script syntax). Available via Package Control.
- **SublimeText/ColdFusion** — Legacy `.tmLanguage` grammar (deprecated, redirects to jcberquist's package). Still used by GitHub Linguist for ColdFusion highlighting.
- **textmate/coldfusion.tmbundle** — Original TextMate bundle with `.tmLanguage` files.

**Gap details:**

1. Fresh needs a self-contained `.sublime-syntax` grammar. The jcberquist grammar may use `extends` or cross-file `include` directives that Fresh does not support (see `docs/plugins/development/language-packs.md` compatibility warning). This needs to be verified and potentially the grammar needs to be flattened into a standalone file.
2. CFML has two distinct syntax modes (tags and script) that ideally need separate grammar contexts or even separate grammar files, similar to how TypeScript support in `fresh-languages` combines TS and JS highlight queries.
3. The language pack docs (`docs/plugins/development/language-packs.md`) only show examples of single-grammar languages. CFML's dual-syntax nature (tag + script) isn't addressed.

**Action needed:**
- Evaluate whether `jcberquist/sublimetext-cfml` can be used standalone or needs flattening.
- Document how to handle a language with two syntax modes in a single language pack.
- Create or adapt a self-contained `.sublime-syntax` grammar for CFML covering both tag and script syntax.

---

## Gap 2: No Tree-Sitter Integration

**Status:** Not present. The `Language` enum in `crates/fresh-languages/src/lib.rs` has 18 languages; CFML is not among them.

**What exists externally:**

- **cfmleditor/tree-sitter-cfml** — A tree-sitter grammar on GitHub using a three-parser architecture:
  - `cfml/` — top-level CFML grammar
  - `cfhtml/` — ColdFusion HTML tag-based syntax
  - `cfscript/` — CFScript (script-based syntax)
- Forked from `tree-sitter-html`, adapted for CFML.

**Gap details:**

1. No `tree-sitter-cfml` Cargo crate exists as a dependency in `Cargo.toml`. This may not exist on crates.io at all.
2. The three-parser architecture is unusual. Fresh's existing tree-sitter integration assumes one parser per language (with TypeScript being the closest exception, combining TS + JS highlight queries). The multi-grammar CFML parser would need a new integration pattern.
3. `fresh-languages/src/lib.rs` would need a new `Language::Cfml` variant with `from_path`, `highlight_config`, `highlight_category`, `id`, `display_name`, `from_id`, and `from_name` implementations.
4. The tree-sitter grammar's maturity and completeness is unknown — the cfmleditor repo may be in early stages.
5. The language pack documentation does not describe how to add tree-sitter support via language packs (tree-sitter is currently compiled-in only). This is a documentation gap independent of CFML.

**Action needed:**
- Evaluate maturity of `cfmleditor/tree-sitter-cfml`.
- Determine whether to compile it in (like the 18 existing languages) or if the language pack system needs to support tree-sitter grammars loaded at runtime.
- Document the multi-parser integration pattern if built-in support is pursued.

---

## Gap 3: No LSP Configuration

**Status:** Not present. No CFML LSP is configured in `docs/features/lsp.md`, no `cfml-lsp.ts` plugin exists, and no `config.json` example for CFML appears in the docs.

**What exists externally:**

| LSP Implementation | Type | Notes |
|---|---|---|
| **Adobe ColdFusion Builder VS Code Extension** (`adobe-cfml-lsp`) | Java-based, proprietary | Bundled with Adobe's VS Code extension. May not be usable standalone via stdio. |
| **Lucee Built-in LSP** | Built into Lucee server | Available in Lucee 7+. Enabled via `lucee.lsp.enabled=true`. Communicates via the running Lucee instance, not a standalone stdio process. |
| **KamasamaK/vscode-cfml** | Regex-based, not a true LSP | Provides hover/completion but via VS Code extension APIs, not a standalone LSP server. |

**Gap details:**

1. Neither Adobe's nor Lucee's LSP appears to be a standalone stdio-based language server that Fresh can launch via `"command"` + `"args": ["--stdio"]`. This is a fundamental compatibility issue with Fresh's LSP model (`docs/plugins/development/language-packs.md` assumes a launchable executable).
2. The `docs/features/lsp.md` "Configuring LSP for a New Language" section does not address languages where the LSP is embedded in a runtime (like Lucee's built-in LSP) rather than being a standalone CLI tool.
3. No `cfml-lsp.ts` error-handling plugin exists (compare to `go-lsp.ts`, `python-lsp.ts`, etc.).
4. The LSP docs don't mention CFML in the "Common LSP Servers" table.
5. CFLint (a standalone CFML linter) could potentially be integrated as a diagnostics source, but Fresh's docs don't describe how to use non-LSP linters alongside or instead of LSP.

**Action needed:**
- Investigate whether Lucee's LSP can be configured to communicate over stdio (or if it only works over TCP/sockets).
- Investigate whether Adobe's LSP server can be extracted from the VS Code extension and run standalone.
- If neither works over stdio, document what Fresh would need to support TCP/socket-based LSP connections (currently not addressed in docs).
- Consider documenting CFLint integration as an alternative for diagnostics.
- Add a CFML entry to `docs/features/lsp.md`.

---

## Gap 4: No Language Configuration Defaults

**Status:** Not present. No CFML-specific language settings (comment styles, indentation, etc.) are documented or configured.

**What CFML needs:**

```
commentPrefix: N/A (CFML doesn't have a universal single-line comment)
blockCommentStart: "<!---" (CFML comments, note: 3 dashes, not HTML's 2)
blockCommentEnd: "--->"
```

**Gap details:**

1. CFML's comment syntax differs from HTML: `<!--- ... --->` (three dashes) vs HTML's `<!-- ... -->` (two dashes). The language pack `commentPrefix` field doesn't accommodate a language where line comments don't exist and only block comments are available (in tag mode). In CFScript mode, `//` and `/* */` are used.
2. The language configuration schema in `docs/plugins/development/language-packs.md` doesn't address languages with context-dependent comment styles (tag mode uses `<!--- --->`, script mode uses `//` and `/* */`).
3. Auto-indentation rules for CFML would need to handle both tag-style nesting (`<cfif>...</cfif>`) and script-style braces (`if { ... }`).
4. No formatter is documented. CFFormat (from the sublimetext-cfml package) exists for CFScript, but there's no widely-adopted standalone CFML formatter.

**Action needed:**
- Extend the language configuration schema documentation to address dual-comment-style languages.
- Define sensible defaults for `.cfm` vs `.cfc` files (`.cfc` files are often pure CFScript).
- Document the lack of a standardized formatter for CFML.

---

## Gap 5: No Embedded Language Support Documentation

**Status:** Fresh's docs don't address embedded language highlighting at all, which is critical for CFML.

**CFML embeds these languages:**

| Context | Embedded Language |
|---|---|
| Everything outside `<cf*>` tags | HTML |
| Inside `<cfquery>` tags | SQL (with `#variable#` interpolation) |
| Inside `<script>` tags | JavaScript |
| Inside `<style>` tags | CSS |
| Inside `<cfscript>` blocks | CFScript (ECMAScript-like) |

**Gap details:**

1. The sublime-syntax format supports `embed` and `include` directives for language injection. Fresh's compatibility notes warn about `extends` but don't clarify whether `embed` is supported.
2. The language pack documentation doesn't describe how to declare embedded language support.
3. Lucee's "tag islands" feature (tag-based syntax inside CFScript blocks) adds another layer of embedding complexity that no existing grammar fully handles.
4. `#expression#` interpolation can appear in any output context (HTML attributes, SQL strings, JS code, CSS values). This is unique to CFML and not covered by any existing language pack pattern.

**Action needed:**
- Document Fresh's support (or lack thereof) for `embed` directives in `.sublime-syntax` grammars.
- Add guidance for language packs that need embedded language highlighting.
- Note CFML-specific embedding challenges (tag islands, hash interpolation) as known limitations.

---

## Gap 6: No `.cfc` Component-Specific Support

**Status:** Not addressed. `.cfc` files have different conventions than `.cfm` files.

**Gap details:**

1. `.cfc` files are frequently written entirely in CFScript (no tags). They would benefit from being detected as CFScript rather than CFML tag syntax. Fresh's `extensions` field in language packs maps to a single grammar — there's no way to map different extensions to different grammars within one language pack.
2. `Application.cfc` is a special framework file with known lifecycle methods (`onApplicationStart`, `onRequestStart`, `onError`, etc.) that an LSP should understand for completion/documentation.
3. Component features (access modifiers, `extends`, `implements`, `property` declarations) need dedicated syntax support.

**Action needed:**
- Determine if the language pack system should support per-extension grammar overrides.
- If not, document that `.cfc` files use the same grammar as `.cfm` files (most existing grammars handle both).

---

## Gap 7: Documentation Doesn't Cover CFML in Any Example

**Status:** No mention of ColdFusion, Lucee, or CFML anywhere in the docs or codebase.

**Affected documentation files:**

| File | Gap |
|---|---|
| `docs/features/lsp.md` | No CFML entry in the LSP server table |
| `docs/plugins/development/language-packs.md` | No CFML example; no guidance for tag-based or multi-syntax languages |
| `crates/fresh-languages/src/lib.rs` | No `Language::Cfml` variant |
| `README.md` | CFML not listed in supported languages |
| `docs/features/lsp.md` "Configuring LSP for a New Language" | Example uses C# (a simple single-syntax language). No example for a language with CFML's complexity. |

**Action needed:**
- Add CFML to the LSP documentation once an LSP integration path is determined.
- Add a "complex language" example to the language pack docs using CFML as the case study.
- Consider adding a note in the language pack docs about languages where no standalone LSP server exists.

---

## Summary: Priority-Ordered Gap List

| Priority | Gap | Effort | Blocker? |
|----------|-----|--------|----------|
| **P0** | Syntax highlighting grammar (Gap 1) | Medium | Yes — without this, no CFML highlighting at all |
| **P0** | Language configuration defaults (Gap 4) | Low | Yes — needed for comment toggling and indentation |
| **P1** | LSP integration (Gap 3) | High | Blocked by LSP availability over stdio |
| **P1** | Embedded language support docs (Gap 5) | Medium | Determines quality of .cfm highlighting |
| **P2** | Tree-sitter integration (Gap 2) | High | Enhances highlighting but syntect fallback works |
| **P2** | .cfc component support (Gap 6) | Low-Medium | Improves developer experience |
| **P3** | Documentation coverage (Gap 7) | Low | Not a functional blocker |

## Recommended Implementation Path

1. **Start with a language pack** — Create a CFML language pack with a self-contained sublime-syntax grammar adapted from `jcberquist/sublimetext-cfml`. This provides immediate syntax highlighting without any changes to Fresh core.
2. **Add language config** — Define comment styles and indentation defaults in the language pack's `package.json`.
3. **Investigate LSP options** — Test whether Lucee's built-in LSP or Adobe's LSP can communicate over stdio. If not, file an issue to support TCP-based LSP connections in Fresh.
4. **Create a `cfml-lsp.ts` plugin** — Following the pattern of `go-lsp.ts`, provide error handling and install instructions.
5. **Evaluate tree-sitter** — If `cfmleditor/tree-sitter-cfml` matures, consider adding compiled-in support.
6. **Update documentation** — Add CFML examples throughout the docs as each layer of support is implemented.
