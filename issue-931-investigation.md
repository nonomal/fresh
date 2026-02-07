# Issue #931 Investigation: LSP Takes Focus

## Status: STILL RELEVANT (confirmed reproducible)

## Issue Summary
When the LSP completion popup appears, it operates in fully modal mode,
capturing all keyboard input. While type-to-filter works (characters are
inserted into the buffer AND filter the list), several behaviors are
problematic:

## Confirmed Problems

### 1. Enter key is captured by the popup (default behavior)
When `accept_suggestion_on_enter` is `On` (the default), pressing Enter
while the completion popup is open **accepts the completion** instead of
inserting a newline. Users who are typing new code and happen to trigger
the popup cannot press Enter to go to the next line without first
dismissing the popup with Escape.

### 2. Popup reappears aggressively after dismissal
After pressing Escape to dismiss the completion popup, typing just 2 more
characters triggers the popup to reappear (via the `quick_suggestions`
debounced trigger). This creates the "unusable" cycle described in the issue:
popup appears -> press Escape -> type 2 chars -> popup reappears.

### 3. All non-character keys are consumed
The completion popup returns `InputResult::Consumed` for ALL unhandled keys
(line 63 and 117 of `completion.rs`). This means keys like:
- Left/Right arrows (for cursor movement within the line)
- Home/End
- Any modifier combinations not explicitly handled
are silently swallowed while the popup is open.

## Code References

### Root cause: Modal popup behavior
- `crates/fresh-editor/src/view/popup_input.rs:26-28`: `is_modal()` returns
  `true` when popup is visible, blocking all input from reaching the buffer.
- `crates/fresh-editor/src/view/popup/input/completion.rs:62-63`: Catch-all
  `_ => InputResult::Consumed` swallows all unhandled keys.

### Type-to-filter does work correctly
- `crates/fresh-editor/src/app/popup_actions.rs:205-223`: `handle_popup_type_char`
  correctly inserts characters into the buffer before re-filtering.

### Aggressive re-triggering
- `crates/fresh-editor/src/app/lsp_requests.rs:444-`: `maybe_trigger_completion`
  triggers completion on word characters after a debounce delay, causing the
  popup to reappear shortly after being dismissed.

## Test Setup
- Tested with clangd 18.1.3 on Ubuntu
- Used fresh 0.1.99 (debug build from current master)
- C++ file with `std::string` member access (`msg.`)
- Completion popup appeared correctly and showed string methods
- Confirmed: typing letters inserts + filters, Enter accepts completion,
  Escape dismisses, popup reappears after 2 chars

## Suggested Fixes
1. Make the popup non-modal: let unhandled keys pass through to the buffer
   (similar to VS Code behavior where the popup is an overlay, not a modal)
2. Add a "cooldown" after Escape dismissal to prevent immediate re-triggering
3. Change default `accept_suggestion_on_enter` to `Smart` or `Off`
4. Pass Left/Right arrow keys through to the buffer for cursor movement

## Relates to
- Issue #931: "LSP takes focus (Java)"
- Comment issuecomment-3861575584 by X-Ryl669: Same issue in C++
