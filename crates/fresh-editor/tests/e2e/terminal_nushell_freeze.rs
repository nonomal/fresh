//! Regression test for issue #884: nushell freezing on terminal entry
//!
//! When nushell starts inside a PTY, its line editor (reedline) sends a kitty
//! keyboard protocol query (`\x1b[?u`) and blocks until it gets a response.
//!
//! The fix (setting `kitty_keyboard: true` in alacritty_terminal's TermConfig)
//! ensures this query gets a response via the existing PtyWrite pipeline.
//!
//! This test uses a fake shell (`tests/fixtures/fake_nushell.py`) that mimics
//! nushell's probing behavior: it sends `\x1b[?u` and only becomes interactive
//! once it receives a response. Without the fix, the fake shell stays stuck.

use crate::common::harness::EditorTestHarness;
use portable_pty::{native_pty_system, PtySize};
use std::path::PathBuf;

fn harness_or_skip(width: u16, height: u16) -> Option<EditorTestHarness> {
    if native_pty_system()
        .openpty(PtySize {
            rows: 1,
            cols: 1,
            pixel_width: 0,
            pixel_height: 0,
        })
        .is_err()
    {
        eprintln!("Skipping terminal test: PTY not available in this environment");
        return None;
    }

    EditorTestHarness::new(width, height).ok()
}

macro_rules! harness_or_return {
    ($w:expr, $h:expr) => {
        match harness_or_skip($w, $h) {
            Some(h) => h,
            None => return,
        }
    };
}

/// Return the absolute path to the fake_nushell.py fixture.
fn fake_nushell_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("tests/fixtures/fake_nushell.py")
}

/// Regression test for #884: shells that probe terminal capabilities must get
/// responses from fresh's PTY, otherwise they freeze.
///
/// Uses a fake shell that sends `\x1b[?u` (kitty keyboard protocol query) on
/// startup and only prints `FAKE_SHELL_READY` once a response arrives. Without
/// the fix (`kitty_keyboard: true`), the query goes unanswered and the fake
/// shell prints `FAKE_SHELL_STUCK_NO_RESPONSE` instead.
#[test]
#[cfg_attr(target_os = "windows", ignore)] // Uses python3 / Unix PTY
fn test_nushell_terminal_capability_queries_get_responses() {
    let mut harness = harness_or_return!(100, 30);

    let fake_shell = fake_nushell_path();
    assert!(
        fake_shell.exists(),
        "fake_nushell.py fixture not found at {:?}",
        fake_shell
    );

    // Point SHELL at the fake nushell so open_terminal() spawns it.
    // The fixture has a #!/usr/bin/env python3 shebang so it's directly executable.
    std::env::set_var("SHELL", fake_shell.to_str().unwrap());

    harness.editor_mut().open_terminal();
    harness.render().unwrap();

    // The fake shell sends \x1b[?u and waits for the kitty keyboard response.
    // With the fix, alacritty_terminal responds with \x1b[?0u and the fake
    // shell prints FAKE_SHELL_READY. Without the fix, it times out (5s) and
    // prints FAKE_SHELL_STUCK_NO_RESPONSE.
    let got_output = harness
        .wait_for_async(
            |h| {
                let screen = h.screen_to_string();
                screen.contains("FAKE_SHELL_READY")
                    || screen.contains("FAKE_SHELL_STUCK_NO_RESPONSE")
            },
            15_000,
        )
        .expect("wait_for_async should not error");

    let screen = harness.screen_to_string();

    assert!(
        got_output,
        "Fake shell produced no output at all within 15s. Screen:\n{}",
        screen
    );

    assert!(
        screen.contains("FAKE_SHELL_READY"),
        "Expected FAKE_SHELL_READY (kitty keyboard query answered), \
         but the fake shell is stuck because \\x1b[?u got no response. \
         Screen:\n{}",
        screen
    );
}
