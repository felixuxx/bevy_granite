use crate::history::command::{CommandError, CommandResult, EditorCommand};
use bevy::prelude::*;

/// Command for undoing/redoing entity transform changes
///
/// This command stores the old and new transform values and can
/// restore an entity to either state.
#[derive(Clone)]
pub struct TransformCommand {
    entity: Entity,
    old_transform: Transform,
    new_transform: Transform,
    description: String,
}

impl TransformCommand {
    /// Create a new transform command
    ///
    /// # Arguments
    /// * `entity` - The entity being transformed
    /// * `old` - The transform before the change
    /// * `new` - The transform after the change
    pub fn new(entity: Entity, old: Transform, new: Transform) -> Self {
        let description = format!("Transform {}", Self::format_transform_change(&old, &new));

        Self {
            entity,
            old_transform: old,
            new_transform: new,
            description,
        }
    }

    /// Helper to format a readable description of what changed
    fn format_transform_change(old: &Transform, new: &Transform) -> String {
        let mut changes = Vec::new();

        // Check if translation changed
        if (old.translation - new.translation).length_squared() > 0.001 {
            changes.push("position");
        }

        // Check if rotation changed
        if old.rotation.dot(new.rotation).abs() < 0.9999 {
            changes.push("rotation");
        }

        // Check if scale changed
        if (old.scale - new.scale).length_squared() > 0.001 {
            changes.push("scale");
        }

        if changes.is_empty() {
            "entity".to_string()
        } else {
            changes.join(", ")
        }
    }

    /// Get the entity this command affects
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Get the old transform
    pub fn old_transform(&self) -> &Transform {
        &self.old_transform
    }

    /// Get the new transform
    pub fn new_transform(&self) -> &Transform {
        &self.new_transform
    }
}

impl EditorCommand for TransformCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        if let Ok(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut transform) = entity.get_mut::<Transform>() {
                *transform = self.new_transform;
                Ok(())
            } else {
                Err(CommandError::ComponentNotFound("Transform".to_string()))
            }
        } else {
            Err(CommandError::EntityNotFound(self.entity))
        }
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if let Ok(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut transform) = entity.get_mut::<Transform>() {
                *transform = self.old_transform;
                Ok(())
            } else {
                Err(CommandError::ComponentNotFound("Transform".to_string()))
            }
        } else {
            Err(CommandError::EntityNotFound(self.entity))
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn clone_command(&self) -> Box<dyn EditorCommand> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_command_execute() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let old_transform = Transform::default();
        let new_transform = Transform::from_xyz(1.0, 2.0, 3.0);

        let mut cmd = TransformCommand::new(entity, old_transform, new_transform);

        cmd.execute(&mut world).unwrap();

        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_transform_command_undo() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let old_transform = Transform::default();
        let new_transform = Transform::from_xyz(1.0, 2.0, 3.0);

        let mut cmd = TransformCommand::new(entity, old_transform, new_transform);

        cmd.execute(&mut world).unwrap();
        cmd.undo(&mut world).unwrap();

        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.translation, Vec3::ZERO);
    }

    #[test]
    fn test_transform_command_missing_entity() {
        let mut world = World::new();
        let entity = Entity::from_raw_u32(999).unwrap();

        let old_transform = Transform::default();
        let new_transform = Transform::from_xyz(1.0, 2.0, 3.0);

        let mut cmd = TransformCommand::new(entity, old_transform, new_transform);
        let result = cmd.execute(&mut world);

        assert!(result.is_err());
        if let Err(CommandError::EntityNotFound(_)) = result {
            // Expected
        } else {
            panic!("Expected EntityNotFound error");
        }
    }

    #[test]
    fn test_transform_description() {
        let old = Transform::default();
        let new = Transform::from_xyz(1.0, 2.0, 3.0);

        let cmd = TransformCommand::new(Entity::from_raw_u32(0).unwrap(), old, new);
        let desc = cmd.description();

        assert!(desc.contains("position"));
    }
}
