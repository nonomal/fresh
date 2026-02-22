//! Test that moving the cursor up with the Up key does not scroll the viewport
//! when the cursor is still well within the visible area.
//!
//! Scenario: a file with many long lines (line-wrap enabled) is opened.
//! Ctrl+End jumps to the end of the document, placing the cursor at the
//! bottom of the viewport.  Pressing Up moves the cursor one visual row
//! higher — but since the cursor is still near the bottom of the screen,
//! the viewport should NOT scroll.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use fresh::config::Config;

fn config_with_line_wrap() -> Config {
    let mut config = Config::default();
    config.editor.line_wrap = true;
    config
}

/// Create content with many long lines that will each soft-wrap several times.
/// Each line is distinguishable by its prefix ("LINE_001: …").
fn make_many_long_lines(count: usize, chars_per_line: usize) -> String {
    (1..=count)
        .map(|i| {
            let prefix = format!("LINE_{:03}: ", i);
            let filler_len = chars_per_line.saturating_sub(prefix.len());
            format!("{}{}", prefix, "x".repeat(filler_len))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// After Ctrl+End the cursor sits at the bottom of the viewport on the last
/// line.  Pressing Up should move the cursor one visual row higher WITHOUT
/// scrolling the viewport, because the cursor is still visible.
#[test]
fn test_cursor_up_after_ctrl_end_does_not_scroll() {
    const WIDTH: u16 = 80;
    const HEIGHT: u16 = 24;

    let mut harness =
        EditorTestHarness::with_config(WIDTH, HEIGHT, config_with_line_wrap()).unwrap();

    // 50 lines, each ~300 chars → each wraps to ~4-5 visual rows at 80 cols.
    // Total visual rows ≫ viewport height, so Ctrl+End will scroll.
    let content = make_many_long_lines(50, 300);
    let _fixture = harness.load_buffer_from_text(&content).unwrap();
    harness.render().unwrap();

    // Jump to the very end of the document
    harness
        .send_key(KeyCode::End, KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Record the viewport position and cursor row BEFORE pressing Up
    let top_byte_before = harness.top_byte();
    let top_vline_before = harness.top_view_line_offset();
    let (_cx_before, cy_before) = harness.screen_cursor_position();

    eprintln!(
        "Before Up: top_byte={}, top_vline_offset={}, cursor_row={}",
        top_byte_before, top_vline_before, cy_before
    );

    // Press Up once
    harness
        .send_key(KeyCode::Up, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let top_byte_after = harness.top_byte();
    let top_vline_after = harness.top_view_line_offset();
    let (_cx_after, cy_after) = harness.screen_cursor_position();

    eprintln!(
        "After Up:  top_byte={}, top_vline_offset={}, cursor_row={}",
        top_byte_after, top_vline_after, cy_after
    );

    // The cursor should have moved up on screen
    assert!(
        cy_after < cy_before,
        "Cursor should move up on screen: before row {}, after row {}",
        cy_before,
        cy_after
    );

    // The viewport should NOT have scrolled — the top position must be unchanged
    assert_eq!(
        top_byte_before, top_byte_after,
        "Viewport top_byte should not change when cursor is still visible.\n\
         Before: top_byte={}, After: top_byte={}",
        top_byte_before, top_byte_after
    );
    assert_eq!(
        top_vline_before, top_vline_after,
        "Viewport top_view_line_offset should not change when cursor is still visible.\n\
         Before: {}, After: {}",
        top_vline_before, top_vline_after
    );
}

/// Same scenario but press Up several times — as long as the cursor remains
/// within the content area the viewport must stay put.
#[test]
fn test_multiple_cursor_up_after_ctrl_end_no_scroll_until_near_top() {
    const WIDTH: u16 = 80;
    const HEIGHT: u16 = 24;

    let mut harness =
        EditorTestHarness::with_config(WIDTH, HEIGHT, config_with_line_wrap()).unwrap();

    let content = make_many_long_lines(50, 300);
    let _fixture = harness.load_buffer_from_text(&content).unwrap();
    harness.render().unwrap();

    // Jump to end
    harness
        .send_key(KeyCode::End, KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    let top_byte_initial = harness.top_byte();
    let top_vline_initial = harness.top_view_line_offset();
    let (_, cy_initial) = harness.screen_cursor_position();
    let (content_first_row, _) = harness.content_area_rows();

    eprintln!(
        "Initial: top_byte={}, vline_offset={}, cursor_row={}, content_first_row={}",
        top_byte_initial, top_vline_initial, cy_initial, content_first_row
    );

    // Calculate how many Up presses we can do while staying well inside the
    // viewport (at least 3 rows from the top of the content area).
    let safe_ups = (cy_initial as usize).saturating_sub(content_first_row + 3);
    let ups_to_test = safe_ups.min(5); // test up to 5

    assert!(
        ups_to_test >= 2,
        "Need at least 2 safe Up presses to test, but only got {}. \
         cursor_row={}, content_first_row={}",
        ups_to_test,
        cy_initial,
        content_first_row
    );

    for i in 1..=ups_to_test {
        harness
            .send_key(KeyCode::Up, KeyModifiers::NONE)
            .unwrap();
        harness.render().unwrap();

        let top_byte_now = harness.top_byte();
        let top_vline_now = harness.top_view_line_offset();
        let (_, cy_now) = harness.screen_cursor_position();

        eprintln!(
            "After Up #{}: top_byte={}, vline_offset={}, cursor_row={}",
            i, top_byte_now, top_vline_now, cy_now
        );

        assert_eq!(
            top_byte_initial, top_byte_now,
            "Viewport scrolled unexpectedly on Up press #{}: \
             top_byte went from {} to {}",
            i, top_byte_initial, top_byte_now
        );
        assert_eq!(
            top_vline_initial, top_vline_now,
            "Viewport vline offset changed unexpectedly on Up press #{}: \
             went from {} to {}",
            i, top_vline_initial, top_vline_now
        );
    }
}
