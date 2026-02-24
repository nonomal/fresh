# Indent Architecture Review: Option B (Layered Providers)

## Executive Summary

This document validates the three proposed architecture alternatives against the
actual codebase and recommends **Option B (VS Code-style Layered Providers)** as
the target architecture, with the tree-sitter layer using **Option A's ancestor
counting algorithm**.

## Current Architecture Analysis

The current indentation system lives in two files:

- **`indent.rs`** (1344 lines) — `IndentCalculator`: tree-sitter + pattern hybrid
- **`indent_pattern.rs`** (332 lines) — `PatternIndentCalculator`: WASM-compatible pattern-only

### Current Call Chain

```
User presses Enter
  → actions.rs:950 calls IndentCalculator::calculate_indent()
      → Try calculate_indent_tree_sitter()     [tree-sitter path]
      → Try calculate_indent_pattern()         [pattern fallback]
      → get_current_line_indent()              [copy-indent fallback]

User types }
  → actions.rs:437 calls IndentCalculator::calculate_dedent_for_delimiter()
      → Try tree-sitter dedent (count @indent ancestors)
      → Fall back to calculate_dedent_pattern() [pattern nesting tracker]
```

### Current Problems Identified

1. **Character-level heuristics interleaved with AST logic**: The tree-sitter
   path in `calculate_indent_tree_sitter` (lines 797-817) checks
   `last_nonws_is_closing_brace` to suppress `@indent` counting. Lines 884-897
   re-check for `:`, `{`, `[`, `(` triggers when delta == 0. These are
   workarounds for the fundamental issue of parsing only `0..cursor_position`.

2. **Truncated parsing window**: `MAX_PARSE_BYTES = 2000` and only content
   *before* cursor is parsed (line 692-699). This means tree-sitter always sees
   incomplete syntax at the cursor edge, producing ERROR nodes frequently.

3. **Duplicated fallback logic**: Both `IndentCalculator` and
   `PatternIndentCalculator` implement overlapping pattern-matching logic
   (`calculate_indent_pattern`, `calculate_dedent_pattern`).

4. **No LSP integration**: Zero references to `onTypeFormatting` or
   `documentOnTypeFormatting` anywhere in the codebase. The LSP client stores
   `ServerCapabilities` but doesn't check for or use formatting capabilities.

5. **No regex-based indent rules**: There are no `increaseIndentPattern` /
   `decreaseIndentPattern` regex rules per language. The current "pattern
   matching" is delimiter-based only (`{ [ ( : } ] )`).

## Validation of Proposed Alternatives

### Option A: Count Ancestors (Tree-sitter Strategy)

**Claim**: Replace "compare captures at two positions" with counting `@indent`
ancestors around the cursor.

**Validation**: The current code already *partially* does ancestor counting
(lines 822-870 in `indent.rs`), but it's hobbled by:
- Only parsing `0..position` (no content after cursor)
- Using `last_nonws_is_closing_brace` to patch around incomplete parses
- Needing indent-trigger detection as a secondary heuristic

**Feasibility**: Sound. If the full buffer is parsed, ancestor counting becomes
the clean algorithm described. The existing `@indent` / `@dedent` captures in
all 18 `indents.scm` files are already structured for this approach.

**Missing**: The current queries only use `@indent` and `@dedent`. To fully
implement Neovim/Helix-style counting, we'd benefit from:
- `@indent.begin` / `@indent.end` — to mark indent scope start/end separately
  (currently both are `@indent`)
- `@indent.branch` — for `else`, `elif`, `catch` etc. that maintain indent
  without increasing it

However, this is **not strictly required** for a first implementation. The
existing `@indent` capture marking block nodes (e.g., `(block)`,
`(statement_block)`) works for ancestor counting if we parse the full buffer.

**Con validated**: Re-parsing the full buffer is more expensive than parsing
2000 bytes. For large files, this needs incremental parsing support (which
tree-sitter already supports via `tree.edit()` + re-parse).

### Option B: Layered Providers

**Claim**: Chain of independent indent providers, each returning `Option<usize>`.

**Validation against codebase**:

| Layer | Current Status | Work Required |
|-------|---------------|---------------|
| 1. LSP `onTypeFormatting` | **Not implemented** — no code references it at all | New: register capability, send request, handle response |
| 2. Tree-sitter ancestor counting | **Partially implemented** — hobbled by heuristics | Refactor: parse full buffer, simplify to pure ancestor counting |
| 3. Regex indent rules | **Not implemented** — only delimiter-based pattern matching exists | New: per-language regex patterns (increaseIndentPattern, etc.) |
| 4. Copy previous line indent | **Implemented** — `get_current_line_indent` | Existing, works |

**Feasibility**: This is the cleanest path forward. The current code is already
implicitly a 2-layer chain (tree-sitter → pattern fallback → copy indent). The
refactor would formalize this as a trait/enum-based provider chain.

**Additional considerations not in the original proposal**:

1. **WASM compatibility**: `PatternIndentCalculator` exists specifically for
   WASM builds where tree-sitter is unavailable. A layered architecture should
   compile-time exclude tree-sitter and LSP layers in WASM builds, keeping
   regex + copy-indent layers.

2. **Provider result types**: `Option<usize>` is too simple. Providers should
   distinguish between "I don't know" (`None`) and "indent should be 0"
   (`Some(0)`). The current code already handles this correctly but the
   distinction should be formalized.

3. **Dedent vs indent**: The current architecture has two separate paths —
   `calculate_indent` (Enter key) and `calculate_dedent_for_delimiter` (typing
   `}`, `]`, `)`). The layered architecture should handle both through the same
   provider chain, with providers receiving an `IndentContext` enum
   (`NewLine { position }` vs `ClosingDelimiter { position, delimiter }`).

### Option C: Minimal Fix

**Claim**: Keep current architecture, just simplify by parsing full buffer and
removing heuristics.

**Validation**: This is effectively "implement Option A without the layered
architecture." It would fix the immediate problems but:
- Leaves the implicit fallback chain informal and hard to extend
- Doesn't create a path for LSP indent support
- Doesn't address the missing regex rules for languages like Python where
  tree-sitter alone doesn't capture all indent semantics (e.g., `\` line
  continuations)

## Recommended Architecture: Option B with Option A's Tree-sitter Strategy

### Proposed Provider Trait

```rust
/// Context for indent calculation
pub enum IndentRequest {
    /// User pressed Enter — calculate indent for new line
    NewLine { position: usize },
    /// User typed a closing delimiter — calculate dedent
    ClosingDelimiter { position: usize, delimiter: char },
}

/// Result from an indent provider
pub enum IndentResult {
    /// Provider determined the indent level
    Indent(usize),
    /// Provider cannot determine indent — pass to next provider
    Pass,
}

/// An indent provider in the chain
pub trait IndentProvider {
    fn calculate(&mut self, buffer: &Buffer, request: &IndentRequest, tab_size: usize) -> IndentResult;
}
```

### Provider Chain

```rust
pub struct IndentChain {
    providers: Vec<Box<dyn IndentProvider>>,
}

impl IndentChain {
    fn calculate(&mut self, buffer: &Buffer, request: &IndentRequest, tab_size: usize) -> usize {
        for provider in &mut self.providers {
            if let IndentResult::Indent(level) = provider.calculate(buffer, request, tab_size) {
                return level;
            }
        }
        0 // final fallback
    }
}
```

### Providers (in priority order)

#### 1. `LspIndentProvider` (new)

```rust
/// Queries LSP server's onTypeFormatting capability
struct LspIndentProvider {
    // Reference to LSP client for the current language
}
```

- Check `ServerCapabilities::document_on_type_formatting_provider`
- Send `textDocument/onTypeFormatting` with trigger character (`\n`, `}`, etc.)
- Parse response edits to extract indent level
- Returns `Pass` if LSP not connected or doesn't support it

**Note**: This is the lowest-effort, highest-value addition. Many LSP servers
(rust-analyzer, gopls, typescript-language-server) support `onTypeFormatting`
and handle edge cases we'd never cover with heuristics.

#### 2. `TreeSitterIndentProvider` (refactored from current)

The core algorithm becomes Option A's ancestor counting:

```rust
/// Pure tree-sitter ancestor counting
struct TreeSitterIndentProvider {
    configs: HashMap<&'static str, (Parser, Query)>,
}
```

Key changes from current implementation:
- Parse the **full buffer** (not just `0..position`), using incremental parsing
- Count `@indent` ancestors at cursor position → that's the indent level
- For `ClosingDelimiter`: indent level = ancestor count - 1
- **Remove all character heuristics**: no `last_nonws_is_closing_brace`, no
  indent trigger detection, no reference line scanning
- Handle ERROR nodes by returning `Pass` (let next provider handle it)

#### 3. `RegexIndentProvider` (new, replaces current pattern matching)

```rust
/// Per-language regex rules (VS Code compatible)
struct RegexIndentProvider {
    rules: HashMap<&'static str, LanguageIndentRules>,
}

struct LanguageIndentRules {
    increase_indent: Regex,  // e.g., r"\{[^}]*$|\b(if|for|while)\b.*:$"
    decrease_indent: Regex,  // e.g., r"^\s*[}\])]"
    indent_next_line: Option<Regex>,  // single-line indent (e.g., if without braces)
    unindented_line: Option<Regex>,   // lines that ignore indent (e.g., #preprocessor)
}
```

This replaces the current `calculate_indent_pattern` / `calculate_dedent_pattern`
with explicit per-language rules. Benefits over current approach:
- Handles Python `\` continuations, `pass`/`return`/`break` dedent
- Handles C preprocessor directives (#include, #define)
- Handles Ruby `end`, Lua `end`, Bash `fi`/`done`/`esac`
- Can be loaded from config files for user customization

#### 4. `CopyIndentProvider` (existing, minimal change)

```rust
/// Copy indent from previous non-empty line
struct CopyIndentProvider;
```

This is the existing `get_current_line_indent` / `find_reference_line_indent`
logic. Always returns `Indent(n)` — never `Pass`.

### Migration Strategy

**Phase 1** — Define the `IndentProvider` trait and `IndentChain`. Wrap existing
code as providers without changing behavior. Wire into `actions.rs`.

**Phase 2** — Refactor `TreeSitterIndentProvider` to parse full buffer and use
pure ancestor counting. Remove character heuristics. Keep pattern fallback as
safety net.

**Phase 3** — Add `RegexIndentProvider` with per-language rules. Migrate
delimiter-based logic from `PatternIndentCalculator`. Update WASM build to use
regex + copy-indent only.

**Phase 4** — Add `LspIndentProvider`. Check `onTypeFormatting` capability
during initialization. Wire through async message channel.

### Additional Things to Consider

1. **Async LSP provider**: LSP requests are async but indent calculation
   happens synchronously during keystroke handling. Options:
   - Fire-and-forget: apply indent immediately from local providers, then
     adjust if LSP responds with a different indent (VS Code does this)
   - Cache: remember last LSP indent response per-file, use cached value
   - Skip LSP for real-time indent, only use it for format-on-save

2. **Incremental tree-sitter parsing**: Parsing the full buffer on every
   keystroke is expensive for large files. The tree-sitter `Tree::edit()` API
   allows incremental re-parsing. The `HighlightEngine` likely already
   maintains a tree that could be shared with the indent provider.

3. **Interaction with format-on-save**: If LSP provides `documentFormatting`,
   indent calculation during typing is less critical — the file gets
   reformatted on save. The layered system should still work for a good typing
   experience, but knowing format-on-save exists reduces the pressure on
   getting every edge case right.

4. **Testing strategy**: Each provider should be independently testable.
   The current test suite (16 tests in `indent.rs`, 4 in `indent_pattern.rs`)
   should be refactored into per-provider test modules. Integration tests
   should verify the full chain with realistic multi-language scenarios.

5. **Configuration**: Users should be able to disable specific layers (e.g.,
   disable LSP indent if their server's formatting is bad) via editor settings.

6. **`@indent.branch` captures**: For languages with `else`/`elif`/`catch`,
   the current `@dedent` capture is overloaded. Adding `@indent.branch` would
   let the tree-sitter provider handle these cases without falling through to
   regex. This is a tree-sitter query enhancement, not an architecture change.

7. **Bracket pair expansion**: The current `actions.rs` (lines 960-975) has
   special logic for expanding `{|}` into `{\n  |\n}` on Enter. This should
   be handled as a special case in the `NewLine` indent request, not as
   separate logic in the action handler.
