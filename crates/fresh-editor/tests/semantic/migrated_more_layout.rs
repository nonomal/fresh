//! More migrated LayoutScenarios from
//! `tests/e2e/line_wrap_full_visibility.rs`,
//! `tests/e2e/line_wrap_parity.rs`,
//! `tests/e2e/horizontal_scrollbar.rs`,
//! `tests/e2e/virtual_lines.rs`.

use crate::common::scenario::layout_scenario::{assert_layout_scenario, LayoutScenario};
use crate::common::scenario::render_snapshot::RenderSnapshotExpect;
use fresh::test_api::Action;

#[test]
fn migrated_short_buffer_in_wide_terminal_top_byte_zero() {
    assert_layout_scenario(LayoutScenario {
        description: "short buffer in wide terminal: top_byte == 0".into(),
        initial_text: "small".into(),
        width: 200,
        height: 24,
        actions: vec![],
        expected_top_byte: Some(0),
        expected_snapshot: RenderSnapshotExpect::default(),
    });
}

#[test]
fn migrated_long_line_in_narrow_terminal_top_byte_zero() {
    assert_layout_scenario(LayoutScenario {
        description: "long line in narrow terminal: top_byte == 0".into(),
        initial_text: "x".repeat(500),
        width: 20,
        height: 8,
        actions: vec![],
        expected_top_byte: Some(0),
        expected_snapshot: RenderSnapshotExpect::default(),
    });
}

#[test]
fn migrated_move_document_start_after_end_resets_top_byte() {
    let text: String = (0..40).map(|i| format!("L{i}\n")).collect();
    assert_layout_scenario(LayoutScenario {
        description: "MoveDocumentEnd then MoveDocumentStart resets top_byte to 0".into(),
        initial_text: text,
        width: 30,
        height: 8,
        actions: vec![Action::MoveDocumentEnd, Action::MoveDocumentStart],
        expected_top_byte: Some(0),
        expected_snapshot: RenderSnapshotExpect::default(),
    });
}

#[test]
fn migrated_one_line_per_row_keeps_top_byte_at_zero() {
    // Buffer fits exactly within visible rows, so no scroll.
    let text = "a\nb\nc\nd";
    assert_layout_scenario(LayoutScenario {
        description: "buffer fitting in viewport: top_byte == 0".into(),
        initial_text: text.into(),
        width: 80,
        height: 24,
        actions: vec![],
        expected_top_byte: Some(0),
        expected_snapshot: RenderSnapshotExpect::default(),
    });
}
