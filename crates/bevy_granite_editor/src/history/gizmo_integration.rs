//! Integration between gizmo transform changes and the undo/redo system
//!
//! This module provides systems that track transform changes made via the gizmo
//! and automatically create TransformCommand entries in the undo/redo history.
//!
//! # How it works
//!
//! When a user updates a transform (via gizmo or inspector), a UserUpdatedTransformEvent
//! is fired. This system listens to those events and queues TransformCommand entries
//! that can be undone/redone.

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;
use bevy_granite_gizmos::GizmoTransformAppliedEvent;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::interface::events::UserUpdatedTransformEvent;
use crate::interface::tabs::entity_editor::widgets::EntityGlobalTransformData;

use super::{CommandHistory, TransformCommand};

/// Resource that queues pending transform commands to be processed by the exclusive system
#[derive(Resource, Default)]
pub struct PendingTransformCommands {
    pub commands: Vec<(Entity, Transform, Transform)>, // (entity, old_transform, new_transform)
}

impl PendingTransformCommands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_transform(
        &mut self,
        entity: Entity,
        old_transform: Transform,
        new_transform: Transform,
    ) {
        self.commands.push((entity, old_transform, new_transform));
    }

    pub fn take_all(&mut self) -> Vec<(Entity, Transform, Transform)> {
        std::mem::take(&mut self.commands)
    }
}

/// System that converts GizmoTransformAppliedEvent (from gizmo system) to UserUpdatedTransformEvent
/// This bridges the gizmos crate with the editor's transform event system
pub fn convert_gizmo_transform_events(
    mut gizmo_events: MessageReader<GizmoTransformAppliedEvent>,
    mut transform_event_writer: MessageWriter<UserUpdatedTransformEvent>,
) {
    for event in gizmo_events.read() {
        // Convert the gizmo event to a UserUpdatedTransformEvent
        let user_event = UserUpdatedTransformEvent {
            entity: event.entity,
            data: EntityGlobalTransformData {
                global_transform_data: event.new_transform.clone(),
                transform_data_changed: true,
                gizmo_axis: None,
                editing_rotation: [false; 3],
                euler_degrees: Vec3::ZERO,
                euler_radians: Vec3::ZERO,
                last_synced_quat: event.new_transform.rotation,
            },
        };

        transform_event_writer.write(user_event);
    }
}

/// System that records transform changes from UserUpdatedTransformEvent
/// Queues TransformCommand entries for later processing
pub fn record_user_transform_changes(
    mut reader: MessageReader<UserUpdatedTransformEvent>,
    transforms: Query<&Transform>,
    mut queue: ResMut<PendingTransformCommands>,
) {
    for event in reader.read() {
        let entity = event.entity;

        // Get the current transform
        let Ok(current_transform) = transforms.get(entity) else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::System,
                "Could not find transform for entity {:?} in UserUpdatedTransformEvent",
                entity
            );
            continue;
        };

        let current = *current_transform;

        // Extract the old transform from the event data
        let old_transform = Transform {
            translation: event.data.global_transform_data.position,
            rotation: event.data.global_transform_data.rotation,
            scale: event.data.global_transform_data.scale,
        };

        // Only queue if the transform actually changed
        if old_transform != current {
            queue.queue_transform(entity, old_transform, current);
        }
    }
}

/// Exclusive system that processes queued transform commands and adds them to history
pub fn process_pending_transform_commands(world: &mut World) {
    // Take all pending commands
    let commands = {
        let mut queue = world
            .get_resource_mut::<PendingTransformCommands>()
            .unwrap();
        queue.take_all()
    };

    // Process each command
    for (entity, old_transform, new_transform) in commands {
        let command = TransformCommand::new(entity, old_transform, new_transform);

        world.resource_scope(
            |world, mut history: bevy::ecs::change_detection::Mut<CommandHistory>| match history
                .execute(Box::new(command), world)
            {
                Ok(()) => {
                    log!(
                        LogType::Editor,
                        LogLevel::OK,
                        LogCategory::System,
                        "Recorded transform change for entity {:?}",
                        entity
                    );
                }
                Err(e) => {
                    log!(
                        LogType::Editor,
                        LogLevel::Warning,
                        LogCategory::System,
                        "Failed to record transform change: {}",
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
    fn test_pending_transform_commands_creation() {
        let queue = PendingTransformCommands::new();
        assert!(queue.commands.is_empty());
    }

    #[test]
    fn test_pending_transform_commands_queue() {
        let mut queue = PendingTransformCommands::new();
        let entity = Entity::from_raw_u32(1).unwrap();
        let old_transform = Transform::default();
        let new_transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));

        queue.queue_transform(entity, old_transform, new_transform);
        assert_eq!(queue.commands.len(), 1);
    }

    #[test]
    fn test_pending_transform_commands_take_all() {
        let mut queue = PendingTransformCommands::new();
        let entity = Entity::from_raw_u32(1).unwrap();
        let old_transform = Transform::default();
        let new_transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));

        queue.queue_transform(entity, old_transform, new_transform);
        let commands = queue.take_all();
        assert_eq!(commands.len(), 1);
        assert!(queue.commands.is_empty());
    }
}
