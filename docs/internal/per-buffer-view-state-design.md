# Design: Keyed Per-Buffer Per-View State

## Status
Proposed: February 14, 2026

## Problem

Editor state is currently split between `EditorState` (per buffer) and `SplitViewState` (per split), causing several problems:

- **Shared split state**: View settings like `view_mode`, `compose_width`, and `compose_column_guides` live on `SplitViewState`. Switching buffers within a split carries these settings across — enabling Compose mode on a Markdown file and switching to a Rust file leaves Rust in Compose mode.

- **State duplication**: `view_mode` and compose fields exist in both `EditorState.compose` and `SplitViewState`, with no clear source of truth and manual sync required between them.

- **Cursor ownership split**: `EditorState` owns `cursors`, but `SplitViewState` also stores a copy. `save_current_split_view_state()` and `restore_current_split_view_state()` manually shuttle cursors between them during buffer switches. Missing a sync call loses cursor state.

- **Runtime/persistence mismatch**: `workspace.rs` already has a keyed `file_states: HashMap<PathBuf, SerializedFileState>` for cursor/scroll persistence, but the runtime `SplitViewState` has no equivalent map — just a single active state.

- **No plugin state persistence**: Plugins like `markdown_compose` track state in global JS-side structures (`composeBuffers` Set, `tableColumnWidths` Map) that don't persist across sessions and aren't per-split.

## Architecture

### `BufferViewState`

A new struct encapsulating all state specific to a **particular buffer viewed in a particular split**:

```rust
pub struct BufferViewState {
    pub cursors: Cursors,
    pub viewport: Viewport,
    pub view_mode: ViewMode,
    pub compose_width: Option<u16>,
    pub compose_column_guides: Option<Vec<u16>>,
    pub view_transform: Option<ViewTransformPayload>,

    /// Opaque plugin-defined state, keyed by plugin/feature name.
    pub plugin_state: HashMap<String, serde_json::Value>,
}
```

### Updated `SplitViewState`

`SplitViewState` no longer holds buffer-specific fields directly. It stores an `active_buffer` ID and a map of `BufferViewState` entries. There is no separate "active state" cache — the active state is always accessed through the map, eliminating the snapshot/restore desync bugs that plague the current design.

```rust
pub struct SplitViewState {
    pub active_buffer: BufferId,
    pub keyed_states: HashMap<BufferId, BufferViewState>,

    // Split-global state (not buffer-specific)
    pub open_buffers: Vec<BufferId>,
    pub tab_scroll_offset: usize,
    pub focus_history: Vec<BufferId>,
    pub sync_group: Option<u32>,
    pub composite_view: Option<BufferId>,
    pub layout: Option<Layout>,
    pub layout_dirty: bool,
}

impl SplitViewState {
    pub fn active_state(&self) -> &BufferViewState {
        &self.keyed_states[&self.active_buffer]
    }

    pub fn active_state_mut(&mut self) -> &mut BufferViewState {
        self.keyed_states.get_mut(&self.active_buffer).unwrap()
    }
}
```

### Cursor ownership

`EditorState` should no longer own cursors. The authoritative cursor state lives in `BufferViewState`. Editing operations receive cursors by reference from the active `BufferViewState` rather than reading `EditorState.cursors`. This eliminates the cursor sync calls entirely.

The migration path: initially, `EditorState.cursors` becomes a derived reference that borrows from `BufferViewState`. Once all editing operations are updated to accept cursors as a parameter, the field is removed.

## Buffer Switching

When the active buffer in a split changes from A to B:

1. **No snapshot needed** — A's state remains in `keyed_states[A]`.
2. **Load or init** — If `keyed_states` has an entry for B, it's already there. If not, insert a new `BufferViewState` with defaults (see "Default derivation" below).
3. **Set `active_buffer = B`**.
4. **Invalidate layout** — mark `layout_dirty = true`.

The manual `save_current_split_view_state()` / `restore_current_split_view_state()` calls go away.

### Default derivation

When a buffer is opened in a split for the first time and has no keyed state, defaults are resolved in priority order:

1. **Plugin hook** — `buffer_view_init` hook lets plugins set initial state (e.g., markdown_compose sets `view_mode: Compose` for `.md` files)
2. **Filetype config** — per-language settings from editor configuration
3. **Global config** — editor-wide defaults
4. **Hardcoded** — `ViewMode::Source`, no compose width, empty plugin state

## State Lifetime

Keyed state entries are removed when a buffer is removed from `open_buffers` (tab closed). To support "reopen closed tab" with full state restoration, recently closed entries can be kept in an LRU eviction list (bounded, e.g., 20 entries). Beyond that, they're dropped.

## Workspace Persistence

`SerializedSplitViewState` in `workspace.rs` stores the full keyed map, replacing the current `file_states` cursor-only map:

```json
{
  "split_id": 1,
  "active_tab_index": 0,
  "keyed_states": {
    "src/main.rs": {
      "cursor": { "position": 100 },
      "scroll": { "top": 0 },
      "view_mode": "Source",
      "plugin_state": {}
    },
    "README.md": {
      "cursor": { "position": 0 },
      "scroll": { "top": 0 },
      "view_mode": "Compose",
      "compose_width": 80,
      "plugin_state": {
        "markdown-compose": { "table_widths": {} }
      }
    }
  }
}
```

### Session restore ordering

On workspace restore, buffer content and plugin initialization must complete before view state is applied. The sequence:

1. Deserialize `keyed_states` from workspace file
2. Load buffer content and initialize plugins
3. Apply deserialized `BufferViewState` entries
4. Fire `buffer_view_restored` hook so plugins can react to restored state (e.g., markdown_compose re-enables compose rendering)

## Plugin API

### Methods

- `editor.setViewState(bufferId, key, value)` — stores a JSON-serializable value in the `plugin_state` of the current split's view of the buffer.
- `editor.getViewState(bufferId, key)` — retrieves the value, or `undefined` if not present.

### Robustness contract

Plugin state is stored as opaque JSON (`serde_json::Value`). Plugins must treat values returned by `getViewState` as `unknown` and validate before use. State may be missing, malformed, or from an older version of the plugin. Plugins should fall back to sensible defaults when state cannot be parsed. If a plugin's state schema changes, it is the plugin's responsibility to handle or discard unrecognized shapes.

### Benefits for markdown_compose

The `markdown_compose` plugin currently uses global JS-side structures. With keyed state:

- `view_mode` is stored per-buffer-per-split — switching from a Compose-mode Markdown file to Rust leaves Rust in Source mode.
- `tableColumnWidths` stored in plugin state means two splits viewing the same Markdown file at different terminal widths compute table alignment independently.
- Workspace restore recovers `view_mode` from keyed state, so compose mode reactivates automatically without the plugin needing to re-detect file type.

### Content-dependent plugin state

Plugin state like `tableColumnWidths` depends on buffer content. When a buffer is edited from one split, other splits viewing the same buffer should invalidate content-dependent plugin state. Plugins can listen for `buffer_changed` events and mark their cached state stale.

## Implementation Plan

**Refactor `SplitViewState`**

- Create `BufferViewState` struct.
- Replace buffer-specific fields on `SplitViewState` with `active_buffer: BufferId` and `keyed_states: HashMap<BufferId, BufferViewState>`.
- Add `active_state()` / `active_state_mut()` accessors.
- Update all reads of `SplitViewState.view_mode`, `.compose_width`, `.cursors`, `.viewport`, etc. to go through `active_state()`.
- Update `set_active_buffer` to use the new buffer switching flow.
- Remove `save_current_split_view_state()` / `restore_current_split_view_state()`.

**Cursor migration**

- Update editing operations to access cursors from `BufferViewState` rather than `EditorState`.
- Remove `EditorState.cursors` and all cursor sync calls.

**Remove `ComposeState` from `EditorState`**

- Remove `EditorState.compose` and redirect all reads to `BufferViewState` fields.
- Remove `compose_prev_line_numbers` and other transitional fields.

**Persistence**

- Update `workspace.rs` serialization to store the full keyed map, replacing the `file_states` cursor-only map.

**Plugin API**

- Expose `setViewState` / `getViewState` to the JS plugin runtime.
- Migrate `markdown_compose` to use the plugin state API instead of global JS-side structures.

## Open Questions

- **Undo interaction**: Toggling view mode is currently not undoable (undo only covers buffer content). Should it be? Adding view state to the undo stack is complex; not adding it means accidental mode switches can't be undone. Recommendation: keep view mode changes out of undo for now, revisit if users report it as a problem.

- **Same buffer, multiple splits**: Content is shared (one `Buffer`), but view state is per-split. Cursor and scroll independence is straightforward. Content-dependent state (view transforms, column guides) needs an invalidation mechanism when the buffer is edited from another split. The `buffer_changed` hook approach above may be sufficient, but needs testing.
