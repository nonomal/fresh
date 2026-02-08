# Issue #953 Reproduction Attempt: LSP Pop-up Not Appearing for Go Files

## Issue Summary

User reports that after upgrading from Debian 12 to 13, Fresh editor no longer
displays LSP pop-ups or autocomplete for Go files (.go), despite gopls being
correctly installed. CLangd works fine for C files. The same Go files show
proper LSP popups in Kate editor.

A second user on Ubuntu 24.04.3 LTS reports the warning:
"Failed to spawn LSP client for language: text"

## Environment Used for Reproduction

- OS: Linux 4.4.0 (container)
- Go: go1.24.7 linux/amd64
- gopls: v0.17.1
- Fresh: built from source (debug build, current HEAD)
- Terminal: tmux 3.4

## Test Scenarios and Results

### Test 1: Basic Go LSP with manual start (default auto_start=false)

1. Opened `/tmp/go-test-project/main.go` (with go.mod present)
2. Editor detected language as "go" (shown in status bar)
3. LSP did NOT auto-start (expected, since default `auto_start: false`)
4. Used command palette > "Start/Restart LSP Server"
5. Status bar showed "LSP (go) ready" and "LSP [go: ready]"
6. Typed `fmt.` on a new line
7. **Result: Completion popup appeared correctly** with all fmt package symbols

### Test 2: Hover info after manual LSP start

1. Positioned cursor on `Println` in `fmt.Println(...)`
2. Used command palette > "Show Hover Info" (Alt+K)
3. **Result: Hover popup appeared correctly** with full function signature and documentation

### Test 3: Go file outside of a Go module (no go.mod)

1. Created `/tmp/go-no-module/main.go` without go.mod
2. Started LSP manually via command palette
3. LSP initialized successfully
4. Typed `fmt.` to trigger completion
5. **Result: Completion popup appeared correctly**

### Test 4: gopls not in PATH

1. Started Fresh WITHOUT `~/go/bin` in PATH
2. Used command palette > "Start/Restart LSP Server"
3. **Result: Error shown** - "Go LSP server 'gopls' not found" with "LSP [go: error]"
4. This is correct behavior with a clear error message

### Test 5: auto_start=true via config

1. Created `~/.config/fresh/config.json` with `"auto_start": true` for Go
2. Opened a Go file
3. LSP auto-started without manual intervention, status bar showed "LSP (go) ready"
4. Typed `fmt.` to trigger completion
5. **Result: Completion popup appeared correctly** immediately

## Key Observations

### Could NOT Reproduce the Issue

In all test scenarios with gopls available in PATH, the Go LSP worked correctly
including completions, hover info, and diagnostics.

### Potential Root Causes (hypotheses)

1. **auto_start=false (default)**: The default configuration for Go LSP has
   `auto_start: false`. If the user previously had a version of Fresh that
   auto-started gopls, upgrading may have changed this behavior. The user may
   not realize they need to manually start LSP via the command palette or
   LSP menu.

2. **PATH issue after OS upgrade**: After upgrading from Debian 12 to 13,
   the user's `gopls` binary location may have changed or been removed.
   The editor shows a clear error for this case ("not found"), but the user
   may have missed it.

3. **"Failed to spawn LSP client for language: text"** (from second reporter):
   This error at `file_operations.rs:501` indicates the file's language was
   detected as "text" instead of "go". This could happen if:
   - The language detection fails (e.g., file has no `.go` extension)
   - The config's language definitions are corrupted
   - There's a config layer that overrides the default language mappings

4. **gopls version incompatibility**: Different gopls versions may behave
   differently during initialization. A newer gopls version installed during
   the OS upgrade might have compatibility issues.

5. **Workspace/module issues**: gopls may fail silently if the workspace
   configuration is problematic (though Test 3 showed it works without go.mod).

## Code Analysis

### Language Detection Flow
- `detect_language()` in `manager.rs:691-714` checks file extension then filename
- `.go` files should be detected as language "go"
- Fallback to "text" happens in `state.rs:304-312` when both tree-sitter and
  extension-based detection fail

### LSP Spawn Flow
- `try_spawn()` in `manager.rs:212-241` checks: config exists, enabled, has runtime, auto_start
- With `auto_start: false` (default for Go), returns `NotAutoStart` unless manually allowed
- `force_spawn()` in `manager.rs:347-443` bypasses auto_start check

### Default Go LSP Config (config.rs:2893-2904)
```rust
LspServerConfig {
    command: "gopls".to_string(),
    args: vec![],
    enabled: true,
    auto_start: false,  // <-- User must manually start
    process_limits: ProcessLimits::default(),
    initialization_options: None,
}
```

## Recommendations for Further Investigation

1. Ask the reporter to check if they see "LSP [go: ready]" or "LSP [go: error]"
   in the status bar after manually starting LSP
2. Ask the reporter to share their config.json to check for overrides
3. Ask the reporter what version of gopls they have installed
4. Check if the "text" language detection issue from the second reporter is
   related to a specific file opening scenario
5. Add more visible feedback when `auto_start=false` - e.g., a status bar
   indicator suggesting the user start LSP manually
