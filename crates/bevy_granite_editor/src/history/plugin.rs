use bevy::ecs::change_detection::Mut;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy_granite_core::events::{RequestRedoEvent, RequestUndoEvent};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use super::gizmo_integration::{
    process_pending_transform_commands, record_user_transform_changes, PendingTransformCommands,
};
use super::history::CommandHistory;

/// Plugin that manages the command history and undo/redo system
pub struct CommandHistoryPlugin;

impl Plugin for CommandHistoryPlugin {
    fn build(&self, app: &mut App) {
        app
            // Insert the command history resource
            .insert_resource(CommandHistory::new())
            // Insert a queue for pending undo/redo operations
            .insert_resource(UndoRedoQueue::new())
            // Insert pending transform commands queue
            .insert_resource(PendingTransformCommands::new())
            // Add systems in order
            .add_systems(Update, record_user_transform_changes)
            .add_systems(Update, process_pending_transform_commands)
            .add_systems(Update, queue_undo_redo_requests)
            .add_systems(Update, process_undo_redo_queue_exclusive);
    }
}

/// A queue to store pending undo/redo operations
#[derive(Resource)]
struct UndoRedoQueue {
    undo_count: usize,
    redo_count: usize,
}

impl UndoRedoQueue {
    fn new() -> Self {
        Self {
            undo_count: 0,
            redo_count: 0,
        }
    }

    fn queue_undo(&mut self) {
        self.undo_count += 1;
    }

    fn queue_redo(&mut self) {
        self.redo_count += 1;
    }

    fn take_undo_count(&mut self) -> usize {
        let count = self.undo_count;
        self.undo_count = 0;
        count
    }

    fn take_redo_count(&mut self) -> usize {
        let count = self.redo_count;
        self.redo_count = 0;
        count
    }
}

/// System that queues undo/redo requests from events
fn queue_undo_redo_requests(
    mut undo_events: MessageReader<RequestUndoEvent>,
    mut redo_events: MessageReader<RequestRedoEvent>,
    mut queue: ResMut<UndoRedoQueue>,
) {
    for _ in undo_events.read() {
        queue.queue_undo();
    }

    for _ in redo_events.read() {
        queue.queue_redo();
    }
}

/// Exclusive system that processes the undo/redo queue and applies changes to the world
/// This uses resource_scope to properly handle borrows
fn process_undo_redo_queue_exclusive(world: &mut World) {
    let undo_count = {
        let mut queue = world.get_resource_mut::<UndoRedoQueue>().unwrap();
        queue.take_undo_count()
    };

    let redo_count = {
        let mut queue = world.get_resource_mut::<UndoRedoQueue>().unwrap();
        queue.take_redo_count()
    };

    // Process undos using resource_scope to properly handle borrows
    for _ in 0..undo_count {
        let can_undo = world
            .get_resource::<CommandHistory>()
            .map(|h| h.can_undo())
            .unwrap_or(false);

        if !can_undo {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::System,
                "Nothing to undo"
            );
            continue;
        }

        // Get the description before we consume the resource
        let desc = world
            .get_resource::<CommandHistory>()
            .and_then(|h| h.undo_description());

        // Use resource_scope to safely call undo which needs mutable access to both
        // the history resource and the world
        world.resource_scope(
            |world, mut history: Mut<CommandHistory>| match history.undo(world) {
                Ok(()) => {
                    if let Some(d) = desc {
                        log!(
                            LogType::Editor,
                            LogLevel::OK,
                            LogCategory::System,
                            "Undo: {}",
                            d
                        );
                    } else {
                        log!(
                            LogType::Editor,
                            LogLevel::OK,
                            LogCategory::System,
                            "Undo successful"
                        );
                    }
                }
                Err(e) => {
                    log!(
                        LogType::Editor,
                        LogLevel::Warning,
                        LogCategory::System,
                        "Undo failed: {}",
                        e
                    );
                }
            },
        );
    }

    // Process redos using resource_scope to properly handle borrows
    for _ in 0..redo_count {
        let can_redo = world
            .get_resource::<CommandHistory>()
            .map(|h| h.can_redo())
            .unwrap_or(false);

        if !can_redo {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::System,
                "Nothing to redo"
            );
            continue;
        }

        // Get the description before we consume the resource
        let desc = world
            .get_resource::<CommandHistory>()
            .and_then(|h| h.redo_description());

        // Use resource_scope to safely call redo which needs mutable access to both
        // the history resource and the world
        world.resource_scope(
            |world, mut history: Mut<CommandHistory>| match history.redo(world) {
                Ok(()) => {
                    if let Some(d) = desc {
                        log!(
                            LogType::Editor,
                            LogLevel::OK,
                            LogCategory::System,
                            "Redo: {}",
                            d
                        );
                    } else {
                        log!(
                            LogType::Editor,
                            LogLevel::OK,
                            LogCategory::System,
                            "Redo successful"
                        );
                    }
                }
                Err(e) => {
                    log!(
                        LogType::Editor,
                        LogLevel::Warning,
                        LogCategory::System,
                        "Redo failed: {}",
                        e
                    );
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_insertion() {
        let mut app = App::new();
        app.add_plugins(CommandHistoryPlugin);

        // Check that the resource was inserted
        assert!(app.world().contains_resource::<CommandHistory>());
        assert!(app.world().contains_resource::<UndoRedoQueue>());
    }

    #[test]
    fn test_undo_redo_queue() {
        let mut queue = UndoRedoQueue::new();

        queue.queue_undo();
        queue.queue_undo();
        queue.queue_redo();

        assert_eq!(queue.take_undo_count(), 2);
        assert_eq!(queue.take_redo_count(), 1);

        // After taking, counts should be reset
        assert_eq!(queue.take_undo_count(), 0);
        assert_eq!(queue.take_redo_count(), 0);
    }
}
