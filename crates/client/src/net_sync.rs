// crates/client/src/net_sync.rs
//
// Network to Render synchronization system.
// Connects network events to the rendering system, spawning and updating entities.

#![allow(dead_code)]

use bevy::prelude::*;
use trueworld_core::net::EntityType;

use crate::network::{
    WorldUpdateEvent, EntityDespawnEvent, PlayerSpawnEvent,
};
use crate::render::{
    spawn_entity_sprite, ProceduralSprites, NetworkEntity,
};

/// System to process world updates from network
pub fn handle_world_updates(
    mut commands: Commands,
    mut world_update_events: EventReader<WorldUpdateEvent>,
    procedural_sprites: Res<ProceduralSprites>,
    query: Query<(Entity, &NetworkEntity)>,
) {
    for event in world_update_events.read() {
        for update in &event.updates {
            // Check if this entity already exists
            let mut found = false;
            for (entity, network_entity) in query.iter() {
                if network_entity.0 == update.entity_id {
                    // Update existing entity position
                    let translation = Vec3::new(
                        update.position[0],
                        update.position[1],
                        update.position[2],
                    );
                    commands.entity(entity).insert(Transform::from_translation(translation));
                    found = true;
                    break;
                }
            }

            if !found {
                // New entity, spawn it
                let position = Vec2::new(update.position[0], update.position[1]);

                spawn_entity_sprite(
                    &mut commands,
                    update.entity_id,
                    EntityType::Player,
                    position,
                    &procedural_sprites,
                    None::<String>,
                );
            }
        }
    }
}

/// System to handle entity despawn events
pub fn handle_entity_despawns(
    mut commands: Commands,
    mut despawn_events: EventReader<EntityDespawnEvent>,
    query: Query<(Entity, &NetworkEntity)>,
) {
    for event in despawn_events.read() {
        for (entity, network_entity) in query.iter() {
            if network_entity.0 == event.entity_id {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// System to handle player spawn events
pub fn handle_player_spawns(
    mut commands: Commands,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
    procedural_sprites: Res<ProceduralSprites>,
) {
    for event in spawn_events.read() {
        info!("Spawning player: {} at {:?}", event.username, event.position);

        let position = Vec2::new(event.position[0], event.position[1]);

        spawn_entity_sprite(
            &mut commands,
            event.entity_id,
            EntityType::Player,
            position,
            &procedural_sprites,
            Some(event.username.clone()),
        );
    }
}

/// Plugin for network-render synchronization
pub struct NetSyncPlugin;

impl Plugin for NetSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_world_updates,
            handle_entity_despawns,
            handle_player_spawns,
        ).chain());
    }
}
