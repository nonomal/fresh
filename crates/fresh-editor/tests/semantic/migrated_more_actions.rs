//! Additional action scenarios — claims from
//! `tests/e2e/dabbrev_completion.rs` (text-state portion),
//! `tests/e2e/recovery.rs`, `tests/e2e/triple_click.rs`,
//! `tests/e2e/folding.rs` (text-state portion),
//! `tests/e2e/inline_diagnostics.rs` (text-state portion).
//!
//! All collapse to BufferScenario or TraceScenario claims.

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, repeat, BufferScenario, CursorExpect,
};
use crate::common::scenario::trace_scenario::{assert_trace_scenario, TraceScenario};
use fresh::test_api::Action;

#[test]
fn migrated_select_line_then_delete_removes_full_line_with_newline() {
    assert_buffer_scenario(BufferScenario {
        description: "SelectLine + DeleteBackward removes line + trailing newline".into(),
        initial_text: "alpha\nbravo\n".into(),
        actions: vec![Action::SelectLine, Action::DeleteBackward],
        expected_text: "bravo\n".into(),
        expected_primary: CursorExpect::at(0),
        expected_selection_text: Some(String::new()),
        ..Default::default()
    });
}

#[test]
fn migrated_repeated_typing_undo_chain_is_atomic() {
    // Type 5 chars then Undo 5 times → original.
    assert_trace_scenario(TraceScenario {
        description: "5 inserts, 5 undos = identity".into(),
        initial_text: "x".into(),
        actions: vec![
            Action::MoveDocumentEnd,
            Action::InsertChar('a'),
            Action::InsertChar('b'),
            Action::InsertChar('c'),
            Action::InsertChar('d'),
            Action::InsertChar('e'),
        ],
        expected_text: "xabcde".into(),
        undo_count: 5,
    });
}

#[test]
fn migrated_select_then_delete_atomic_undo_unit() {
    // SelectAll + DeleteBackward is one undo unit; one Undo restores.
    assert_trace_scenario(TraceScenario {
        description: "SelectAll + Delete is one undo unit".into(),
        initial_text: "stable".into(),
        actions: vec![Action::SelectAll, Action::DeleteBackward],
        expected_text: String::new(),
        undo_count: 1,
    });
}

#[test]
fn migrated_select_right_anchors_at_origin() {
    // SelectRight repeatedly grows the selection from byte 0.
    let actions: Vec<Action> = repeat(Action::SelectRight, 3).collect();
    assert_buffer_scenario(BufferScenario {
        description: "3 SelectRight from 0 yields range 0..3".into(),
        initial_text: "abcdef".into(),
        actions,
        expected_text: "abcdef".into(),
        expected_primary: CursorExpect::range(0, 3),
        expected_selection_text: Some("abc".into()),
        ..Default::default()
    });
}

#[test]
fn migrated_typing_after_select_replaces_selection() {
    assert_buffer_scenario(BufferScenario {
        description: "InsertChar with active selection replaces it".into(),
        initial_text: "abcdef".into(),
        actions: vec![
            Action::SelectRight,
            Action::SelectRight,
            Action::SelectRight,
            Action::InsertChar('X'),
        ],
        expected_text: "Xdef".into(),
        expected_primary: CursorExpect::at(1),
        expected_selection_text: Some(String::new()),
        ..Default::default()
    });
}

#[test]
fn migrated_move_word_left_lands_at_word_boundary() {
    assert_buffer_scenario(BufferScenario {
        description: "MoveWordLeft from end of 'foo bar baz' lands at 'baz'".into(),
        initial_text: "foo bar baz".into(),
        actions: vec![Action::MoveDocumentEnd, Action::MoveWordLeft],
        expected_text: "foo bar baz".into(),
        // Land at 'b' of 'baz' = byte 8.
        expected_primary: CursorExpect::at(8),
        ..Default::default()
    });
}

#[test]
fn migrated_move_word_right_lands_at_word_boundary() {
    assert_buffer_scenario(BufferScenario {
        description: "MoveWordRight from byte 0 of 'foo bar' lands at 'bar' word start".into(),
        initial_text: "foo bar".into(),
        actions: vec![Action::MoveWordRight],
        expected_text: "foo bar".into(),
        expected_primary: CursorExpect::at(4),
        ..Default::default()
    });
}
