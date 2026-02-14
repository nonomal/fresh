# Design: Keyed Per-Buffer Per-View State

## Status
Proposed: February 14, 2026
Author: Gemini CLI Agent

## 1. Problem Statement

Currently, the editor's state is split between `EditorState` (per buffer) and `SplitViewState` (per split). This leads to several issues:

1.  **Shared Split State**: Settings like `view_mode` (Source vs. Compose), `compose_width`, and `line_wrap` are stored directly on the `SplitViewState`. When a user switches buffers within a split, these settings persist across buffers. For example, enabling "Compose" mode for a Markdown file and then switching to a Rust file in the same split leaves the Rust file in "Compose" mode.
2.  **State Duplication**: Some fields (like `view_mode`) exist in both `EditorState.compose` and `SplitViewState`, leading to synchronization complexity and ambiguity about the "source of truth."
3.  **Plugin State Persistence**: Plugins (like `markdown_compose`) often need to track which buffers are in a specific mode or store metadata per buffer. There is currently no unified way for plugins to attach custom, serializable state to a buffer's "view" that persists in the workspace.
4.  **Cursor/Scroll Sync**: While `workspace.rs` already implements a form of keyed state (`file_states`) for cursors and scroll positions, this is not reflected in the runtime `SplitViewState` objects, requiring manual sync calls (`save_current_split_view_state`) during buffer/split switches.

## 2. Proposed Architecture: Keyed State Model

Inspired by VS Code's distributed, URI-keyed data model, we will move towards a **Keyed View State** architecture.

### 2.1 The `BufferViewState` Struct
We introduce a new struct to encapsulate all state that is specific to a **particular buffer when viewed in a particular split**.

```rust
pub struct BufferViewState {
    /// Independent cursor set for this view
    pub cursors: Cursors,
    /// Independent scroll position (viewport)
    pub viewport: Viewport,
    /// View mode (Source/Compose)
    pub view_mode: ViewMode,
    /// Compose specific settings
    pub compose_width: Option<u16>,
    pub compose_column_guides: Option<Vec<u16>>,
    pub view_transform: Option<ViewTransformPayload>,
    
    /// Generic map for plugin-defined state.
    /// Keyed by plugin name or feature ID.
    pub plugin_state: HashMap<String, serde_json::Value>,
}
```

### 2.2 Updating `SplitViewState`
`SplitViewState` will no longer hold buffer-specific fields directly. Instead, it will manage a map of `BufferViewState` objects.

```rust
pub struct SplitViewState {
    /// The "active" state for the currently displayed buffer.
    /// This acts as a 'hot' cache for performance during rendering/editing.
    pub active_state: BufferViewState,

    /// Map of states for all buffers that have been opened in this split.
    /// Allows switching back to a buffer and finding it exactly as it was left.
    pub keyed_states: HashMap<BufferId, BufferViewState>,

    /// Split-global state (not buffer-specific)
    pub open_buffers: Vec<BufferId>,
    pub tab_scroll_offset: usize,
    pub sync_group: Option<u32>,
    // ...
}
```

## 3. Workflow Transitions

### 3.1 Switching Buffers (`set_active_buffer`)
When the active buffer in a split changes from `A` to `B`:
1.  **Snapshot**: Copy `active_state` into `keyed_states.insert(A, ...)`.
2.  **Load/Init**: 
    - If `keyed_states.get(&B)` exists, copy it into `active_state`.
    - If not, initialize `active_state` with defaults (or derived from global config/buffer type).
3.  **Invalidate**: Mark layout as dirty to trigger re-render in the new mode.

### 3.2 Workspace Persistence
The `SerializedSplitViewState` in `workspace.rs` will be updated to reflect this map. Instead of a flat `file_states` map for just cursors/scroll, it will store the full `SerializedBufferViewState`.

```json
{
  "split_id": 1,
  "keyed_states": {
    "src/main.rs": {
      "cursor": { "position": 100 },
      "view_mode": "Source",
      "plugin_state": {}
    },
    "README.md": {
      "cursor": { "position": 0 },
      "view_mode": "Compose",
      "compose_width": 80,
      "plugin_state": {
        "markdown-compose": { "table_widths": { ... } }
      }
    }
  }
}
```

## 4. Plugin API Integration

Plugins will gain the ability to store and retrieve serializable data within this keyed state.

### 4.1 New API Methods
- `editor.setViewState(bufferId, key, value)`: Stores a JSON-serializable value in the `plugin_state` of the current split's view of the buffer.
- `editor.getViewState(bufferId, key)`: Retrieves the value.

### 4.2 Benefits for Markdown Compose
The `markdown_compose.ts` plugin currently uses a global `composeBuffers` Set. In the new design:
1.  It can call `editor.setViewMode(id, "compose")` which updates the keyed state.
2.  It can store its `tableColumnWidths` directly in the view state, ensuring that if the user has two splits viewing the same Markdown file at different widths, the table alignment is computed correctly for each.
3.  When a workspace is restored, the `view_mode` is recovered from the keyed state, allowing the plugin to re-activate "Compose" mode automatically based on the restored state.

## 5. Implementation Plan

1.  **Refactor `SplitViewState`**: Create `BufferViewState` and move cursors, viewport, and compose fields into it.
2.  **Update `Editor` Methods**: Update `save_current_split_view_state` and `restore_current_split_view_state` to handle the map swapping.
3.  **Serialization**: Update `workspace.rs` structs and logic to persist the new map.
4.  **Plugin API**: Expose `plugin_state` access to the JavaScript runtime.
5.  **Clean up `EditorState`**: Remove the redundant `ComposeState` from `EditorState` once the view-authoritative model is stable.
