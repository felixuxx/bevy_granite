use super::GizmoType;
use bevy::prelude::{Entity, Message, Transform};
use bevy_granite_core::TransformData;

#[derive(Message)]
pub struct RotateInitDragEvent;

#[derive(Message)]
pub struct RotateDraggingEvent;

#[derive(Message)]
pub struct RotateResetDragEvent;

#[derive(Message)]
pub struct TransformInitDragEvent;

#[derive(Message)]
pub struct TransformDraggingEvent;

#[derive(Message)]
pub struct TransformResetDragEvent;

#[derive(Message)]
pub struct SpawnGizmoEvent(pub Entity);

#[derive(Message)]
pub struct DespawnGizmoEvent(pub GizmoType);

/// Event emitted when a gizmo applies a transform to an entity
/// This notifies the editor so it can record the change in undo/redo history
#[derive(Message, Clone)]
pub struct GizmoTransformAppliedEvent {
    pub entity: Entity,
    pub old_transform: TransformData,
    pub new_transform: TransformData,
}
