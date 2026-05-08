//! Render a `WidgetSpec` tree into `Vec<TextPropertyEntry>`.
//!
//! This is the path from declarative spec to the bytes the existing
//! virtual-buffer pipeline already knows how to display. By going
//! through `TextPropertyEntry`, widgets paint via exactly the same
//! renderer that today's `setVirtualBufferContent` uses — no parallel
//! render path. This is what makes the new widget API additive: the
//! buffer mid-bytes are indistinguishable from hand-rolled output.
//!
//! v1 dispatches on four kinds:
//!   * `Row` — children laid out left-to-right within a single line
//!     (the result is one `TextPropertyEntry`).
//!   * `Col` — children stacked vertically (the result is one
//!     `TextPropertyEntry` per child output line).
//!   * `HintBar` — keyboard-hint footer (one `TextPropertyEntry`).
//!   * `Raw` — pass-through (zero interpretation; plugin's entries
//!     flow through unchanged).
//!
//! Future kinds (`Toggle`, `Button`, `TextInput`, `List`, `Tree`,
//! `Layer`, `Transient`, `Table`) extend the dispatch without
//! changing the public function signature.

use crate::widgets::registry::HitArea;
use fresh_core::api::{ButtonKind, HintEntry, OverlayColorSpec, OverlayOptions, WidgetSpec};
use fresh_core::text_property::{InlineOverlay, TextPropertyEntry};
use serde_json::json;

// Theme keys used by the v1 widget renderers. Centralized so future
// "role-based" theming (§7 of the design doc) has one place to
// substitute the role→key mapping.
const KEY_HELP_KEY_FG: &str = "ui.help_key_fg";
const KEY_TOGGLE_ON_FG: &str = "ui.tab_active_fg";
const KEY_FOCUSED_FG: &str = "ui.menu_active_fg";
const KEY_FOCUSED_BG: &str = "ui.menu_active_bg";
const KEY_DANGER_FG: &str = "ui.status_error_indicator_fg";

/// Render a spec to a flat `Vec<TextPropertyEntry>` plus a flat list
/// of click-routing `HitArea`s.
///
/// Entries are ready for `set_virtual_buffer_content`; hits are
/// installed in the `WidgetRegistry` so a later `mouse_click` can
/// dispatch a semantic `widget_event`.
pub fn render_spec(spec: &WidgetSpec) -> (Vec<TextPropertyEntry>, Vec<HitArea>) {
    render_collected(spec)
}

/// Internal renderer. Returns the entries and the hit areas
/// produced by `spec` *as if* it were rendered at row 0; callers
/// (Col, Row block path) shift `buffer_row` upward by their own
/// row offset before forwarding.
fn render_collected(spec: &WidgetSpec) -> (Vec<TextPropertyEntry>, Vec<HitArea>) {
    let mut entries: Vec<TextPropertyEntry> = Vec::new();
    let mut hits: Vec<HitArea> = Vec::new();
    match spec {
        WidgetSpec::Row { children, .. } => {
            // Rows collapse inline-sized children into a single
            // `TextPropertyEntry`. Children that emit multiple lines
            // (e.g. nested Col, Raw with several entries) flush the
            // accumulator and pass through. Hit areas from inline
            // children share the merged row; their byte offsets
            // shift by the merged-text length so far. Block
            // children's hits keep their own row index, biased by
            // the number of entries already emitted.
            let mut acc: Option<TextPropertyEntry> = None;
            for child in children {
                let (child_entries, child_hits) = render_collected(child);
                if child_entries.is_empty() {
                    debug_assert!(child_hits.is_empty(), "empty children produce no hits");
                    continue;
                }
                if child_entries.len() == 1 {
                    let mut child_entry = child_entries.into_iter().next().unwrap();
                    let inline_shift = match acc.as_ref() {
                        Some(e) => e.text.len(),
                        None => 0,
                    };
                    for mut h in child_hits {
                        // Inline child's hits all collapse onto the
                        // accumulator's row; byte ranges shift by the
                        // text length we've already merged.
                        h.byte_start += inline_shift;
                        h.byte_end += inline_shift;
                        // buffer_row stays at 0 — caller (Col / top
                        // level) will rebase it.
                        hits.push(h);
                    }
                    match acc.as_mut() {
                        Some(merged) => merge_inline(merged, &mut child_entry),
                        None => acc = Some(child_entry),
                    }
                } else {
                    // Multi-line child: flush the accumulator and
                    // emit the block. Hits from the block keep their
                    // own row index relative to the block's first
                    // line, plus the row offset of where the block
                    // lands in `entries`.
                    if let Some(merged) = acc.take() {
                        entries.push(merged);
                    }
                    let row_offset = entries.len() as u32;
                    for mut h in child_hits {
                        h.buffer_row += row_offset;
                        hits.push(h);
                    }
                    entries.extend(child_entries);
                }
            }
            if let Some(merged) = acc {
                entries.push(merged);
            }
        }
        WidgetSpec::Col { children, .. } => {
            for child in children {
                let (child_entries, child_hits) = render_collected(child);
                let row_offset = entries.len() as u32;
                for mut h in child_hits {
                    h.buffer_row += row_offset;
                    hits.push(h);
                }
                entries.extend(child_entries);
            }
        }
        WidgetSpec::HintBar {
            entries: hint_entries,
            ..
        } => {
            entries.push(render_hint_bar(hint_entries));
            // No hits — HintBar is read-only in v1. (When the
            // keymap layer arrives, individual entries become
            // clickable command targets.)
        }
        WidgetSpec::Toggle {
            checked,
            label,
            focused,
            key,
        } => {
            let entry = render_toggle(*checked, label, *focused);
            let byte_end = entry.text.len();
            hits.push(HitArea {
                widget_key: key.clone().unwrap_or_default(),
                widget_kind: "toggle",
                buffer_row: 0,
                byte_start: 0,
                byte_end,
                payload: json!({ "checked": !*checked }),
                event_type: "toggle",
            });
            entries.push(entry);
        }
        WidgetSpec::Button {
            label,
            focused,
            intent,
            key,
        } => {
            let entry = render_button(label, *focused, *intent);
            let byte_end = entry.text.len();
            hits.push(HitArea {
                widget_key: key.clone().unwrap_or_default(),
                widget_kind: "button",
                buffer_row: 0,
                byte_start: 0,
                byte_end,
                payload: json!({}),
                event_type: "activate",
            });
            entries.push(entry);
        }
        WidgetSpec::Spacer { cols, .. } => {
            // In an inline-row context a Spacer is N spaces; in a
            // block context (top-level / Col) it's a short blank
            // line. Either way: one entry, no hit areas.
            let cols = (*cols).min(4096) as usize;
            let mut text = String::with_capacity(cols);
            for _ in 0..cols {
                text.push(' ');
            }
            entries.push(TextPropertyEntry {
                text,
                properties: Default::default(),
                style: None,
                inline_overlays: Vec::new(),
            });
        }
        WidgetSpec::Raw {
            entries: raw_entries,
            ..
        } => {
            // Raw is the migration escape hatch: the plugin's own
            // bytes flow through unchanged. The plugin still owns
            // mouse clicks within Raw regions (via the existing
            // `mouse_click` hook); the widget runtime intentionally
            // emits no hit areas here.
            entries.extend(raw_entries.iter().cloned());
        }
    }
    (entries, hits)
}

/// Render a HintBar into a single `TextPropertyEntry`.
///
/// Layout: `<keys> <label>  <keys> <label>  …`. The key portion of
/// each entry is highlighted with the `ui.help_key_fg` theme key;
/// labels use the buffer's default foreground.
///
/// This replaces the per-plugin hand-rolled footer at e.g.
/// `crates/fresh-editor/plugins/search_replace.ts:535–541`,
/// `audit_mode.ts:1068–1158`, `pkg.ts:2136–2145`.
pub fn render_hint_bar(entries: &[HintEntry]) -> TextPropertyEntry {
    let separator = "  ";
    let mut text = String::new();
    let mut overlays = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            text.push_str(separator);
        }
        let key_start = text.len();
        text.push_str(&entry.keys);
        let key_end = text.len();
        if key_end > key_start {
            overlays.push(InlineOverlay {
                start: key_start,
                end: key_end,
                style: OverlayOptions {
                    fg: Some(OverlayColorSpec::theme_key(KEY_HELP_KEY_FG)),
                    bold: true,
                    ..Default::default()
                },
                properties: Default::default(),
            });
        }
        if !entry.label.is_empty() {
            text.push(' ');
            text.push_str(&entry.label);
        }
    }
    TextPropertyEntry {
        text,
        properties: Default::default(),
        style: None,
        inline_overlays: overlays,
    }
}

/// Render a `Toggle` to a single `TextPropertyEntry`.
///
/// Layout: `[v] label` when checked, `[ ] label` when not. The check
/// glyph is colored via `ui.tab_active_fg` when checked (no override
/// when unchecked). When focused, the entire entry is given a focused
/// fg/bg pair (`ui.menu_active_fg`/`ui.menu_active_bg`) plus bold —
/// matching the Settings UI's selected-control affordance.
pub fn render_toggle(checked: bool, label: &str, focused: bool) -> TextPropertyEntry {
    let glyph = if checked { "[v]" } else { "[ ]" };
    let mut text = String::with_capacity(glyph.len() + 1 + label.len());
    text.push_str(glyph);
    text.push(' ');
    text.push_str(label);

    let mut overlays = Vec::new();

    // Check-glyph color (only when checked — leaves default fg
    // when unchecked, which is what plugins do today).
    if checked {
        overlays.push(InlineOverlay {
            start: 0,
            end: glyph.len(),
            style: OverlayOptions {
                fg: Some(OverlayColorSpec::theme_key(KEY_TOGGLE_ON_FG)),
                bold: true,
                ..Default::default()
            },
            properties: Default::default(),
        });
    }

    // Focused: full-entry fg/bg + bold.
    if focused {
        overlays.push(InlineOverlay {
            start: 0,
            end: text.len(),
            style: OverlayOptions {
                fg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_FG)),
                bg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_BG)),
                bold: true,
                ..Default::default()
            },
            properties: Default::default(),
        });
    }

    TextPropertyEntry {
        text,
        properties: Default::default(),
        style: None,
        inline_overlays: overlays,
    }
}

/// Render a `Button` to a single `TextPropertyEntry`.
///
/// Layout: `[ Label ]` (with explicit space padding so the label
/// is visually inset from the brackets). Styling depends on `kind`
/// and `focused`:
///
/// * `Normal`     — default fg; focused → fg/bg flip + bold.
/// * `Primary`    — bold; focused → fg/bg flip.
/// * `Danger`     — red fg (theme `ui.status_error_indicator_fg`);
///                  focused → bold.
pub fn render_button(label: &str, focused: bool, kind: ButtonKind) -> TextPropertyEntry {
    let text = format!("[ {} ]", label);
    let mut overlays = Vec::new();

    let base_style = match kind {
        ButtonKind::Normal => OverlayOptions::default(),
        ButtonKind::Primary => OverlayOptions {
            bold: true,
            ..Default::default()
        },
        ButtonKind::Danger => OverlayOptions {
            fg: Some(OverlayColorSpec::theme_key(KEY_DANGER_FG)),
            ..Default::default()
        },
    };

    let style = if focused {
        OverlayOptions {
            fg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_FG)),
            bg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_BG)),
            bold: true,
            ..base_style
        }
    } else {
        base_style
    };

    // Only emit an overlay if the style is non-default — keeps the
    // serialized entry tight.
    if style.fg.is_some()
        || style.bg.is_some()
        || style.bold
        || style.italic
        || style.underline
        || style.strikethrough
    {
        overlays.push(InlineOverlay {
            start: 0,
            end: text.len(),
            style,
            properties: Default::default(),
        });
    }

    TextPropertyEntry {
        text,
        properties: Default::default(),
        style: None,
        inline_overlays: overlays,
    }
}

/// Merge `next` into `merged` for the inline-row collapse path.
/// `next`'s overlays are byte-shifted to account for the merged
/// text length so far.
fn merge_inline(merged: &mut TextPropertyEntry, next: &mut TextPropertyEntry) {
    let shift = merged.text.len();
    merged.text.push_str(&next.text);
    for overlay in next.inline_overlays.drain(..) {
        merged.inline_overlays.push(InlineOverlay {
            start: overlay.start + shift,
            end: overlay.end + shift,
            style: overlay.style,
            properties: overlay.properties,
        });
    }
    // `style` and `properties` from `next` are dropped — Row inline
    // collapse only preserves inline_overlays. Whole-entry style on
    // an inline-row child has no meaningful semantics here; if a
    // plugin needs whole-line styling it should produce a Col with
    // the styled child as its sole element.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_bar_renders_entries_with_key_overlays() {
        let entries = vec![
            HintEntry {
                keys: "Tab".into(),
                label: "next".into(),
            },
            HintEntry {
                keys: "Esc".into(),
                label: "close".into(),
            },
        ];
        let entry = render_hint_bar(&entries);
        assert_eq!(entry.text, "Tab next  Esc close");
        assert_eq!(entry.inline_overlays.len(), 2);
        // First overlay covers "Tab" (bytes 0..3).
        assert_eq!(entry.inline_overlays[0].start, 0);
        assert_eq!(entry.inline_overlays[0].end, 3);
        // Second overlay covers "Esc" (bytes 10..13).
        assert_eq!(entry.inline_overlays[1].start, 10);
        assert_eq!(entry.inline_overlays[1].end, 13);
    }

    #[test]
    fn hint_bar_omits_label_when_empty() {
        let entries = vec![HintEntry {
            keys: "?".into(),
            label: "".into(),
        }];
        let entry = render_hint_bar(&entries);
        assert_eq!(entry.text, "?");
    }

    #[test]
    fn col_stacks_children_top_to_bottom() {
        let spec = WidgetSpec::Col {
            children: vec![
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "A".into(),
                        label: "alpha".into(),
                    }],
                    key: None,
                },
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "B".into(),
                        label: "beta".into(),
                    }],
                    key: None,
                },
            ],
            key: None,
        };
        let (out, hits) = render_spec(&spec);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].text, "A alpha");
        assert_eq!(out[1].text, "B beta");
        assert!(hits.is_empty(), "HintBar emits no hit areas in v1");
    }

    #[test]
    fn raw_passes_through_unchanged() {
        let spec = WidgetSpec::Raw {
            entries: vec![TextPropertyEntry::text("hello")],
            key: None,
        };
        let (out, hits) = render_spec(&spec);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "hello");
        assert!(hits.is_empty());
    }

    #[test]
    fn toggle_checked_emits_glyph_overlay() {
        let entry = render_toggle(true, "Case", false);
        assert_eq!(entry.text, "[v] Case");
        // One overlay for the glyph, no focused overlay.
        assert_eq!(entry.inline_overlays.len(), 1);
        assert_eq!(entry.inline_overlays[0].start, 0);
        assert_eq!(entry.inline_overlays[0].end, 3);
    }

    #[test]
    fn toggle_unchecked_no_glyph_overlay() {
        let entry = render_toggle(false, "Case", false);
        assert_eq!(entry.text, "[ ] Case");
        assert_eq!(entry.inline_overlays.len(), 0);
    }

    #[test]
    fn toggle_focused_adds_full_entry_overlay() {
        let entry = render_toggle(true, "Case", true);
        // Glyph overlay + focused overlay.
        assert_eq!(entry.inline_overlays.len(), 2);
        // Focused overlay spans the full entry.
        assert_eq!(entry.inline_overlays[1].start, 0);
        assert_eq!(entry.inline_overlays[1].end, entry.text.len());
        assert!(entry.inline_overlays[1].style.bold);
    }

    #[test]
    fn button_normal_unfocused_has_no_overlay() {
        let entry = render_button("Replace All", false, ButtonKind::Normal);
        assert_eq!(entry.text, "[ Replace All ]");
        assert!(entry.inline_overlays.is_empty());
    }

    #[test]
    fn button_primary_is_bold() {
        let entry = render_button("Submit", false, ButtonKind::Primary);
        assert_eq!(entry.inline_overlays.len(), 1);
        assert!(entry.inline_overlays[0].style.bold);
    }

    #[test]
    fn button_danger_uses_error_theme_key() {
        let entry = render_button("Delete", false, ButtonKind::Danger);
        assert_eq!(entry.inline_overlays.len(), 1);
        let fg = entry.inline_overlays[0].style.fg.as_ref().unwrap();
        assert_eq!(fg.as_theme_key(), Some("ui.status_error_indicator_fg"));
    }

    #[test]
    fn button_focused_overrides_with_menu_active_keys() {
        let entry = render_button("OK", true, ButtonKind::Normal);
        let style = &entry.inline_overlays[0].style;
        assert_eq!(
            style.fg.as_ref().and_then(|c| c.as_theme_key()),
            Some("ui.menu_active_fg")
        );
        assert_eq!(
            style.bg.as_ref().and_then(|c| c.as_theme_key()),
            Some("ui.menu_active_bg")
        );
        assert!(style.bold);
    }

    #[test]
    fn spacer_in_row_pads_with_spaces() {
        let spec = WidgetSpec::Row {
            children: vec![
                WidgetSpec::Toggle {
                    checked: false,
                    label: "A".into(),
                    focused: false,
                    key: None,
                },
                WidgetSpec::Spacer { cols: 4, key: None },
                WidgetSpec::Button {
                    label: "Go".into(),
                    focused: false,
                    intent: ButtonKind::Normal,
                    key: None,
                },
            ],
            key: None,
        };
        let (out, _hits) = render_spec(&spec);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "[ ] A    [ Go ]");
    }

    #[test]
    fn row_collapses_inline_children_with_shifted_overlays() {
        let spec = WidgetSpec::Row {
            children: vec![
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "Tab".into(),
                        label: "x".into(),
                    }],
                    key: None,
                },
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "Esc".into(),
                        label: "y".into(),
                    }],
                    key: None,
                },
            ],
            key: None,
        };
        let (out, _hits) = render_spec(&spec);
        assert_eq!(out.len(), 1);
        // Two adjacent HintBars are concatenated; the second's overlay shifts.
        assert_eq!(out[0].text, "Tab xEsc y");
        assert_eq!(out[0].inline_overlays.len(), 2);
        assert_eq!(out[0].inline_overlays[1].start, 5);
        assert_eq!(out[0].inline_overlays[1].end, 8);
    }

    // -------------------------------------------------------------
    // Hit-area tests
    // -------------------------------------------------------------

    #[test]
    fn toggle_emits_hit_area_with_toggle_payload() {
        let spec = WidgetSpec::Toggle {
            checked: false,
            label: "Case".into(),
            focused: false,
            key: Some("case".into()),
        };
        let (_entries, hits) = render_spec(&spec);
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert_eq!(h.widget_key, "case");
        assert_eq!(h.widget_kind, "toggle");
        assert_eq!(h.event_type, "toggle");
        assert_eq!(h.buffer_row, 0);
        assert_eq!(h.byte_start, 0);
        assert_eq!(h.byte_end, "[ ] Case".len());
        assert_eq!(h.payload, json!({"checked": true}));
    }

    #[test]
    fn button_emits_hit_area_with_activate_payload() {
        let spec = WidgetSpec::Button {
            label: "Replace All".into(),
            focused: false,
            intent: ButtonKind::Primary,
            key: Some("replace".into()),
        };
        let (_entries, hits) = render_spec(&spec);
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert_eq!(h.widget_key, "replace");
        assert_eq!(h.widget_kind, "button");
        assert_eq!(h.event_type, "activate");
        assert_eq!(h.byte_end, "[ Replace All ]".len());
        assert_eq!(h.payload, json!({}));
    }

    #[test]
    fn row_inline_collapse_shifts_hit_byte_offsets() {
        let spec = WidgetSpec::Row {
            children: vec![
                WidgetSpec::Toggle {
                    checked: true,
                    label: "A".into(),
                    focused: false,
                    key: Some("a".into()),
                },
                WidgetSpec::Spacer { cols: 2, key: None },
                WidgetSpec::Toggle {
                    checked: false,
                    label: "B".into(),
                    focused: false,
                    key: Some("b".into()),
                },
            ],
            key: None,
        };
        let (entries, hits) = render_spec(&spec);
        // One merged row with text "[v] A  [ ] B"
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "[v] A  [ ] B");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].widget_key, "a");
        assert_eq!(hits[0].buffer_row, 0);
        assert_eq!(hits[0].byte_start, 0);
        assert_eq!(hits[0].byte_end, 5); // "[v] A".len()
        // Second toggle shifts past first toggle ("[v] A".len() = 5)
        // + spacer ("  ".len() = 2) = 7.
        assert_eq!(hits[1].widget_key, "b");
        assert_eq!(hits[1].buffer_row, 0);
        assert_eq!(hits[1].byte_start, 7);
        assert_eq!(hits[1].byte_end, 12);
    }

    #[test]
    fn col_stacks_hit_rows() {
        let spec = WidgetSpec::Col {
            children: vec![
                WidgetSpec::Toggle {
                    checked: false,
                    label: "row0".into(),
                    focused: false,
                    key: Some("k0".into()),
                },
                WidgetSpec::Toggle {
                    checked: true,
                    label: "row1".into(),
                    focused: false,
                    key: Some("k1".into()),
                },
            ],
            key: None,
        };
        let (_entries, hits) = render_spec(&spec);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].buffer_row, 0);
        assert_eq!(hits[1].buffer_row, 1);
    }

    #[test]
    fn raw_inside_col_offsets_following_hits() {
        let spec = WidgetSpec::Col {
            children: vec![
                WidgetSpec::Raw {
                    entries: vec![
                        TextPropertyEntry::text("line0"),
                        TextPropertyEntry::text("line1"),
                        TextPropertyEntry::text("line2"),
                    ],
                    key: None,
                },
                WidgetSpec::Toggle {
                    checked: false,
                    label: "after raw".into(),
                    focused: false,
                    key: Some("post".into()),
                },
            ],
            key: None,
        };
        let (entries, hits) = render_spec(&spec);
        assert_eq!(entries.len(), 4);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].buffer_row, 3);
    }
}
