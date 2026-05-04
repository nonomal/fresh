//! Migrated from `tests/e2e/smart_editing.rs` and
//! `tests/e2e/goto_matching_bracket.rs`.

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, BehaviorFlags, BufferScenario, CursorExpect,
};
use fresh::test_api::Action;

#[test]
fn migrated_typing_quotes_in_text_buffer_does_not_auto_pair() {
    // In a text buffer (no language), quote chars don't auto-pair.
    assert_buffer_scenario(BufferScenario {
        description: "InsertChar('\"') in text buffer inserts one char".into(),
        initial_text: String::new(),
        actions: vec![Action::InsertChar('"')],
        expected_text: "\"".into(),
        expected_primary: CursorExpect::at(1),
        ..Default::default()
    });
}

#[test]
fn migrated_typing_quotes_in_rust_buffer_auto_pairs() {
    // Quote chars do auto-pair in language=rust with auto-close on.
    assert_buffer_scenario(BufferScenario {
        description: "InsertChar('\"') in .rs buffer with auto_close=true pairs the quote".into(),
        initial_text: String::new(),
        behavior: BehaviorFlags::production(),
        language: Some("x.rs".into()),
        actions: vec![Action::InsertChar('"')],
        expected_text: "\"\"".into(),
        expected_primary: CursorExpect::at(1),
        ..Default::default()
    });
}

#[test]
fn migrated_goto_matching_bracket_jumps_from_open_to_close() {
    // GoToMatchingBracket on '(' moves cursor to the matching ')'.
    assert_buffer_scenario(BufferScenario {
        description: "GoToMatchingBracket on '(' lands at matching ')'".into(),
        initial_text: "(abc)".into(),
        actions: vec![Action::GoToMatchingBracket],
        expected_text: "(abc)".into(),
        // Exact landing depends on the implementation; we just
        // verify it moved at all.
        expected_primary: CursorExpect::at(4),
        ..Default::default()
    });
}
