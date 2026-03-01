// Game world module for TrueWorld server
//
// This module provides the GameWorld which manages all game state
// including entities, players, and physics simulation.

use std::collections::HashMap;
use std::time::Duration;
use trueworld_core::{PlayerId, PlayerInput, net::{WorldUpdateMessage, ServerMessage, EntityUpdate}};
use super::entity::{Entity, EntityManager};

// ============================================================================
// GameWorld
// ============================================================================

/// The game world manages all game state including entities and players
pub struct GameWorld {
    /// Entity manager for all game entities
    entities: EntityManager,

    /// Player inputs indexed by player ID
    player_inputs: HashMap<PlayerId, PlayerInput>,

    /// Current server tick
    current_tick: u64,
}

impl GameWorld {
    /// Creates a new game world
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            player_inputs: HashMap::new(),
            current_tick: 0,
        }
    }

    /// Returns a reference to the entity manager
    #[must_use]
    pub const fn entities(&self) -> &EntityManager {
        &self.entities
    }

    /// Returns a mutable reference to the entity manager
    pub fn entities_mut(&mut self) -> &mut EntityManager {
        &mut self.entities
    }

    /// Returns the current tick number
    #[must_use]
    pub const fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Spawns a player entity
    pub fn spawn_player(&mut self, player_id: PlayerId, name: impl Into<String>, position: [f32; 3]) -> trueworld_core::EntityId {
        self.entities.spawn_player(player_id, name, position)
    }

    /// Spawns a monster entity
    pub fn spawn_monster(&mut self, monster_type: u32, position: [f32; 3]) -> trueworld_core::EntityId {
        self.entities.spawn_monster(monster_type, position)
    }

    /// Despawns an entity
    pub fn despawn_entity(&mut self, entity_id: trueworld_core::EntityId) -> Option<Entity> {
        self.entities.despawn(entity_id)
    }

    /// Sets the input for a player
    pub fn set_player_input(&mut self, player_id: PlayerId, input: PlayerInput) {
        // Store the input for later processing
        self.player_inputs.insert(player_id, input.clone());

        // Also update the player entity's movement if it exists
        if let Some(entity) = self.entities.find_by_player_id_mut(player_id) {
            // Apply movement from input to entity velocity
            // This is a simple implementation - can be enhanced with proper physics
            let mut velocity = [0.0f32; 3];

            for action in &input.actions {
                match action {
                    trueworld_core::InputAction::MoveForward => velocity[2] = 5.0,
                    trueworld_core::InputAction::MoveBackward => velocity[2] = -5.0,
                    trueworld_core::InputAction::MoveLeft => velocity[0] = -5.0,
                    trueworld_core::InputAction::MoveRight => velocity[0] = 5.0,
                    trueworld_core::InputAction::Jump => velocity[1] = 10.0,
                    _ => {}
                }
            }

            entity.set_velocity(velocity);
        }
    }

    /// Gets the input for a player
    #[must_use]
    pub fn get_player_input(&self, player_id: PlayerId) -> Option<&PlayerInput> {
        self.player_inputs.get(&player_id)
    }

    /// Updates the game world by one tick
    pub fn update(&mut self, delta_time: Duration) {
        self.current_tick = self.current_tick.wrapping_add(1);

        // Update all entities (position, velocity, etc.)
        self.entities.update(delta_time);

        // Process player inputs and apply to entities
        self.process_player_inputs(delta_time);
    }

    /// Process player inputs and update entities
    fn process_player_inputs(&mut self, _delta_time: Duration) {
        // Get all player entities and apply their inputs
        for (&player_id, input) in &self.player_inputs {
            if let Some(entity) = self.entities.find_by_player_id_mut(player_id) {
                // Update rotation based on view direction
                entity.transform.rotation = input.view_direction;

                // Movement is already applied in set_player_input via velocity
                // Additional input processing can be added here
            }
        }
    }

    /// Creates a world update packet for network synchronization
    pub fn create_update_packet(&self) -> ServerMessage {
        let mut update = WorldUpdateMessage::new(self.current_tick, 0);

        // Add all active entities to the update
        for entity in self.entities.all() {
            if entity.is_active() {
                let entity_update = EntityUpdate {
                    entity_id: entity.id,
                    entity_type: entity.entity_type,
                    transform: entity.transform,
                    velocity: entity.velocity,
                    sequence: entity.sequence,
                    data: entity.data.clone(),
                };
                update.add_entity(entity_update);
            }
        }

        ServerMessage::WorldUpdate(update)
    }

    /// Creates a world update packet for entities within range of a position
    pub fn create_update_packet_in_range(&self, position: [f32; 3], range: f32) -> ServerMessage {
        let mut update = WorldUpdateMessage::new(self.current_tick, 0);

        // Add entities within range
        for entity in self.entities.within_distance(position, range) {
            if entity.is_active() {
                let entity_update = EntityUpdate {
                    entity_id: entity.id,
                    entity_type: entity.entity_type,
                    transform: entity.transform,
                    velocity: entity.velocity,
                    sequence: entity.sequence,
                    data: entity.data.clone(),
                };
                update.add_entity(entity_update);
            }
        }

        ServerMessage::WorldUpdate(update)
    }

    /// Removes a player from the game world
    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.player_inputs.remove(&player_id);

        // Find and despawn the player's entity
        if let Some(entity) = self.entities.find_by_player_id(player_id) {
            self.entities.despawn(entity.id);
        }
    }

    /// Clears all entities and player inputs
    pub fn clear(&mut self) {
        self.entities.clear();
        self.player_inputs.clear();
        self.current_tick = 0;
    }

    /// Returns the number of active entities
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Returns the number of active players
    #[must_use]
    pub fn player_count(&self) -> usize {
        self.entities.players().count()
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_game_world_new() {
        let world = GameWorld::new();

        assert_eq!(world.current_tick(), 0);
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.player_count(), 0);
    }

    #[test]
    fn test_game_world_spawn_player() {
        let mut world = GameWorld::new();

        let player_id = PlayerId::new(1);
        let entity_id = world.spawn_player(player_id, "TestPlayer", [0.0, 0.0, 0.0]);

        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.player_count(), 1);

        let entity = world.entities().get(entity_id);
        assert!(entity.is_some());
        assert!(entity.unwrap().is_player());
    }

    #[test]
    fn test_game_world_spawn_monster() {
        let mut world = GameWorld::new();

        let entity_id = world.spawn_monster(42, [10.0, 0.0, 20.0]);

        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.player_count(), 0);

        let entity = world.entities().get(entity_id);
        assert!(entity.is_some());
        assert!(entity.unwrap().is_monster());
    }

    #[test]
    fn test_game_world_despawn_entity() {
        let mut world = GameWorld::new();

        let entity_id = world.spawn_monster(1, [0.0, 0.0, 0.0]);
        assert_eq!(world.entity_count(), 1);

        world.despawn_entity(entity_id);

        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_game_world_set_player_input() {
        let mut world = GameWorld::new();

        let player_id = PlayerId::new(1);
        world.spawn_player(player_id, "TestPlayer", [0.0, 0.0, 0.0]);

        let mut input = PlayerInput::new(0);
        input.add_action(trueworld_core::InputAction::MoveForward);

        world.set_player_input(player_id, input);

        let stored_input = world.get_player_input(player_id);
        assert!(stored_input.is_some());
        assert!(stored_input.unwrap().has_action(trueworld_core::InputAction::MoveForward));
    }

    #[test]
    fn test_game_world_update_increases_tick() {
        let mut world = GameWorld::new();

        assert_eq!(world.current_tick(), 0);

        world.update(Duration::from_secs_f32(1.0));

        assert_eq!(world.current_tick(), 1);

        world.update(Duration::from_secs_f32(1.0));

        assert_eq!(world.current_tick(), 2);
    }

    #[test]
    fn test_game_world_update_moves_entities() {
        let mut world = GameWorld::new();

        let entity_id = world.spawn_monster(1, [0.0, 0.0, 0.0]);
        world.entities_mut()
            .get_mut(entity_id)
            .unwrap()
            .set_velocity([1.0, 0.0, 0.0]);

        world.update(Duration::from_secs_f32(1.0));

        let entity = world.entities().get(entity_id).unwrap();
        assert_eq!(entity.position(), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_game_world_create_update_packet() {
        let mut world = GameWorld::new();

        world.spawn_player(PlayerId::new(1), "Player1", [0.0, 0.0, 0.0]);
        world.spawn_monster(42, [10.0, 0.0, 0.0]);

        let packet = world.create_update_packet();

        match packet {
            ServerMessage::WorldUpdate(update) => {
                assert_eq!(update.server_timestamp, 0);
                assert_eq!(update.entities.len(), 2);
            }
            _ => panic!("Expected WorldUpdate message"),
        }
    }

    #[test]
    fn test_game_world_create_update_packet_in_range() {
        let mut world = GameWorld::new();

        world.spawn_monster(1, [0.0, 0.0, 0.0]);
        world.spawn_monster(2, [5.0, 0.0, 0.0]);
        world.spawn_monster(3, [20.0, 0.0, 0.0]);

        let packet = world.create_update_packet_in_range([0.0, 0.0, 0.0], 10.0);

        match packet {
            ServerMessage::WorldUpdate(update) => {
                assert_eq!(update.entities.len(), 2);
            }
            _ => panic!("Expected WorldUpdate message"),
        }
    }

    #[test]
    fn test_game_world_remove_player() {
        let mut world = GameWorld::new();

        let player_id = PlayerId::new(1);
        world.spawn_player(player_id, "TestPlayer", [0.0, 0.0, 0.0]);

        assert_eq!(world.player_count(), 1);

        world.remove_player(player_id);

        assert_eq!(world.player_count(), 0);
        assert!(world.get_player_input(player_id).is_none());
    }

    #[test]
    fn test_game_world_clear() {
        let mut world = GameWorld::new();

        world.spawn_player(PlayerId::new(1), "Player1", [0.0, 0.0, 0.0]);
        world.spawn_player(PlayerId::new(2), "Player2", [10.0, 0.0, 0.0]);
        world.spawn_monster(1, [20.0, 0.0, 0.0]);

        assert_eq!(world.entity_count(), 3);

        world.clear();

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.current_tick(), 0);
    }

    #[test]
    fn test_game_world_player_input_affects_movement() {
        let mut world = GameWorld::new();

        let player_id = PlayerId::new(1);
        world.spawn_player(player_id, "TestPlayer", [0.0, 0.0, 0.0]);

        let mut input = PlayerInput::new(0);
        input.add_action(trueworld_core::InputAction::MoveForward);

        world.set_player_input(player_id, input);

        let entity = world.entities().find_by_player_id(player_id).unwrap();
        assert_eq!(entity.velocity, [0.0, 0.0, 5.0]);
    }

    #[test]
    fn test_game_world_default() {
        let world = GameWorld::default();

        assert_eq!(world.current_tick(), 0);
        assert_eq!(world.entity_count(), 0);
    }
}
