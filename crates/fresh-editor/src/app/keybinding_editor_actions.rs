//! Keybinding editor action handling
//!
//! This module provides the action handlers for the keybinding editor modal.

use super::keybinding_editor::KeybindingEditor;
use super::Editor;
use crate::input::handler::InputResult;
use crate::view::keybinding_editor::{handle_keybinding_editor_input, KeybindingEditorAction};
use crossterm::event::KeyEvent;

impl Editor {
    /// Open the keybinding editor modal
    pub fn open_keybinding_editor(&mut self) {
        let config_path = self.dir_context.config_path().display().to_string();
        self.keybinding_editor = Some(KeybindingEditor::new(
            &self.config,
            &self.keybindings,
            config_path,
        ));
    }

    /// Handle input when keybinding editor is active
    pub fn handle_keybinding_editor_input(&mut self, event: &KeyEvent) -> InputResult {
        let mut editor = match self.keybinding_editor.take() {
            Some(e) => e,
            None => return InputResult::Ignored,
        };

        let action = handle_keybinding_editor_input(&mut editor, event);

        match action {
            KeybindingEditorAction::Consumed => {
                self.keybinding_editor = Some(editor);
                InputResult::Consumed
            }
            KeybindingEditorAction::Close => {
                // Close without saving
                self.set_status_message("Keybinding editor closed".to_string());
                InputResult::Consumed
            }
            KeybindingEditorAction::SaveAndClose => {
                // Save custom bindings to config
                self.save_keybinding_editor_changes(&editor);
                InputResult::Consumed
            }
            KeybindingEditorAction::StatusMessage(msg) => {
                self.set_status_message(msg);
                self.keybinding_editor = Some(editor);
                InputResult::Consumed
            }
        }
    }

    /// Save keybinding editor changes to config
    fn save_keybinding_editor_changes(&mut self, editor: &KeybindingEditor) {
        if !editor.has_changes {
            return;
        }

        // Collect all custom bindings from the editor
        let new_bindings = editor.get_custom_bindings();

        // Add new bindings to existing custom keybindings
        for binding in new_bindings {
            self.config.keybindings.push(binding);
        }

        // Rebuild the keybinding resolver
        self.keybindings = crate::input::keybindings::KeybindingResolver::new(&self.config);

        // Save to config file via the pending changes mechanism
        let config_value = match serde_json::to_value(&self.config.keybindings) {
            Ok(v) => v,
            Err(e) => {
                self.set_status_message(format!("Failed to serialize keybindings: {}", e));
                return;
            }
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert("/keybindings".to_string(), config_value);

        let resolver = crate::config_io::ConfigResolver::new(
            self.dir_context.clone(),
            self.working_dir.clone(),
        );

        match resolver.save_changes_to_layer(
            &changes,
            &std::collections::HashSet::new(),
            crate::config_io::ConfigLayer::User,
        ) {
            Ok(()) => {
                self.set_status_message("Keybinding changes saved".to_string());
            }
            Err(e) => {
                self.set_status_message(format!("Failed to save keybindings: {}", e));
            }
        }
    }

    /// Check if keybinding editor is active
    pub fn is_keybinding_editor_active(&self) -> bool {
        self.keybinding_editor.is_some()
    }
}
