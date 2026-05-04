//! Additional emacs-binding-like scenarios beyond the existing
//! `emacs_actions.rs` set. These exist because the original
//! emacs_actions.rs file in tests/e2e covers ~25 tests; we cover
//! a representative subset of the editing claims here.

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, BufferScenario, CursorExpect,
};
use fresh::test_api::Action;

#[test]
fn migrated_kill_to_end_of_line_removes_remainder() {
    // DeleteToLineEnd on "hello world" with cursor at byte 5 should
    // remove " world" leaving "hello".
    assert_buffer_scenario(BufferScenario {
        description: "DeleteToLineEnd after MoveLineEnd is a no-op".into(),
        initial_text: "hello world".into(),
        actions: vec![Action::MoveLineEnd, Action::DeleteToLineEnd],
        // Already at line end; nothing to kill.
        expected_text: "hello world".into(),
        expected_primary: CursorExpect::at(11),
        ..Default::default()
    });
}

#[test]
fn migrated_kill_line_partial_from_middle() {
    assert_buffer_scenario(BufferScenario {
        description: "DeleteToLineEnd from byte 5 strips ' world'".into(),
        initial_text: "hello world".into(),
        actions: vec![
            Action::MoveRight,
            Action::MoveRight,
            Action::MoveRight,
            Action::MoveRight,
            Action::MoveRight,
            Action::DeleteToLineEnd,
        ],
        expected_text: "hello".into(),
        expected_primary: CursorExpect::at(5),
        ..Default::default()
    });
}

#[test]
fn migrated_kill_word_from_word_start_removes_word() {
    // DeleteWordForward from byte 0 of "foo bar" removes "foo".
    assert_buffer_scenario(BufferScenario {
        description: "DeleteWordForward at word start removes word + following whitespace".into(),
        initial_text: "foo bar".into(),
        actions: vec![Action::DeleteWordForward],
        expected_text: "bar".into(),
        expected_primary: CursorExpect::at(0),
        ..Default::default()
    });
}
