//! Undo/Redo system for the Bevy Granite editor
//!
//! This module provides a comprehensive undo/redo system using the command pattern.
//! Commands can be executed, undone, and redone. The history maintains separate stacks
//! for undo and redo operations.
//!
//! # Example
//!
//! ```ignore
//! use bevy_granite_editor::history::{CommandHistory, TransformCommand};
//!
//! let mut history = CommandHistory::new();
//! let cmd = TransformCommand::new(entity, old_transform, new_transform);
//! history.execute(Box::new(cmd), &mut world)?;
//!
//! // Later, undo the change
//! history.undo(&mut world)?;
//!
//! // Or redo it
//! history.redo(&mut world)?;
//! ```

pub mod command;
pub mod commands;
pub mod gizmo_integration;
pub mod history;
pub mod plugin;

// Re-export main types for convenience
pub use command::{CommandError, CommandResult, EditorCommand, StoredCommand};
pub use commands::{EntityCreateCommand, EntityDeleteCommand, TransformCommand};
pub use history::CommandHistory;
pub use plugin::CommandHistoryPlugin;

#[cfg(test)]
mod tests;
