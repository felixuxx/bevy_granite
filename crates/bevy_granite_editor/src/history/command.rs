use bevy::prelude::*;
use std::fmt;

/// Error type for command execution
#[derive(Debug, Clone)]
pub enum CommandError {
    EntityNotFound(Entity),
    ComponentNotFound(String),
    InvalidState(String),
    ExecutionFailed(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EntityNotFound(e) => write!(f, "Entity not found: {:?}", e),
            Self::ComponentNotFound(name) => write!(f, "Component not found: {}", name),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Command failed: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Core trait for all undoable commands
/// Implementations must be able to execute and undo their actions
pub trait EditorCommand: Send + Sync {
    /// Execute the command (perform the action)
    ///
    /// This should perform the actual change to the world.
    /// The command should store any data needed to undo the action.
    fn execute(&mut self, world: &mut World) -> CommandResult<()>;

    /// Undo the command (revert the action)
    ///
    /// This should restore the world to the state it was in before execute.
    fn undo(&mut self, world: &mut World) -> CommandResult<()>;

    /// Get a human-readable description of the command
    ///
    /// This is displayed in the UI for undo/redo tooltips
    fn description(&self) -> String;

    /// Clone the command for storage in history
    ///
    /// This is necessary because we need to store commands in a Vec
    fn clone_command(&self) -> Box<dyn EditorCommand>;

    /// Optional: called when command is removed from history
    fn on_discard(&mut self) {}
}

/// A stored command with metadata
pub struct StoredCommand {
    pub command: Box<dyn EditorCommand>,
    pub id: uuid::Uuid,
    pub timestamp: std::time::SystemTime,
}

impl StoredCommand {
    /// Create a new stored command
    pub fn new(command: Box<dyn EditorCommand>) -> Self {
        Self {
            command,
            id: uuid::Uuid::new_v4(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        executed: bool,
    }

    impl EditorCommand for TestCommand {
        fn execute(&mut self, _world: &mut World) -> CommandResult<()> {
            self.executed = true;
            Ok(())
        }

        fn undo(&mut self, _world: &mut World) -> CommandResult<()> {
            self.executed = false;
            Ok(())
        }

        fn description(&self) -> String {
            "Test Command".to_string()
        }

        fn clone_command(&self) -> Box<dyn EditorCommand> {
            Box::new(TestCommand {
                executed: self.executed,
            })
        }
    }

    #[test]
    fn test_command_error_display() {
        let entity = Entity::from_raw_u32(42).unwrap();
        let error = CommandError::EntityNotFound(entity);
        assert!(error.to_string().contains("Entity not found"));
    }
}
