//! Integration tests for the undo/redo system
//!
//! These tests verify that the undo/redo system works correctly
//! with various command types and scenarios.

#[cfg(test)]
mod tests {
    use crate::history::{
        CommandHistory, CommandResult, EditorCommand, EntityCreateCommand, EntityDeleteCommand,
        TransformCommand,
    };
    use bevy::prelude::*;

    /// Test basic execute and undo flow
    #[test]
    fn test_execute_undo_flow() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        // Create an entity
        let name = Name::new("TestEntity");
        let transform = Transform::default();
        let create_cmd = EntityCreateCommand::new(name, transform);

        // Execute through history
        history.execute(Box::new(create_cmd), &mut world).unwrap();

        // Get the entity that was created
        let entity = world
            .query::<(Entity, &Name)>()
            .iter(&world)
            .find(|(_, n)| n.as_str() == "TestEntity")
            .map(|(e, _)| e)
            .unwrap();

        // Verify entity exists
        assert!(world.get_entity(entity).is_ok());

        // Undo
        history.undo(&mut world).unwrap();

        // Verify entity no longer exists
        assert!(world.get_entity(entity).is_err());
    }

    /// Test transform command in history
    #[test]
    fn test_transform_command_history() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        let entity = world.spawn(Transform::default()).id();

        let old_transform = Transform::default();
        let new_transform = Transform::from_xyz(5.0, 5.0, 5.0);

        let cmd = TransformCommand::new(entity, old_transform, new_transform);

        // Execute through history
        history.execute(Box::new(cmd), &mut world).unwrap();

        // Verify transform changed
        let current = world.get::<Transform>(entity).unwrap();
        assert_eq!(current.translation, Vec3::new(5.0, 5.0, 5.0));

        // Undo
        history.undo(&mut world).unwrap();

        // Verify transform reverted
        let current = world.get::<Transform>(entity).unwrap();
        assert_eq!(current.translation, Vec3::ZERO);

        // Redo
        history.redo(&mut world).unwrap();

        // Verify transform restored
        let current = world.get::<Transform>(entity).unwrap();
        assert_eq!(current.translation, Vec3::new(5.0, 5.0, 5.0));
    }

    /// Test multiple commands in sequence
    #[test]
    fn test_multiple_commands() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        // Command 1: Create entity
        let cmd1 = EntityCreateCommand::new(Name::new("Entity1"), Transform::default());
        history.execute(Box::new(cmd1), &mut world).unwrap();

        // Command 2: Create another entity
        let cmd2 = EntityCreateCommand::new(Name::new("Entity2"), Transform::default());
        history.execute(Box::new(cmd2), &mut world).unwrap();

        // Verify both exist
        assert_eq!(world.entities().len(), 2);

        // Undo once
        history.undo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 1);

        // Undo again
        history.undo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 0);

        // Redo once
        history.redo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 1);
    }

    /// Test that new command clears redo stack
    #[test]
    fn test_new_command_clears_redo() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        // Execute commands
        let cmd1 = EntityCreateCommand::new(Name::new("Entity1"), Transform::default());
        history.execute(Box::new(cmd1), &mut world).unwrap();

        let cmd2 = EntityCreateCommand::new(Name::new("Entity2"), Transform::default());
        history.execute(Box::new(cmd2), &mut world).unwrap();

        // Undo
        history.undo(&mut world).unwrap();
        assert!(history.can_redo());

        // Execute new command
        let cmd3 = EntityCreateCommand::new(Name::new("Entity3"), Transform::default());
        history.execute(Box::new(cmd3), &mut world).unwrap();

        // Redo should no longer be available
        assert!(!history.can_redo());
    }

    /// Test descriptions
    #[test]
    fn test_descriptions() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        let cmd = EntityCreateCommand::new(Name::new("MyEntity"), Transform::default());
        history.execute(Box::new(cmd), &mut world).unwrap();

        let undo_desc = history.undo_description();
        assert!(undo_desc.is_some());
        assert!(undo_desc.unwrap().contains("MyEntity"));

        assert!(history.redo_description().is_none());

        history.undo(&mut world).unwrap();

        let redo_desc = history.redo_description();
        assert!(redo_desc.is_some());
        assert!(redo_desc.unwrap().contains("MyEntity"));
    }

    /// Test can_undo and can_redo
    #[test]
    fn test_can_undo_redo() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let cmd = EntityCreateCommand::new(Name::new("Entity"), Transform::default());
        history.execute(Box::new(cmd), &mut world).unwrap();

        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut world).unwrap();

        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo(&mut world).unwrap();

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    /// Test max size enforcement
    #[test]
    fn test_max_size_enforcement() {
        let mut world = World::new();
        let mut history = CommandHistory::with_capacity(3);

        // Add 5 commands
        for i in 0..5 {
            let cmd =
                EntityCreateCommand::new(Name::new(format!("Entity{}", i)), Transform::default());
            history.execute(Box::new(cmd), &mut world).unwrap();
        }

        // Should only have 3 commands
        assert_eq!(history.undo_count(), 3);
    }

    /// Test clear
    #[test]
    fn test_clear() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        for i in 0..3 {
            let cmd =
                EntityCreateCommand::new(Name::new(format!("Entity{}", i)), Transform::default());
            history.execute(Box::new(cmd), &mut world).unwrap();
        }

        history.undo(&mut world).unwrap();
        assert!(history.can_undo());
        assert!(history.can_redo());

        history.clear();

        assert!(!history.can_undo());
        assert_eq!(history.undo_count(), 0);
    }

    /// Test transaction begin/end
    #[test]
    fn test_transaction_state() {
        let mut history = CommandHistory::new();

        assert!(!history.in_transaction());

        history.begin_transaction("MyTransaction");
        assert!(history.in_transaction());

        history.end_transaction();
        assert!(!history.in_transaction());
    }

    /// Test undo_count and redo_count
    #[test]
    fn test_count_methods() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);

        for i in 0..3 {
            let cmd =
                EntityCreateCommand::new(Name::new(format!("Entity{}", i)), Transform::default());
            history.execute(Box::new(cmd), &mut world).unwrap();
        }

        assert_eq!(history.undo_count(), 3);
        assert_eq!(history.redo_count(), 0);

        history.undo(&mut world).unwrap();
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 1);

        history.undo(&mut world).unwrap();
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 2);
    }

    /// Test transform with position, rotation, and scale changes
    #[test]
    fn test_transform_all_components() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        let entity = world.spawn(Transform::default()).id();

        let old_transform = Transform::default();
        let mut new_transform = Transform::default();
        new_transform.translation = Vec3::new(1.0, 2.0, 3.0);
        new_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI / 4.0);
        new_transform.scale = Vec3::new(2.0, 2.0, 2.0);

        let cmd = TransformCommand::new(entity, old_transform, new_transform);

        history.execute(Box::new(cmd), &mut world).unwrap();

        let current = world.get::<Transform>(entity).unwrap();
        assert_eq!(current.translation, Vec3::new(1.0, 2.0, 3.0));
        assert!(current.rotation.dot(new_transform.rotation).abs() > 0.99);
        assert_eq!(current.scale, Vec3::new(2.0, 2.0, 2.0));

        history.undo(&mut world).unwrap();

        let current = world.get::<Transform>(entity).unwrap();
        assert_eq!(current.translation, Vec3::ZERO);
        assert_eq!(current.scale, Vec3::ONE);
    }

    /// Test entity deletion and undeletion
    #[test]
    fn test_entity_deletion_cycle() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        // Create entity
        let entity = world
            .spawn((Name::new("TestEntity"), Transform::default()))
            .id();
        assert!(world.get_entity(entity).is_ok());

        // Delete it
        let delete_cmd =
            EntityDeleteCommand::new(entity, Name::new("TestEntity"), Transform::default());
        history.execute(Box::new(delete_cmd), &mut world).unwrap();
        assert!(world.get_entity(entity).is_err());

        // Undo deletion
        history.undo(&mut world).unwrap();
        assert!(world.entities().len() > 0);

        // Redo deletion
        history.redo(&mut world).unwrap();
    }

    /// Test error handling - undo with empty history
    #[test]
    fn test_undo_empty_history() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        let result = history.undo(&mut world);
        assert!(result.is_err());
    }

    /// Test error handling - redo with empty redo stack
    #[test]
    fn test_redo_empty_history() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        let result = history.redo(&mut world);
        assert!(result.is_err());
    }

    /// Test set_max_size
    #[test]
    fn test_set_max_size() {
        let mut history = CommandHistory::new();
        assert_eq!(history.max_size(), 100); // Default

        history.set_max_size(50);
        assert_eq!(history.max_size(), 50);
    }

    /// Test complex workflow: multiple operations
    #[test]
    fn test_complex_workflow() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        // Step 1: Create entity
        let create_cmd = EntityCreateCommand::new(Name::new("Player"), Transform::default());
        history.execute(Box::new(create_cmd), &mut world).unwrap();
        assert_eq!(world.entities().len(), 1);

        // Step 2: Create second entity
        let create_cmd2 = EntityCreateCommand::new(Name::new("Enemy"), Transform::default());
        history.execute(Box::new(create_cmd2), &mut world).unwrap();

        // Verify state - 2 entities exist
        assert_eq!(world.entities().len(), 2);

        // Undo creation of enemy
        history.undo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 1);

        // Undo creation of player
        history.undo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 0);

        // Redo all - redo player creation
        history.redo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 1);

        // Redo enemy creation
        history.redo(&mut world).unwrap();
        assert_eq!(world.entities().len(), 2);

        // Verify both entities exist by name
        {
            let mut query = world.query::<&Name>();
            let names: Vec<_> = query.iter(&world).map(|n| n.as_str()).collect();
            assert!(names.contains(&"Player"));
            assert!(names.contains(&"Enemy"));
        }
    }
}
