//! `RenderSnapshot` — typed, theme-free layout observable.
//!
//! Produced by extracting layout state from a live editor *after* a
//! single render pass settles the viewport. Asserted on by
//! [`super::layout_scenario::LayoutScenario`].
//!
//! Today's implementation pulls fields from `EditorTestApi` —
//! `viewport_top_byte`, `hardware_cursor_position`, `gutter_width`,
//! `visible_byte_range`. The doc's longer-term `RenderSnapshot`
//! includes per-row segments, decorations, popup placement; those
//! get added incrementally as layout scenarios demand them. Adding
//! a field here means adding the corresponding accessor on
//! `EditorTestApi`.

use crate::common::harness::EditorTestHarness;
use crate::common::scenario::observable::Observable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RenderSnapshot {
    pub width: u16,
    pub height: u16,
    pub viewport: ViewportSnapshot,
    pub hardware_cursor: Option<(u16, u16)>,
    pub gutter_width: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ViewportSnapshot {
    pub top_byte: usize,
    /// Byte range currently visible. None ⇒ unknown (extension not
    /// yet wired through `EditorTestApi`).
    #[serde(default)]
    pub visible_byte_range: Option<(usize, usize)>,
}

impl Observable for RenderSnapshot {
    fn extract(harness: &mut EditorTestHarness) -> Self {
        let _ = harness.render();
        let api = harness.api_mut();
        RenderSnapshot {
            width: api.terminal_width(),
            height: api.terminal_height(),
            viewport: ViewportSnapshot {
                top_byte: api.viewport_top_byte(),
                visible_byte_range: api.visible_byte_range(),
            },
            hardware_cursor: api.hardware_cursor_position(),
            gutter_width: api.gutter_width(),
        }
    }
}

/// Partial expectation: only fields set on the expectation are
/// asserted. Unspecified fields wildcard-match the editor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderSnapshotExpect {
    #[serde(default)]
    pub viewport_top_byte: Option<usize>,
    #[serde(default)]
    pub hardware_cursor: Option<(u16, u16)>,
    #[serde(default)]
    pub gutter_width: Option<u16>,
    #[serde(default)]
    pub visible_byte_range: Option<(usize, usize)>,
}

impl RenderSnapshotExpect {
    /// Returns `Some((field, expected, actual))` on the first
    /// mismatch.
    pub fn check_against(&self, actual: &RenderSnapshot) -> Option<(&'static str, String, String)> {
        if let Some(want) = self.viewport_top_byte {
            if want != actual.viewport.top_byte {
                return Some((
                    "viewport_top_byte",
                    want.to_string(),
                    actual.viewport.top_byte.to_string(),
                ));
            }
        }
        if let Some(want) = self.hardware_cursor {
            if Some(want) != actual.hardware_cursor {
                return Some((
                    "hardware_cursor",
                    format!("{want:?}"),
                    format!("{:?}", actual.hardware_cursor),
                ));
            }
        }
        if let Some(want) = self.gutter_width {
            if want != actual.gutter_width {
                return Some((
                    "gutter_width",
                    want.to_string(),
                    actual.gutter_width.to_string(),
                ));
            }
        }
        if let Some(want) = self.visible_byte_range {
            if Some(want) != actual.viewport.visible_byte_range {
                return Some((
                    "visible_byte_range",
                    format!("{want:?}"),
                    format!("{:?}", actual.viewport.visible_byte_range),
                ));
            }
        }
        None
    }
}
