// Network entity synchronization system
//
// This module handles:
// - Mapping server entity IDs to client Bevy entities
// - Spawning entities from network updates
// - Updating existing entities from server data
// - Despawning removed entities

use bevy::prelude::*;
use trueworld_core::{
    EntityId, EntityType, TransformState, EntityUpdate,
    WorldUpdateMessage,
};

use super::{ProceduralSprites, spawn_entity_sprite};
use super::animation::{AnimationController, AnimationState};

/// Marker component for network-synchronized entities
#[derive(Component)]
pub struct NetworkEntityMarker;

/// Component linking client entity to server entity ID
#[derive(Component, Debug, Clone, Copy)]
pub struct NetworkEntity(pub EntityId);

/// Resource for mapping server entity IDs to client entities
#[derive(Resource, Default, Debug)]
pub struct EntityMapping {
    /// Map from server EntityId to client Bevy Entity
    server_to_client: std::collections::HashMap<EntityId, Entity>,
    /// Map from client Bevy Entity to server EntityId
    client_to_server: std::collections::HashMap<Entity, EntityId>,
}

impl EntityMapping {
    /// Insert a new mapping
    pub fn insert(&mut self, server_id: EntityId, client_entity: Entity) {
        self.server_to_client.insert(server_id, client_entity);
        self.client_to_server.insert(client_entity, server_id);
    }

    /// Get client entity from server ID
    #[must_use]
    pub fn get_client(&self, server_id: EntityId) -> Option<Entity> {
        self.server_to_client.get(&server_id).copied()
    }

    /// Get server ID from client entity
    #[must_use]
    pub fn get_server(&self, client_entity: Entity) -> Option<EntityId> {
        self.client_to_server.get(&client_entity).copied()
    }

    /// Remove a mapping
    pub fn remove(&mut self, server_id: EntityId) -> Option<Entity> {
        if let Some(client_entity) = self.server_to_client.remove(&server_id) {
            self.client_to_server.remove(&client_entity);
            Some(client_entity)
        } else {
            None
        }
    }

    /// Check if server ID exists
    #[must_use]
    pub fn contains_server(&self, server_id: EntityId) -> bool {
        self.server_to_client.contains_key(&server_id)
    }

    /// Get all server IDs
    #[must_use]
    pub fn server_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.server_to_client.keys().copied()
    }
}

/// Resource storing pending world updates to process
#[derive(Resource, Default)]
pub struct PendingWorldUpdate {
    /// Pending update message
    pub update: Option<WorldUpdateMessage>,
}

/// Plugin for network entity synchronization
pub struct EntitySyncPlugin;

impl Plugin for EntitySyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityMapping>()
           .init_resource::<PendingWorldUpdate>()
           .add_systems(Update, (
               sync_entities_from_network,
               remove_despawned_entities,
               update_entity_transforms,
               update_entity_animations,
           ));
    }
}

/// Sync entities from pending world update
pub fn sync_entities_from_network(
    mut commands: Commands,
    mut mapping: ResMut<EntityMapping>,
    mut pending_update: ResMut<PendingWorldUpdate>,
    sprites: Res<ProceduralSprites>,
    mut entity_query: Query<(
        Entity,
        &mut Transform,
        &mut AnimationController,
    ), With<NetworkEntityMarker>>,
) {
    let Some(update) = pending_update.update.take() else {
        return;
    };

    // Process each entity update
    for entity_update in update.entities {
        let server_id = entity_update.entity_id;

        if let Some(client_entity) = mapping.get_client(server_id) {
            // Update existing entity
            if let Ok((_, mut transform, mut animation)) = entity_query.get_mut(client_entity) {
                // Update position (convert 3D position to 2D)
                let pos = &entity_update.transform.position;
                transform.translation.x = pos[0];
                transform.translation.y = pos[1];

                // Update animation state based on velocity
                let velocity = Vec2::new(entity_update.velocity[0], entity_update.velocity[1]);
                if velocity.length_squared() > 0.01 {
                    animation.update_facing_from_velocity(velocity);
                    animation.change_state(AnimationState::Walking);
                } else {
                    animation.change_state(AnimationState::Idle);
                }
            }
        } else {
            // Spawn new entity
            let pos = &entity_update.transform.position;
            let position = Vec2::new(pos[0], pos[1]);

            let new_entity = spawn_entity_sprite(
                &mut commands,
                server_id,
                entity_update.entity_type,
                position,
                &sprites,
                None, // TODO: extract name from entity data
            );

            mapping.insert(server_id, new_entity);
        }
    }
}

/// Remove entities that were despawned on server
pub fn remove_despawned_entities(
    mut commands: Commands,
    mut mapping: ResMut<EntityMapping>,
    mut pending_update: ResMut<PendingWorldUpdate>,
) {
    let Some(update) = pending_update.update.as_ref() else {
        return;
    };

    for server_id in &update.removed_entities {
        if let Some(client_entity) = mapping.remove(*server_id) {
            commands.entity(client_entity).despawn_recursive();
            info!("Despawned entity {:?}", server_id);
        }
    }
}

/// Update entity transforms (interpolation could be added here)
fn update_entity_transforms(
    _query: Query<&mut Transform, With<NetworkEntityMarker>>,
    _time: Res<Time>,
) {
    // Basic transform update
    // TODO: Add interpolation for smoother movement
}

/// Update entity animations based on state
fn update_entity_animations(
    mut query: Query<&mut AnimationController, With<NetworkEntityMarker>>,
    time: Res<Time>,
) {
    for mut animation in query.iter_mut() {
        animation.tick(time.delta());
    }
}

/// Queue a world update for processing
pub fn queue_world_update(pending_update: &mut PendingWorldUpdate, update: WorldUpdateMessage) {
    pending_update.update = Some(update);
}

/// Find network entity by server ID
#[must_use]
pub fn find_network_entity(
    mapping: &EntityMapping,
    server_id: EntityId,
) -> Option<Entity> {
    mapping.get_client(server_id)
}

/// Get local player entity (first player entity found)
#[must_use]
pub fn get_local_player_entity(
    mapping: &EntityMapping,
    local_player_id: EntityId,
) -> Option<Entity> {
    mapping.get_client(local_player_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_mapping_insert() {
        let mut mapping = EntityMapping::default();
        let server_id = EntityId::new(1);
        let client_entity = Entity::from_raw(100);

        mapping.insert(server_id, client_entity);

        assert_eq!(mapping.get_client(server_id), Some(client_entity));
        assert_eq!(mapping.get_server(client_entity), Some(server_id));
    }

    #[test]
    fn test_entity_mapping_remove() {
        let mut mapping = EntityMapping::default();
        let server_id = EntityId::new(1);
        let client_entity = Entity::from_raw(100);

        mapping.insert(server_id, client_entity);
        let removed = mapping.remove(server_id);

        assert_eq!(removed, Some(client_entity));
        assert!(!mapping.contains_server(server_id));
    }

    #[test]
    fn test_entity_mapping_contains() {
        let mut mapping = EntityMapping::default();
        let server_id = EntityId::new(1);
        let client_entity = Entity::from_raw(100);

        assert!(!mapping.contains_server(server_id));

        mapping.insert(server_id, client_entity);
        assert!(mapping.contains_server(server_id));
    }
}
