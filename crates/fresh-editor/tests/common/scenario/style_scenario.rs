//! `StyleScenario` — `StyledFrame` (cell-role × theme) assertions.
//!
//! **Production hook required (Phase 4):** the renderer's
//! cell-style decisions need to be exposed as a pure function
//! `style(snapshot: &RenderSnapshot, theme: &Theme,
//! roles: &RoleTable) -> StyledFrame`. The current `render()` body
//! couples layout + style + emit into one call.
//!
//! Until that extraction lands, scenarios constructed here will
//! panic with a precise blocker message; the data shape is real
//! and serialises into the corpus, so external drivers can already
//! generate StyleScenarios that the runner will pick up the moment
//! the production hook ships.

use crate::common::scenario::context::ThemeRef;
use crate::common::scenario::failure::ScenarioFailure;
use crate::common::scenario::input_event::InputEvent;
use crate::common::scenario::observable::StyledFrame;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StyleScenario {
    pub description: String,
    pub initial_text: String,
    #[serde(default)]
    pub theme: ThemeRef,
    pub events: Vec<InputEvent>,
    /// Cells the scenario asserts on. Use [`Inspect`] to scope.
    pub inspect: Inspect,
    pub expected: StyledFrame,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Inspect {
    Cell {
        row: u16,
        col: u16,
    },
    Row {
        row: u16,
    },
    Column {
        col: u16,
    },
    Region {
        row: u16,
        col: u16,
        rows: u16,
        cols: u16,
    },
    FullFrame,
}

impl Default for Inspect {
    fn default() -> Self {
        Inspect::FullFrame
    }
}

pub fn check_style_scenario(_s: StyleScenario) -> Result<(), ScenarioFailure> {
    Err(ScenarioFailure::InputProjectionFailed {
        description: "StyleScenario".into(),
        reason: "Phase 4 not yet implemented: needs `style()` extracted from \
            `render()` so the cell-role × theme projection is invocable \
            from tests. See docs/internal/e2e-test-migration-design.md §7.1."
            .into(),
    })
}

pub fn assert_style_scenario(s: StyleScenario) {
    if let Err(f) = check_style_scenario(s) {
        panic!("{f}");
    }
}
