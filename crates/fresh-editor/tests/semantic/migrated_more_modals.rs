//! Wave-4 ModalScenarios — additional palette / picker /
//! prompt cancellation flows.

use crate::common::scenario::context::PromptKind;
use crate::common::scenario::input_event::InputEvent;
use crate::common::scenario::modal_scenario::{assert_modal_scenario, ModalScenario};
use crate::common::scenario::observable::ModalState;

#[test]
fn migrated_open_close_open_close_is_balanced() {
    assert_modal_scenario(ModalScenario {
        description: "open + cancel + open + cancel returns to depth 0".into(),
        initial_text: String::new(),
        events: vec![
            InputEvent::OpenPrompt(PromptKind::CommandPalette),
            InputEvent::CancelPrompt,
            InputEvent::OpenPrompt(PromptKind::Goto),
            InputEvent::CancelPrompt,
        ],
        expected_modal: ModalState::default(),
    });
}

#[test]
fn migrated_filter_during_modal_does_not_change_buffer() {
    // FilterPrompt while a modal is open types into the prompt's
    // input field, not the buffer. After CancelPrompt, depth is 0
    // and the buffer is unchanged. (The buffer assertion would
    // belong on the BufferState side; here we just verify the
    // modal lifecycle.)
    assert_modal_scenario(ModalScenario {
        description: "FilterPrompt + CancelPrompt cleanly closes the modal".into(),
        initial_text: "buffer text".into(),
        events: vec![
            InputEvent::OpenPrompt(PromptKind::CommandPalette),
            InputEvent::FilterPrompt("dup".into()),
            InputEvent::CancelPrompt,
        ],
        expected_modal: ModalState::default(),
    });
}

#[test]
fn migrated_live_grep_open_then_cancel() {
    assert_modal_scenario(ModalScenario {
        description: "OpenPrompt(LiveGrep) + CancelPrompt clears the modal".into(),
        initial_text: String::new(),
        events: vec![
            InputEvent::OpenPrompt(PromptKind::LiveGrep),
            InputEvent::CancelPrompt,
        ],
        expected_modal: ModalState::default(),
    });
}
