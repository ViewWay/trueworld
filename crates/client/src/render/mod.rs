// Client render module for TrueWorld
//
// This module handles all client-side rendering including:
// - Procedural sprite generation
// - Entity sprite spawning and management
// - Animation system
// - Camera following player

pub mod sprite;
pub mod animation;
pub mod camera;
pub mod sync;

use bevy::prelude::*;

// Private imports for internal use
use sprite::SpritePlugin;
use animation::AnimationPlugin;
use camera::CameraPlugin;
use sync::EntitySyncPlugin;

// Re-export from submodules for public API
pub use sync::NetworkEntity;
pub use sprite::ProceduralSprites;
pub use animation::{AnimationState, FacingDirection};

/// Main render plugin that combines all rendering subsystems
pub struct EntityRenderPlugin;

impl Plugin for EntityRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SpritePlugin,
            AnimationPlugin,
            CameraPlugin,
            EntitySyncPlugin,
        ));
    }
}

// Re-export commonly used types
pub use sprite::{spawn_entity_sprite, generate_entity_sprite};
pub use camera::CameraFollowTarget;
pub use sync::{sync_entities_from_network, remove_despawned_entities};
