//! Migrated from `tests/e2e/macros.rs` and parts of
//! `tests/e2e/vi_mode.rs` that reduce to plain action sequences.

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, BufferScenario, CursorExpect,
};
use crate::common::scenario::trace_scenario::{assert_trace_scenario, TraceScenario};
use fresh::test_api::Action;

#[test]
fn migrated_macro_replay_is_action_sequence() {
    // The whole point of macro support is "replay these actions as
    // a unit." A scenario captures that as data directly — no
    // record/play distinction.
    assert_buffer_scenario(BufferScenario {
        description: "5-action 'macro' applied to empty buffer produces 'abcde'".into(),
        initial_text: String::new(),
        actions: vec![
            Action::InsertChar('a'),
            Action::InsertChar('b'),
            Action::InsertChar('c'),
            Action::InsertChar('d'),
            Action::InsertChar('e'),
        ],
        expected_text: "abcde".into(),
        expected_primary: CursorExpect::at(5),
        ..Default::default()
    });
}

#[test]
fn migrated_repeated_action_sequence_is_idempotent_for_movement() {
    // Repeating a movement-only sequence shouldn't change buffer.
    assert_buffer_scenario(BufferScenario {
        description: "MoveDocumentEnd × 3 leaves cursor at byte 5".into(),
        initial_text: "hello".into(),
        actions: vec![
            Action::MoveDocumentEnd,
            Action::MoveDocumentEnd,
            Action::MoveDocumentEnd,
        ],
        expected_text: "hello".into(),
        expected_primary: CursorExpect::at(5),
        ..Default::default()
    });
}

#[test]
fn migrated_macro_undo_atomicity() {
    // 5-action macro is 5 separate undo units (one per insertion).
    assert_trace_scenario(TraceScenario {
        description: "5-char macro = 5 undo units".into(),
        initial_text: "X".into(),
        actions: vec![
            Action::MoveDocumentEnd,
            Action::InsertChar('a'),
            Action::InsertChar('b'),
            Action::InsertChar('c'),
            Action::InsertChar('d'),
            Action::InsertChar('e'),
        ],
        expected_text: "Xabcde".into(),
        undo_count: 5,
    });
}

#[test]
fn migrated_macro_with_selection_replace() {
    // Macro that replaces a selection in one step.
    assert_buffer_scenario(BufferScenario {
        description: "selection-replace macro: SelectAll + InsertChar".into(),
        initial_text: "old".into(),
        actions: vec![Action::SelectAll, Action::InsertChar('!')],
        expected_text: "!".into(),
        expected_primary: CursorExpect::at(1),
        expected_selection_text: Some(String::new()),
        ..Default::default()
    });
}
