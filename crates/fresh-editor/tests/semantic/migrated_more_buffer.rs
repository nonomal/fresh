//! Smaller migrations from a grab bag of e2e files:
//! `tests/e2e/document_model.rs`,
//! `tests/e2e/triple_click.rs` (text-state portion),
//! `tests/e2e/select_to_paragraph.rs` (uncovered cases).

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, BufferScenario, CursorExpect,
};
use fresh::test_api::Action;

#[test]
fn migrated_select_word_action_picks_word_at_cursor() {
    // Triple-click selects the line; SelectWord/SelectLine actions
    // are the semantic equivalents.
    assert_buffer_scenario(BufferScenario {
        description: "SelectWord at byte 0 of 'foo bar' selects 'foo'".into(),
        initial_text: "foo bar".into(),
        actions: vec![Action::SelectWord],
        expected_text: "foo bar".into(),
        expected_primary: CursorExpect::range(0, 3),
        expected_selection_text: Some("foo".into()),
        ..Default::default()
    });
}

#[test]
fn migrated_select_line_first_line_includes_trailing_newline() {
    assert_buffer_scenario(BufferScenario {
        description: "SelectLine on line 1 of 'a\\nb' selects 'a\\n'".into(),
        initial_text: "a\nb".into(),
        actions: vec![Action::SelectLine],
        expected_text: "a\nb".into(),
        expected_primary: CursorExpect::range(0, 2),
        expected_selection_text: Some("a\n".into()),
        ..Default::default()
    });
}

#[test]
fn migrated_document_model_long_buffer_round_trip() {
    // Document model: a long buffer should preserve all bytes
    // through a no-op action sequence.
    let text = (0..30).map(|i| format!("row {i}\n")).collect::<String>();
    assert_buffer_scenario(BufferScenario {
        description: "no-op leaves a 30-line buffer byte-exact".into(),
        initial_text: text.clone(),
        actions: vec![],
        expected_text: text,
        expected_primary: CursorExpect::at(0),
        ..Default::default()
    });
}

#[test]
fn migrated_move_to_document_end_lands_at_buffer_length() {
    let text = "alpha\nbravo\ncharlie";
    assert_buffer_scenario(BufferScenario {
        description: "MoveDocumentEnd on a 19-byte buffer lands at byte 19".into(),
        initial_text: text.into(),
        actions: vec![Action::MoveDocumentEnd],
        expected_text: text.into(),
        expected_primary: CursorExpect::at(text.len()),
        ..Default::default()
    });
}
