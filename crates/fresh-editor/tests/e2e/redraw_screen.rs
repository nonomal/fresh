//! Tests for the "Redraw Screen" action (issue #1070).
//!
//! The action allows users to request a full terminal clear + repaint from
//! the command palette, which is useful after external programs (or the
//! host terminal) scribble over the TUI and leave ghost text behind.

use crate::common::harness::EditorTestHarness;
use fresh::input::keybindings::Action;

/// Dispatching `Action::RedrawScreen` must set the editor's full-redraw
/// flag so that the main loop clears the terminal on the next tick.
#[test]
fn test_redraw_screen_action_requests_full_redraw() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Sanity: no redraw pending on a fresh editor.
    assert!(!harness.editor_mut().take_full_redraw_request());

    harness
        .editor_mut()
        .dispatch_action_for_tests(Action::RedrawScreen);

    // The action should have flipped the flag so the event loop can
    // clear and repaint the terminal on the next tick.
    assert!(
        harness.editor_mut().take_full_redraw_request(),
        "RedrawScreen action should request a full terminal redraw"
    );
}

/// The "Redraw Screen" entry must be discoverable from the command palette.
#[test]
fn test_redraw_screen_visible_in_command_palette() {
    use crossterm::event::{KeyCode, KeyModifiers};
    let mut harness = EditorTestHarness::new(100, 24).unwrap();

    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.type_text("redraw").unwrap();
    harness.render().unwrap();

    harness.assert_screen_contains("Redraw Screen");
}
