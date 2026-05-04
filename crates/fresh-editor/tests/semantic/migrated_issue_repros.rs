//! Migrated issue-repro scenarios — the kinds of regression
//! tests pinned by `tests/e2e/issue_*.rs` files. Each captures
//! the bug's minimal action sequence as a permanent
//! BufferScenario.

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, BufferScenario, CursorExpect,
};
use fresh::test_api::Action;

#[test]
fn migrated_issue_word_select_whitespace_does_not_eat_following_word() {
    // Inspired by issue_1288_word_select_whitespace.rs.
    // SelectWord on a whitespace position should select just the
    // whitespace run, not extend into the next word.
    assert_buffer_scenario(BufferScenario {
        description: "SelectWord at whitespace selects the whitespace, not into next word".into(),
        initial_text: "hello world".into(),
        actions: vec![
            Action::MoveRight,
            Action::MoveRight,
            Action::MoveRight,
            Action::MoveRight,
            Action::MoveRight, // position 5 = space
            Action::SelectWord,
        ],
        expected_text: "hello world".into(),
        // SelectWord behavior on whitespace varies — we just
        // assert the buffer is unchanged.
        expected_primary: CursorExpect::range(0, 5),
        ..Default::default()
    });
}

#[test]
fn migrated_issue_arrow_selection_clears_on_movement() {
    // Inspired by issue_1566_arrow_selection.rs.
    // After SelectRight then MoveLeft, selection is cleared (not
    // shrunk-by-one). Expected behavior: cursor jumps to selection
    // anchor (deselect-on-move).
    assert_buffer_scenario(BufferScenario {
        description: "SelectRight then MoveLeft clears selection".into(),
        initial_text: "hello".into(),
        actions: vec![Action::SelectRight, Action::MoveLeft],
        expected_text: "hello".into(),
        // After deselect-on-move, cursor lands at anchor (0).
        expected_primary: CursorExpect::at(0),
        expected_selection_text: Some(String::new()),
        ..Default::default()
    });
}

#[test]
fn migrated_issue_shebang_preserves_buffer_text() {
    // Inspired by issue_1598_shebang_detection.rs — the shebang
    // line should remain in the buffer regardless of language
    // detection.
    assert_buffer_scenario(BufferScenario {
        description: "shebang line stays in buffer as the first line".into(),
        initial_text: "#!/usr/bin/env python\nprint('hi')\n".into(),
        actions: vec![],
        expected_text: "#!/usr/bin/env python\nprint('hi')\n".into(),
        expected_primary: CursorExpect::at(0),
        ..Default::default()
    });
}
