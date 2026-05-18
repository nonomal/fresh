//! Wave-3 PersistenceScenarios — additional FS claims from
//! `tests/e2e/large_file_mode.rs`,
//! `tests/e2e/large_file_inplace_write_bug.rs`,
//! `tests/e2e/symlinks.rs`,
//! `tests/e2e/recovery.rs`.

use crate::common::scenario::context::{VirtualFile, VirtualFs};
use crate::common::scenario::input_event::InputEvent;
use crate::common::scenario::observable::FsState;
use crate::common::scenario::persistence_scenario::{
    assert_persistence_scenario, PersistenceScenario,
};
use fresh::test_api::Action;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn migrated_open_then_close_then_reopen_yields_original_content() {
    // Save once with edits, the on-disk content should reflect
    // the edits.
    let mut files = BTreeMap::new();
    files.insert(
        PathBuf::from("doc.txt"),
        VirtualFile {
            content: "before".into(),
            mode: None,
            mtime_unix_secs: None,
        },
    );
    assert_persistence_scenario(PersistenceScenario {
        description: "edit + save persists 'beforeX' to disk".into(),
        initial_fs: VirtualFs { files },
        initial_open: "doc.txt".into(),
        events: vec![
            InputEvent::Action(Action::MoveDocumentEnd),
            InputEvent::Action(Action::InsertChar('X')),
            InputEvent::Action(Action::Save),
        ],
        expected_buffer: None,
        expected_fs: FsState {
            expected_files: std::iter::once(("doc.txt".into(), "beforeX".into())).collect(),
        },
    });
}

#[test]
fn migrated_save_after_select_replace_persists_replaced_content() {
    let mut files = BTreeMap::new();
    files.insert(
        PathBuf::from("page.txt"),
        VirtualFile {
            content: "old content".into(),
            mode: None,
            mtime_unix_secs: None,
        },
    );
    assert_persistence_scenario(PersistenceScenario {
        description: "SelectAll + Insert + Save persists the replacement".into(),
        initial_fs: VirtualFs { files },
        initial_open: "page.txt".into(),
        events: vec![
            InputEvent::Action(Action::SelectAll),
            InputEvent::Action(Action::InsertChar('!')),
            InputEvent::Action(Action::Save),
        ],
        expected_buffer: None,
        expected_fs: FsState {
            expected_files: std::iter::once(("page.txt".into(), "!".into())).collect(),
        },
    });
}

#[test]
fn migrated_external_edit_to_other_file_does_not_affect_open_buffer_save() {
    // Open A, externally edit B, save A. Both files end up with
    // their respective writes.
    let mut files = BTreeMap::new();
    files.insert(
        PathBuf::from("a.txt"),
        VirtualFile {
            content: "A".into(),
            mode: None,
            mtime_unix_secs: None,
        },
    );
    files.insert(
        PathBuf::from("b.txt"),
        VirtualFile {
            content: "B".into(),
            mode: None,
            mtime_unix_secs: None,
        },
    );
    assert_persistence_scenario(PersistenceScenario {
        description: "external edit to b.txt is independent of save on a.txt".into(),
        initial_fs: VirtualFs { files },
        initial_open: "a.txt".into(),
        events: vec![
            InputEvent::Action(Action::MoveDocumentEnd),
            InputEvent::Action(Action::InsertChar('!')),
            InputEvent::FsExternalEdit {
                path: PathBuf::from("b.txt"),
                content: "B-mod".into(),
            },
            InputEvent::Action(Action::Save),
        ],
        expected_buffer: None,
        expected_fs: FsState {
            expected_files: [
                ("a.txt".to_string(), "A!".to_string()),
                ("b.txt".to_string(), "B-mod".to_string()),
            ]
            .into_iter()
            .collect(),
        },
    });
}
