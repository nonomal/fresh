# Memory Leak Investigation: PageDown Scroll OOM

## Summary

Scrolling through a large file (~9K lines, 372KB) with repeated PageDown causes Fresh to consume 12+ GB of memory and get OOM-killed. The issue was reported by users running Fresh in terminals like Konsole and Kitty.

## How to Reproduce

1. Open `crates/fresh-editor/tests/fixtures/large.rs` in Fresh (any terminal)
2. Hold PageDown
3. At some point while scrolling, the editor freezes and memory explodes from ~80MB to 12+ GB within a second
4. The Linux OOM killer terminates the process

The issue reproduces consistently. It is **not** a gradual accumulation — memory is stable until a specific scroll position, then explodes in a single render frame.

## What We Found

### Kernel OOM logs confirm the explosion

```
Out of memory: Killed process (fresh) total-vm:27023576kB, anon-rss:13071312kB
```

Three separate OOM kills all showed ~13 GB anonymous RSS.

### Heaptrack profiling (debug build, ~313 MB captured before SIGTERM)

The top allocation sites, all in the render path:

| Leaked | Call site |
|--------|-----------|
| 224 MB | `SplitRenderer::build_view_data` → `apply_wrapping_transform` → `emit_break_with_indent` (split_rendering.rs:3732) |
| 56 MB  | Same path, different allocation within `emit_break_with_indent` |
| ~1 MB  | `TextMateEngine::highlight_viewport` → `full_parse` → `scope_stack_to_category` |
| ~1 MB  | `SplitRenderer::scrollbar_visual_row_counts` → `TextBuffer::get_line` → `wrap_line` |

### The render call chain

```
Editor::render()                           (render.rs:440)
  → Terminal::draw()                       (ratatui)
    → SplitRenderer::render_content()      (split_rendering.rs:1273)
      → render_buffer_in_split()           (split_rendering.rs:6208)
        → compute_buffer_layout()          (split_rendering.rs:5821)
          → build_view_data()              (split_rendering.rs:2775)
            → apply_wrapping_transform()   (split_rendering.rs:3821)
              → emit_break_with_indent()   (split_rendering.rs:3732)
                → " ".repeat(line_indent)  ← allocates new String EVERY CALL
```

### Two separate problems identified

#### 1. `scrollbar_visual_row_counts()` — O(n) per frame (split_rendering.rs:2379)

When line wrapping is enabled (on by default), this function iterates **every line in the entire file** on **every render frame** to compute scrollbar position:

```rust
for line_idx in 0..line_count {
    let line_content = state.buffer.get_line(line_idx)  // allocates String per line
        ...to_string();
    let segments = wrap_line(&line_content, &wrap_config);  // allocates Vec per line
    total_visual_rows += segments.len().max(1);
}
```

For large.rs (9,223 lines), this creates ~18K allocations per frame. Called from `render_content` (line 1317) on every frame.

#### 2. `emit_break_with_indent()` — unbounded allocation in wrapping (split_rendering.rs:3718)

```rust
fn emit_break_with_indent(wrapped: &mut Vec<ViewTokenWire>, ...) {
    wrapped.push(ViewTokenWire { kind: ViewTokenWireKind::Break, ... });
    if line_indent > 0 {
        wrapped.push(ViewTokenWire {
            kind: ViewTokenWireKind::Text(" ".repeat(line_indent)),  // new String alloc
            ...
        });
    }
}
```

This is called from `apply_wrapping_transform()` for every line break in every wrapped line, every frame. The heaptrack showed 1.6 million calls to this allocator in an 8-minute session.

#### 3. Possible trigger: ratatui `autoresize()`

The explosion only reproduces with `CrosstermBackend`, not `TestBackend`. One theory: ratatui's `Terminal::draw()` calls `autoresize()` which queries the real terminal size via `crossterm::terminal::size()`. If this returns unexpected dimensions (e.g., 0×0 or very large values), the wrapping code could produce enormous output. This needs investigation.

### What does NOT leak

- `build_view_data()` returns a local `ViewData` that is dropped each frame — no cross-frame accumulation.
- `checkpoint_states` in `TextMateEngine` grows as you scroll (one entry per 256 bytes of file), but this accounts for <1MB over the whole file.

## The Test

File: `crates/fresh-editor/tests/e2e/memory_scroll_leak.rs`

Three tests:

1. **`test_page_down_to_bottom_no_memory_explosion`** — Uses `TestBackend` (ratatui in-memory buffer). Scrolls to bottom checking RSS every 10 PageDowns. Currently **passes** (3 MB growth). Serves as a regression guard for the render logic.

2. **`test_page_down_to_bottom_crossterm_backend_no_memory_explosion`** — Uses `CrosstermBackend<Sink>` which exercises the real ANSI diff/output codepath. **Reproduces the OOM** — gets killed around PageDown #240. Uses `prlimit` to cap memory at 1GB so it aborts instead of OOM-killing the test runner.

The test opens `tests/fixtures/large.rs` (9,223-line Rust file, copied from editor-benchmark repo), does repeated PageDown with `editor.handle_key()` + `terminal.draw()`, and checks RSS at intervals.

### Running the tests

```bash
# TestBackend test (passes, ~20s)
cargo test --test e2e_tests e2e::memory_scroll_leak::test_page_down_to_bottom_no -- --nocapture

# CrosstermBackend test (reproduces OOM, ~30s until kill)
cargo test --test e2e_tests e2e::memory_scroll_leak::test_page_down_to_bottom_crossterm -- --nocapture
```

## Suggested Next Steps

### Investigation needed

1. **Why does CrosstermBackend trigger the explosion but TestBackend doesn't?**
   - Both call `editor.render(frame)` identically
   - Check if ratatui's `autoresize()` inside `Terminal::draw()` changes the frame dimensions mid-scroll when using CrosstermBackend with a non-terminal writer (like `sink()`)
   - If autoresize returns 0×0 or a huge size, check what happens in `WrapConfig::new()` and `wrap_line()` with `width=0`
   - Try disabling autoresize: `terminal.draw()` → manually call the render closure with a fixed-size frame

2. **Find the exact line/position that triggers the explosion**
   - Add logging inside the CrosstermBackend test to print the frame area dimensions on each draw
   - Add a guard in `wrap_line()` to panic if segment count exceeds a reasonable limit (say, 10000) — this will give a backtrace pointing to the exact trigger

### Fixes to implement

3. **Cache `scrollbar_visual_row_counts()` (high priority)**
   - This is O(n) per frame where n = total lines in file, even when only the viewport changed
   - Cache `(total_visual_rows, top_visual_row)` keyed by `(buffer_version, top_byte, viewport_width, wrap_column)`
   - Invalidate on: buffer edit, terminal resize, wrap settings change
   - This is clearly wrong regardless of the OOM bug — for a 100K-line file it does 100K string allocations per frame at 16fps

4. **Guard against pathological wrapping**
   - In `wrap_line()` (line_wrapping.rs:129): if `width == 0`, return a single unwrapped segment instead of producing one segment per character
   - In `apply_wrapping_transform()`: add a sanity limit on the number of Break tokens emitted — if it exceeds some multiple of the visible line count, bail out
   - In `emit_break_with_indent()`: consider reusing a shared indent string instead of `" ".repeat()` on every call

5. **Reuse wrapping allocations across frames**
   - `apply_wrapping_transform()` creates a new `Vec<ViewTokenWire>` every call
   - Consider passing a reusable buffer that gets `.clear()`ed between frames instead of allocating fresh

6. **Bound `checkpoint_states` in TextMateEngine**
   - Currently grows without limit as you scroll through the file (one entry per 256 bytes)
   - Add LRU eviction or distance-based pruning (remove checkpoints far from current viewport)
   - Minor compared to the other issues but worth fixing for very large files

### Architecture note

The `scrollbar_visual_row_counts` problem reveals a design tension: the scrollbar needs to know the total visual row count (which depends on wrapping every line), but computing this is expensive. Other editors solve this by:
- Estimating the scrollbar position (VS Code)
- Computing wrapping incrementally/lazily and caching per-line results
- Using a separate background thread for scrollbar computation

The cheapest correct fix is caching with invalidation on edit/resize.
