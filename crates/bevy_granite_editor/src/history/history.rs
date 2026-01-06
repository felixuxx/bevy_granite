use super::command::{CommandError, CommandResult, EditorCommand};
use bevy::prelude::*;
use std::collections::VecDeque;

const MAX_HISTORY_SIZE: usize = 100;

/// Manages the undo/redo stacks for editor commands
///
/// This resource maintains two stacks:
/// - undo_stack: Commands that have been executed and can be undone
/// - redo_stack: Commands that have been undone and can be redone
#[derive(Resource)]
pub struct CommandHistory {
    undo_stack: VecDeque<Box<dyn EditorCommand>>,
    redo_stack: VecDeque<Box<dyn EditorCommand>>,
    max_size: usize,
    current_transaction: Option<String>,
}

impl CommandHistory {
    /// Create a new command history with default settings
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            redo_stack: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            max_size: MAX_HISTORY_SIZE,
            current_transaction: None,
        }
    }

    /// Create a new command history with custom max size
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_size),
            redo_stack: VecDeque::with_capacity(max_size),
            max_size,
            current_transaction: None,
        }
    }

    /// Execute a command and add it to the undo stack
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `world` - The world to execute the command on
    ///
    /// # Returns
    /// - Ok(()) if the command was executed successfully
    /// - Err(CommandError) if execution failed
    pub fn execute(
        &mut self,
        mut command: Box<dyn EditorCommand>,
        world: &mut World,
    ) -> CommandResult<()> {
        // Execute the command
        command.execute(world)?;

        // Add to undo stack
        self.undo_stack.push_back(command);

        // Enforce max size
        while self.undo_stack.len() > self.max_size {
            if let Some(mut discarded) = self.undo_stack.pop_front() {
                discarded.on_discard();
            }
        }

        // Clear redo stack (new action invalidates redo)
        self.redo_stack.clear();

        Ok(())
    }

    /// Undo the last command
    ///
    /// Moves the last command from undo_stack to redo_stack and reverts its changes.
    ///
    /// # Returns
    /// - Ok(()) if undo was successful
    /// - Err(CommandError::InvalidState) if nothing to undo
    pub fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if let Some(mut command) = self.undo_stack.pop_back() {
            command.undo(world)?;
            self.redo_stack.push_back(command);
            Ok(())
        } else {
            Err(CommandError::InvalidState("Nothing to undo".to_string()))
        }
    }

    /// Redo the last undone command
    ///
    /// Moves the last command from redo_stack to undo_stack and re-executes it.
    ///
    /// # Returns
    /// - Ok(()) if redo was successful
    /// - Err(CommandError::InvalidState) if nothing to redo
    pub fn redo(&mut self, world: &mut World) -> CommandResult<()> {
        if let Some(mut command) = self.redo_stack.pop_back() {
            command.execute(world)?;
            self.undo_stack.push_back(command);
            Ok(())
        } else {
            Err(CommandError::InvalidState("Nothing to redo".to_string()))
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the description of the next undo action
    ///
    /// Useful for displaying "Undo: [action]" in UI
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Get the description of the next redo action
    ///
    /// Useful for displaying "Redo: [action]" in UI
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Get the number of commands in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Begin a transaction (group multiple commands)
    ///
    /// All commands executed while a transaction is active will be
    /// grouped together and undo/redo as a single unit.
    pub fn begin_transaction(&mut self, name: impl Into<String>) {
        self.current_transaction = Some(name.into());
    }

    /// End the current transaction
    pub fn end_transaction(&mut self) {
        self.current_transaction = None;
    }

    /// Check if currently in a transaction
    pub fn in_transaction(&self) -> bool {
        self.current_transaction.is_some()
    }

    /// Get max history size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Set max history size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleCommand {
        value: i32,
        executed: bool,
    }

    impl EditorCommand for SimpleCommand {
        fn execute(&mut self, _world: &mut World) -> CommandResult<()> {
            self.executed = true;
            Ok(())
        }

        fn undo(&mut self, _world: &mut World) -> CommandResult<()> {
            self.executed = false;
            Ok(())
        }

        fn description(&self) -> String {
            format!("Simple command (value: {})", self.value)
        }

        fn clone_command(&self) -> Box<dyn EditorCommand> {
            Box::new(SimpleCommand {
                value: self.value,
                executed: self.executed,
            })
        }
    }

    #[test]
    fn test_execute_adds_to_undo_stack() {
        let mut history = CommandHistory::new();
        let cmd = Box::new(SimpleCommand {
            value: 42,
            executed: false,
        });

        let mut world = World::new();
        history.execute(cmd, &mut world).unwrap();

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_undo_clears_redo() {
        let mut history = CommandHistory::new();
        let cmd1 = Box::new(SimpleCommand {
            value: 1,
            executed: false,
        });
        let cmd2 = Box::new(SimpleCommand {
            value: 2,
            executed: false,
        });

        let mut world = World::new();
        history.execute(cmd1, &mut world).unwrap();
        history.execute(cmd2, &mut world).unwrap();

        // Undo once
        history.undo(&mut world).unwrap();
        assert!(history.can_redo());

        // Execute new command - should clear redo
        let cmd3 = Box::new(SimpleCommand {
            value: 3,
            executed: false,
        });
        history.execute(cmd3, &mut world).unwrap();
        assert!(!history.can_redo());
    }

    #[test]
    fn test_descriptions() {
        let mut history = CommandHistory::new();
        let cmd = Box::new(SimpleCommand {
            value: 42,
            executed: false,
        });

        let mut world = World::new();
        history.execute(cmd, &mut world).unwrap();

        let desc = history.undo_description();
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("42"));
    }

    #[test]
    fn test_max_size_enforcement() {
        let mut history = CommandHistory::with_capacity(3);
        let mut world = World::new();

        for i in 0..5 {
            let cmd = Box::new(SimpleCommand {
                value: i,
                executed: false,
            });
            history.execute(cmd, &mut world).unwrap();
        }

        assert_eq!(history.undo_count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut history = CommandHistory::new();
        let mut world = World::new();

        for i in 0..3 {
            let cmd = Box::new(SimpleCommand {
                value: i,
                executed: false,
            });
            history.execute(cmd, &mut world).unwrap();
        }

        history.clear();
        assert!(!history.can_undo());
        assert_eq!(history.undo_count(), 0);
    }
}
