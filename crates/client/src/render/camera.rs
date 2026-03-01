// Camera system for following the player
//
// This module handles:
// - Camera following player entity
// - Smooth camera movement
// - World bounds checking

use bevy::prelude::*;
use trueworld_core::EntityId;

/// Marker component for the camera follow target
#[derive(Component, Debug, Clone, Copy)]
pub struct CameraFollowTarget {
    /// Entity ID of the entity to follow
    pub entity_id: EntityId,
    /// Smooth factor for camera movement (0.0 = instant, 1.0 = no movement)
    pub smooth_factor: f32,
    /// Minimum bounds for camera position
    pub min_bounds: Vec2,
    /// Maximum bounds for camera position
    pub max_bounds: Vec2,
    /// Offset from target position
    pub offset: Vec2,
}

impl Default for CameraFollowTarget {
    fn default() -> Self {
        Self {
            entity_id: EntityId::new(0),
            smooth_factor: 0.1,
            min_bounds: Vec2::new(-5000.0, -5000.0),
            max_bounds: Vec2::new(5000.0, 5000.0),
            offset: Vec2::ZERO,
        }
    }
}

impl CameraFollowTarget {
    /// Create a new camera follow target
    #[must_use]
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            ..Default::default()
        }
    }

    /// Set the smooth factor
    #[must_use]
    pub fn with_smooth_factor(mut self, factor: f32) -> Self {
        self.smooth_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Set the world bounds
    #[must_use]
    pub fn with_bounds(mut self, min: Vec2, max: Vec2) -> Self {
        self.min_bounds = min;
        self.max_bounds = max;
        self
    }

    /// Set the camera offset
    #[must_use]
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
}

/// Resource storing the current camera follow target entity
#[derive(Resource, Default)]
pub struct CurrentCameraTarget {
    /// The Bevy entity ID of the camera follow target
    pub entity: Option<Entity>,
}

/// Plugin for camera system
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentCameraTarget>()
           .add_systems(Startup, setup_camera)
           .add_systems(Update, (
               update_camera_follow,
               smooth_camera_movement,
           ).chain());
    }
}

/// Setup the 2D camera
fn setup_camera(mut commands: Commands) {
    // Spawn 2D camera - Camera2d automatically sets up orthographic projection
    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)),
        // Marker for camera follow system
        CameraFollowTarget::default(),
    ));

    info!("Camera system initialized");
}

/// Update camera follow target entity from network entity mapping
fn update_camera_follow(
    mut camera_target: ResMut<CurrentCameraTarget>,
    camera_query: Query<(Entity, &Camera), With<Camera2d>>,
) {
    // Find the camera and follow target
    if let Ok((camera_entity, _)) = camera_query.get_single() {
        // Update the current target if changed
        if camera_target.entity.is_none() || camera_target.entity != Some(camera_entity) {
            camera_target.entity = Some(camera_entity);
        }
    }
}

/// Smooth camera movement following the target
fn smooth_camera_movement(
    mut camera_query: Query<(&mut Transform, &CameraFollowTarget), With<Camera2d>>,
    target_query: Query<&Transform, Without<Camera>>,
) {
    for (mut camera_transform, follow_target) in camera_query.iter_mut() {
        // Find the target entity's transform
        let target_position = if let Ok(target_transform) = target_query.get_single() {
            target_transform.translation.truncate()
        } else {
            continue;
        };

        // Calculate desired camera position with offset
        let desired_position = target_position + follow_target.offset;

        // Smooth movement
        let current_position = camera_transform.translation.truncate();
        let new_position = current_position.lerp(
            desired_position,
            follow_target.smooth_factor,
        );

        // Apply bounds
        let clamped_position = Vec2::new(
            new_position.x.clamp(follow_target.min_bounds.x, follow_target.max_bounds.x),
            new_position.y.clamp(follow_target.min_bounds.y, follow_target.max_bounds.y),
        );

        // Update camera position (z stays the same)
        camera_transform.translation.x = clamped_position.x;
        camera_transform.translation.y = clamped_position.y;
    }
}

/// Set the camera follow target
pub fn set_camera_follow_target(
    _commands: &mut Commands,
    _current_target: &mut CurrentCameraTarget,
    _entity_id: EntityId,
    _smooth_factor: f32,
) {
    // Remove old target if exists
    // TODO: Implement camera follow target switching
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::animation::FacingDirection;

    #[test]
    fn test_camera_follow_target_default() {
        let target = CameraFollowTarget::default();
        assert_eq!(target.entity_id, EntityId::new(0));
        assert_eq!(target.smooth_factor, 0.1);
    }

    #[test]
    fn test_camera_follow_target_builder() {
        let entity_id = EntityId::new(42);
        let target = CameraFollowTarget::new(entity_id)
            .with_smooth_factor(0.5)
            .with_offset(Vec2::new(10.0, 20.0));

        assert_eq!(target.entity_id, entity_id);
        assert_eq!(target.smooth_factor, 0.5);
        assert_eq!(target.offset, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_facing_direction_from_velocity_diagonal() {
        // Diagonal movement should choose dominant axis
        let dir = FacingDirection::from_velocity(Vec2::new(1.0, 0.5));
        assert_eq!(dir, FacingDirection::Right);

        let dir = FacingDirection::from_velocity(Vec2::new(0.5, 1.0));
        assert_eq!(dir, FacingDirection::Up);
    }
}
