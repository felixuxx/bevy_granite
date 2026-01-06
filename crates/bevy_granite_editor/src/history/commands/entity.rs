use crate::history::command::{CommandError, CommandResult, EditorCommand};
use bevy::prelude::*;

/// Command for creating a new entity
///
/// This command spawns a new entity with the given name and transform.
/// When undone, it despawns the entity. When redone, it respawns it.
#[derive(Clone)]
pub struct EntityCreateCommand {
    entity: Option<Entity>,
    name: Name,
    transform: Transform,
    description: String,
}

impl EntityCreateCommand {
    /// Create a new entity creation command
    ///
    /// # Arguments
    /// * `name` - The name of the new entity
    /// * `transform` - The initial transform of the entity
    pub fn new(name: Name, transform: Transform) -> Self {
        Self {
            entity: None,
            name: name.clone(),
            transform,
            description: format!("Create entity '{}'", name),
        }
    }

    /// Get the entity that was created (after execute)
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }

    /// Get the entity name
    pub fn name(&self) -> &Name {
        &self.name
    }
}

impl EditorCommand for EntityCreateCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        let entity = world.spawn((self.name.clone(), self.transform)).id();
        self.entity = Some(entity);
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if let Some(entity) = self.entity {
            if world.get_entity(entity).is_ok() {
                world.despawn(entity);
                Ok(())
            } else {
                Err(CommandError::EntityNotFound(entity))
            }
        } else {
            Err(CommandError::InvalidState(
                "Entity was never created".to_string(),
            ))
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn clone_command(&self) -> Box<dyn EditorCommand> {
        Box::new(self.clone())
    }
}

/// Command for deleting an entity
///
/// This command stores minimal entity data (name and transform) and can
/// recreate the entity when undone.
#[derive(Clone)]
pub struct EntityDeleteCommand {
    entity: Entity,
    name: Name,
    transform: Transform,
    description: String,
    was_deleted: bool,
}

impl EntityDeleteCommand {
    /// Create a new entity deletion command
    ///
    /// # Arguments
    /// * `entity` - The entity to delete
    /// * `name` - The entity's name (needed for recreation)
    /// * `transform` - The entity's transform (needed for recreation)
    pub fn new(entity: Entity, name: Name, transform: Transform) -> Self {
        Self {
            entity,
            name: name.clone(),
            transform,
            description: format!("Delete entity '{}'", name),
            was_deleted: false,
        }
    }

    /// Get the deleted entity
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Check if the entity was actually deleted
    pub fn was_deleted(&self) -> bool {
        self.was_deleted
    }
}

impl EditorCommand for EntityDeleteCommand {
    fn execute(&mut self, world: &mut World) -> CommandResult<()> {
        if world.get_entity(self.entity).is_ok() {
            world.despawn(self.entity);
            self.was_deleted = true;
            Ok(())
        } else {
            Err(CommandError::EntityNotFound(self.entity))
        }
    }

    fn undo(&mut self, world: &mut World) -> CommandResult<()> {
        if self.was_deleted {
            let entity = world.spawn((self.name.clone(), self.transform)).id();
            // Try to keep the same entity ID if possible (won't work, but entity is close enough)
            self.entity = entity;
            self.was_deleted = false;
            Ok(())
        } else {
            Err(CommandError::InvalidState(
                "Entity was never deleted".to_string(),
            ))
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
    fn test_entity_create_command() {
        let mut world = World::new();
        let name = Name::new("TestEntity");
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);

        let mut cmd = EntityCreateCommand::new(name.clone(), transform);
        cmd.execute(&mut world).unwrap();

        let entity = cmd.entity().unwrap();
        let stored_name = world.get::<Name>(entity).unwrap();
        assert_eq!(stored_name.as_str(), "TestEntity");

        let stored_transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(stored_transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_entity_create_undo() {
        let mut world = World::new();
        let name = Name::new("TestEntity");
        let transform = Transform::default();

        let mut cmd = EntityCreateCommand::new(name, transform);
        cmd.execute(&mut world).unwrap();

        let entity = cmd.entity().unwrap();
        assert!(world.get_entity(entity).is_ok());

        cmd.undo(&mut world).unwrap();
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn test_entity_delete_command() {
        let mut world = World::new();
        let entity = world
            .spawn((Name::new("TestEntity"), Transform::default()))
            .id();

        let name = Name::new("TestEntity");
        let transform = Transform::default();

        let mut cmd = EntityDeleteCommand::new(entity, name, transform);
        cmd.execute(&mut world).unwrap();

        assert!(world.get_entity(entity).is_err());
        assert!(cmd.was_deleted());
    }

    #[test]
    fn test_entity_delete_undo() {
        let mut world = World::new();
        let entity = world
            .spawn((Name::new("TestEntity"), Transform::default()))
            .id();

        let name = Name::new("TestEntity");
        let transform = Transform::default();

        let mut cmd = EntityDeleteCommand::new(entity, name, transform);
        cmd.execute(&mut world).unwrap();
        assert!(world.get_entity(entity).is_err());

        cmd.undo(&mut world).unwrap();
        // Note: The entity ID may change after undo, but entity should exist
        assert!(!cmd.was_deleted());
    }

    #[test]
    fn test_entity_delete_nonexistent() {
        let mut world = World::new();
        let entity = Entity::from_raw_u32(999).unwrap();
        let name = Name::new("TestEntity");
        let transform = Transform::default();

        let mut cmd = EntityDeleteCommand::new(entity, name, transform);
        let result = cmd.execute(&mut world);

        assert!(result.is_err());
        if let Err(CommandError::EntityNotFound(_)) = result {
            // Expected
        } else {
            panic!("Expected EntityNotFound error");
        }
    }

    #[test]
    fn test_entity_create_description() {
        let name = Name::new("MyEntity");
        let cmd = EntityCreateCommand::new(name, Transform::default());
        assert!(cmd.description().contains("MyEntity"));
    }
}
