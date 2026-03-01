// Entity system for TrueWorld server
//
// This module provides the core entity management system for the game,
// including the Entity struct and EntityManager for handling all game entities.

use std::collections::HashMap;
use std::time::Duration;
use trueworld_core::{
    EntityId, PlayerId,
    TransformState, Position, Velocity,
    net::{EntityUpdate, EntityType, EntityData, PlayerEntityData, MonsterEntityData},
};

// ============================================================================
// Entity
// ============================================================================

/// A game entity representing any object in the game world
#[derive(Debug, Clone)]
pub struct Entity {
    /// Unique entity identifier
    pub id: EntityId,

    /// Type of entity
    pub entity_type: EntityType,

    /// Transform state (position, rotation, scale)
    pub transform: TransformState,

    /// Current velocity
    pub velocity: Velocity,

    /// Entity-specific data
    pub data: Option<EntityData>,

    /// Whether this entity is active
    pub active: bool,

    /// When this entity was spawned
    pub spawned_at: u64,

    /// Last update sequence number
    pub sequence: u32,
}

impl Entity {
    /// Creates a new entity with the given ID and type
    #[must_use]
    pub fn new(id: EntityId, entity_type: EntityType, position: Position) -> Self {
        Self {
            id,
            entity_type,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity: [0.0, 0.0, 0.0],
            data: None,
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Creates a new entity with full transform
    #[must_use]
    pub fn with_transform(id: EntityId, entity_type: EntityType, transform: TransformState) -> Self {
        Self {
            id,
            entity_type,
            transform,
            velocity: [0.0, 0.0, 0.0],
            data: None,
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Sets the entity type
    pub fn with_type(mut self, entity_type: EntityType) -> Self {
        self.entity_type = entity_type;
        self
    }

    /// Sets the entity-specific data
    pub fn with_data(mut self, data: EntityData) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the velocity
    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = velocity;
        self
    }

    /// Sets the spawn timestamp
    pub fn with_spawn_time(mut self, timestamp: u64) -> Self {
        self.spawned_at = timestamp;
        self
    }

    /// Returns the entity's position
    #[must_use]
    pub const fn position(&self) -> Position {
        self.transform.position
    }

    /// Returns the entity's rotation
    #[must_use]
    pub const fn rotation(&self) -> [f32; 3] {
        self.transform.rotation
    }

    /// Returns true if this entity is a player
    #[must_use]
    pub const fn is_player(&self) -> bool {
        matches!(self.entity_type, EntityType::Player)
    }

    /// Returns true if this entity is a monster
    #[must_use]
    pub const fn is_monster(&self) -> bool {
        matches!(self.entity_type, EntityType::Monster)
    }

    /// Returns true if this entity is a prop/static object
    #[must_use]
    pub const fn is_prop(&self) -> bool {
        matches!(self.entity_type, EntityType::Prop)
    }

    /// Returns true if this entity is an item
    #[must_use]
    pub const fn is_item(&self) -> bool {
        matches!(self.entity_type, EntityType::Item)
    }

    /// Returns true if this entity is a projectile
    #[must_use]
    pub const fn is_projectile(&self) -> bool {
        matches!(self.entity_type, EntityType::Projectile)
    }

    /// Returns true if this entity is an effect
    #[must_use]
    pub const fn is_effect(&self) -> bool {
        matches!(self.entity_type, EntityType::Effect)
    }

    /// Updates the entity's position based on velocity and delta time
    pub fn update_position(&mut self, delta_time: Duration) {
        let dt = delta_time.as_secs_f32();
        self.transform.position[0] += self.velocity[0] * dt;
        self.transform.position[1] += self.velocity[1] * dt;
        self.transform.position[2] += self.velocity[2] * dt;
    }

    /// Sets the entity's position
    pub fn set_position(&mut self, position: Position) {
        self.transform.position = position;
    }

    /// Sets the entity's rotation
    pub fn set_rotation(&mut self, rotation: [f32; 3]) {
        self.transform.rotation = rotation;
    }

    /// Sets the entity's velocity
    pub fn set_velocity(&mut self, velocity: Velocity) {
        self.velocity = velocity;
    }

    /// Translates the entity by a given offset
    pub fn translate(&mut self, offset: [f32; 3]) {
        self.transform.translate(offset);
    }

    /// Rotates the entity by a given Euler angle offset
    pub fn rotate(&mut self, offset: [f32; 3]) {
        self.transform.rotate(offset);
    }

    /// Marks the entity as inactive (despawned)
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Returns true if the entity is active
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Creates an EntityUpdate for network synchronization
    #[must_use]
    pub fn create_update(&self) -> EntityUpdate {
        EntityUpdate {
            entity_id: self.id,
            entity_type: self.entity_type,
            transform: self.transform,
            velocity: self.velocity,
            sequence: self.sequence,
            data: self.data.clone(),
        }
    }

    /// Creates a player entity
    #[must_use]
    pub fn player(id: EntityId, player_id: PlayerId, name: impl Into<String>, position: Position) -> Self {
        Self {
            id,
            entity_type: EntityType::Player,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity: [0.0, 0.0, 0.0],
            data: Some(EntityData::Player(
                PlayerEntityData::new(player_id, name)
            )),
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Creates a monster entity
    #[must_use]
    pub fn monster(id: EntityId, monster_type: u32, position: Position) -> Self {
        Self {
            id,
            entity_type: EntityType::Monster,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity: [0.0, 0.0, 0.0],
            data: Some(EntityData::Monster(
                MonsterEntityData::new(monster_type)
            )),
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Creates a prop entity
    #[must_use]
    pub fn prop(id: EntityId, position: Position) -> Self {
        Self {
            id,
            entity_type: EntityType::Prop,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity: [0.0, 0.0, 0.0],
            data: None,
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Creates an item entity
    #[must_use]
    pub fn item(id: EntityId, position: Position) -> Self {
        Self {
            id,
            entity_type: EntityType::Item,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity: [0.0, 0.0, 0.0],
            data: None,
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Creates a projectile entity
    #[must_use]
    pub fn projectile(id: EntityId, position: Position, velocity: Velocity) -> Self {
        Self {
            id,
            entity_type: EntityType::Projectile,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity,
            data: None,
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }

    /// Creates an effect entity
    #[must_use]
    pub fn effect(id: EntityId, position: Position) -> Self {
        Self {
            id,
            entity_type: EntityType::Effect,
            transform: TransformState::new(position, [0.0, 0.0, 0.0], 1.0),
            velocity: [0.0, 0.0, 0.0],
            data: None,
            active: true,
            spawned_at: 0,
            sequence: 0,
        }
    }
}

// ============================================================================
// EntityManager
// ============================================================================

/// Manages all entities in the game world
#[derive(Debug, Default)]
pub struct EntityManager {
    /// All entities indexed by ID
    entities: HashMap<EntityId, Entity>,

    /// Counter for generating new entity IDs
    next_entity_id: u64,

    /// Current tick for sequence numbers
    current_sequence: u32,
}

impl EntityManager {
    /// Creates a new entity manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_entity_id: 1,
            current_sequence: 0,
        }
    }

    /// Generates a new unique entity ID
    fn generate_id(&mut self) -> EntityId {
        let id = EntityId::new(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    /// Spawns a new entity
    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        self.entities.insert(id, entity);
        id
    }

    /// Spawns a new entity with an auto-generated ID
    pub fn spawn_auto(&mut self, mut entity: Entity) -> EntityId {
        let id = self.generate_id();
        entity.id = id;
        self.entities.insert(id, entity);
        id
    }

    /// Spawns a player entity
    pub fn spawn_player(&mut self, player_id: PlayerId, name: impl Into<String>, position: Position) -> EntityId {
        let id = self.generate_id();
        let entity = Entity::player(id, player_id, name, position)
            .with_spawn_time(self.current_sequence as u64);
        self.entities.insert(id, entity);
        id
    }

    /// Spawns a monster entity
    pub fn spawn_monster(&mut self, monster_type: u32, position: Position) -> EntityId {
        let id = self.generate_id();
        let entity = Entity::monster(id, monster_type, position)
            .with_spawn_time(self.current_sequence as u64);
        self.entities.insert(id, entity);
        id
    }

    /// Spawns a prop entity
    pub fn spawn_prop(&mut self, position: Position) -> EntityId {
        let id = self.generate_id();
        let entity = Entity::prop(id, position);
        self.entities.insert(id, entity);
        id
    }

    /// Spawns an item entity
    pub fn spawn_item(&mut self, position: Position) -> EntityId {
        let id = self.generate_id();
        let entity = Entity::item(id, position);
        self.entities.insert(id, entity);
        id
    }

    /// Spawns a projectile entity
    pub fn spawn_projectile(&mut self, position: Position, velocity: Velocity) -> EntityId {
        let id = self.generate_id();
        let entity = Entity::projectile(id, position, velocity);
        self.entities.insert(id, entity);
        id
    }

    /// Spawns an effect entity
    pub fn spawn_effect(&mut self, position: Position) -> EntityId {
        let id = self.generate_id();
        let entity = Entity::effect(id, position);
        self.entities.insert(id, entity);
        id
    }

    /// Despawns an entity by ID
    pub fn despawn(&mut self, entity_id: EntityId) -> Option<Entity> {
        self.entities.remove(&entity_id)
    }

    /// Gets a reference to an entity by ID
    #[must_use]
    pub fn get(&self, entity_id: EntityId) -> Option<&Entity> {
        self.entities.get(&entity_id)
    }

    /// Gets a mutable reference to an entity by ID
    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&entity_id)
    }

    /// Returns true if an entity with the given ID exists
    #[must_use]
    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.entities.contains_key(&entity_id)
    }

    /// Returns the number of active entities
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns true if there are no entities
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Updates all entities (called each tick)
    pub fn update(&mut self, delta_time: Duration) {
        self.current_sequence = self.current_sequence.wrapping_add(1);

        // Update all entity positions based on velocity
        for entity in self.entities.values_mut() {
            if entity.active {
                entity.update_position(delta_time);
                entity.sequence = self.current_sequence;
            }
        }

        // Remove inactive entities
        self.entities.retain(|_, entity| entity.active);
    }

    /// Clears all entities
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Returns all entities
    #[must_use]
    pub fn all(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Returns all mutable entities
    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities.values_mut()
    }

    /// Returns all entities of a specific type
    #[must_use]
    pub fn by_type(&self, entity_type: EntityType) -> impl Iterator<Item = &Entity> {
        self.entities.values().filter(move |e| e.entity_type == entity_type)
    }

    /// Returns all entities within a certain distance of a position
    #[must_use]
    pub fn within_distance(&self, position: Position, max_distance: f32) -> impl Iterator<Item = &Entity> {
        let max_distance_sq = max_distance * max_distance;
        self.entities.values().filter(move |e| {
            let dx = e.transform.position[0] - position[0];
            let dy = e.transform.position[1] - position[1];
            let dz = e.transform.position[2] - position[2];
            (dx * dx + dy * dy + dz * dz) <= max_distance_sq
        })
    }

    /// Gets all player entities
    #[must_use]
    pub fn players(&self) -> impl Iterator<Item = &Entity> {
        self.by_type(EntityType::Player)
    }

    /// Gets all monster entities
    #[must_use]
    pub fn monsters(&self) -> impl Iterator<Item = &Entity> {
        self.by_type(EntityType::Monster)
    }

    /// Gets all projectile entities
    #[must_use]
    pub fn projectiles(&self) -> impl Iterator<Item = &Entity> {
        self.by_type(EntityType::Projectile)
    }

    /// Finds an entity by player ID
    #[must_use]
    pub fn find_by_player_id(&self, player_id: PlayerId) -> Option<&Entity> {
        self.entities.values().find(|e| {
            if let Some(EntityData::Player(data)) = &e.data {
                data.player_id == player_id
            } else {
                false
            }
        })
    }

    /// Finds an entity by player ID (mutable)
    pub fn find_by_player_id_mut(&mut self, player_id: PlayerId) -> Option<&mut Entity> {
        self.entities.values_mut().find(|e| {
            if let Some(EntityData::Player(data)) = &e.data {
                data.player_id == player_id
            } else {
                false
            }
        })
    }

    /// Creates update packets for all entities
    pub fn create_updates(&self) -> Vec<EntityUpdate> {
        self.entities.values()
            .filter(|e| e.active)
            .map(|e| e.create_update())
            .collect()
    }

    /// Creates update packets for entities within range of a position
    pub fn create_updates_in_range(&self, position: Position, range: f32) -> Vec<EntityUpdate> {
        self.within_distance(position, range)
            .filter(|e| e.active)
            .map(|e| e.create_update())
            .collect()
    }

    /// Returns the entity IDs of all active entities
    #[must_use]
    pub fn entity_ids(&self) -> Vec<EntityId> {
        self.entities.keys().copied().collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ------------------------------------------------------------------------
    // Entity Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_entity_new() {
        let id = EntityId::new(1);
        let entity = Entity::new(id, EntityType::Player, [0.0, 0.0, 0.0]);

        assert_eq!(entity.id, id);
        assert_eq!(entity.entity_type, EntityType::Player);
        assert_eq!(entity.transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(entity.velocity, [0.0, 0.0, 0.0]);
        assert!(entity.data.is_none());
        assert!(entity.active);
    }

    #[test]
    fn test_entity_builder() {
        let id = EntityId::new(1);
        let entity = Entity::new(id, EntityType::Player, [10.0, 20.0, 30.0])
            .with_velocity([1.0, 2.0, 3.0])
            .with_type(EntityType::Monster);

        assert_eq!(entity.entity_type, EntityType::Monster);
        assert_eq!(entity.velocity, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_entity_position() {
        let id = EntityId::new(1);
        let entity = Entity::new(id, EntityType::Player, [5.0, 10.0, 15.0]);

        assert_eq!(entity.position(), [5.0, 10.0, 15.0]);
    }

    #[test]
    fn test_entity_type_checks() {
        let player = Entity::new(EntityId::new(1), EntityType::Player, [0.0, 0.0, 0.0]);
        let monster = Entity::new(EntityId::new(2), EntityType::Monster, [0.0, 0.0, 0.0]);
        let prop = Entity::new(EntityId::new(3), EntityType::Prop, [0.0, 0.0, 0.0]);
        let item = Entity::new(EntityId::new(4), EntityType::Item, [0.0, 0.0, 0.0]);
        let projectile = Entity::new(EntityId::new(5), EntityType::Projectile, [0.0, 0.0, 0.0]);
        let effect = Entity::new(EntityId::new(6), EntityType::Effect, [0.0, 0.0, 0.0]);

        assert!(player.is_player());
        assert!(monster.is_monster());
        assert!(prop.is_prop());
        assert!(item.is_item());
        assert!(projectile.is_projectile());
        assert!(effect.is_effect());

        assert!(!player.is_monster());
        assert!(!monster.is_player());
    }

    #[test]
    fn test_entity_update_position() {
        let id = EntityId::new(1);
        let mut entity = Entity::new(id, EntityType::Player, [0.0, 0.0, 0.0])
            .with_velocity([1.0, 2.0, 3.0]);

        entity.update_position(Duration::from_secs_f32(1.0));

        assert_eq!(entity.position(), [1.0, 2.0, 3.0]);

        entity.update_position(Duration::from_secs_f32(0.5));

        assert_eq!(entity.position(), [1.5, 3.0, 4.5]);
    }

    #[test]
    fn test_entity_set_position() {
        let id = EntityId::new(1);
        let mut entity = Entity::new(id, EntityType::Player, [0.0, 0.0, 0.0]);

        entity.set_position([10.0, 20.0, 30.0]);

        assert_eq!(entity.position(), [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_entity_set_velocity() {
        let id = EntityId::new(1);
        let mut entity = Entity::new(id, EntityType::Player, [0.0, 0.0, 0.0]);

        entity.set_velocity([5.0, 10.0, 15.0]);

        assert_eq!(entity.velocity, [5.0, 10.0, 15.0]);
    }

    #[test]
    fn test_entity_translate() {
        let id = EntityId::new(1);
        let mut entity = Entity::new(id, EntityType::Player, [10.0, 20.0, 30.0]);

        entity.translate([1.0, 2.0, 3.0]);

        assert_eq!(entity.position(), [11.0, 22.0, 33.0]);
    }

    #[test]
    fn test_entity_deactivate() {
        let id = EntityId::new(1);
        let mut entity = Entity::new(id, EntityType::Player, [0.0, 0.0, 0.0]);

        assert!(entity.is_active());

        entity.deactivate();

        assert!(!entity.is_active());
    }

    #[test]
    fn test_entity_player_factory() {
        let entity = Entity::player(
            EntityId::new(1),
            PlayerId::new(100),
            "TestPlayer",
            [5.0, 10.0, 15.0],
        );

        assert!(entity.is_player());
        assert_eq!(entity.position(), [5.0, 10.0, 15.0]);

        if let Some(EntityData::Player(data)) = &entity.data {
            assert_eq!(data.player_id, PlayerId::new(100));
            assert_eq!(data.name, "TestPlayer");
        } else {
            panic!("Expected player data");
        }
    }

    #[test]
    fn test_entity_monster_factory() {
        let entity = Entity::monster(EntityId::new(1), 42, [5.0, 0.0, 10.0]);

        assert!(entity.is_monster());
        assert_eq!(entity.position(), [5.0, 0.0, 10.0]);

        if let Some(EntityData::Monster(data)) = &entity.data {
            assert_eq!(data.monster_type, 42);
        } else {
            panic!("Expected monster data");
        }
    }

    #[test]
    fn test_entity_projectile_factory() {
        let entity = Entity::projectile(
            EntityId::new(1),
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
        );

        assert!(entity.is_projectile());
        assert_eq!(entity.velocity, [10.0, 0.0, 0.0]);
    }

    #[test]
    fn test_entity_create_update() {
        let entity = Entity::player(
            EntityId::new(1),
            PlayerId::new(100),
            "TestPlayer",
            [5.0, 10.0, 15.0],
        ).with_velocity([1.0, 2.0, 3.0]);

        let update = entity.create_update();

        assert_eq!(update.entity_id, EntityId::new(1));
        assert_eq!(update.entity_type, EntityType::Player);
        assert_eq!(update.transform.position, [5.0, 10.0, 15.0]);
        assert_eq!(update.velocity, [1.0, 2.0, 3.0]);
    }

    // ------------------------------------------------------------------------
    // EntityManager Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_entity_manager_new() {
        let manager = EntityManager::new();

        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_entity_manager_spawn_player() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_player(PlayerId::new(100), "TestPlayer", [0.0, 0.0, 0.0]);

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id));

        let entity = manager.get(id).unwrap();
        assert!(entity.is_player());
    }

    #[test]
    fn test_entity_manager_spawn_monster() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_monster(42, [10.0, 0.0, 20.0]);

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id));

        let entity = manager.get(id).unwrap();
        assert!(entity.is_monster());
    }

    #[test]
    fn test_entity_manager_spawn_prop() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_prop([5.0, 0.0, 10.0]);

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id));

        let entity = manager.get(id).unwrap();
        assert!(entity.is_prop());
    }

    #[test]
    fn test_entity_manager_spawn_item() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_item([5.0, 0.0, 10.0]);

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id));

        let entity = manager.get(id).unwrap();
        assert!(entity.is_item());
    }

    #[test]
    fn test_entity_manager_spawn_projectile() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_projectile([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id));

        let entity = manager.get(id).unwrap();
        assert!(entity.is_projectile());
        assert_eq!(entity.velocity, [10.0, 0.0, 0.0]);
    }

    #[test]
    fn test_entity_manager_spawn_effect() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_effect([5.0, 10.0, 15.0]);

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id));

        let entity = manager.get(id).unwrap();
        assert!(entity.is_effect());
    }

    #[test]
    fn test_entity_manager_despawn() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_prop([0.0, 0.0, 0.0]);
        assert_eq!(manager.len(), 1);

        let entity = manager.despawn(id).unwrap();
        assert_eq!(entity.id, id);
        assert_eq!(manager.len(), 0);
        assert!(!manager.contains(id));
    }

    #[test]
    fn test_entity_manager_get() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_player(PlayerId::new(100), "TestPlayer", [0.0, 0.0, 0.0]);

        let entity = manager.get(id);
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().id, id);

        assert!(manager.get(EntityId::new(999)).is_none());
    }

    #[test]
    fn test_entity_manager_get_mut() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_player(PlayerId::new(100), "TestPlayer", [0.0, 0.0, 0.0]);

        let entity = manager.get_mut(id);
        assert!(entity.is_some());

        entity.unwrap().set_position([10.0, 20.0, 30.0]);

        assert_eq!(manager.get(id).unwrap().position(), [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_entity_manager_update() {
        let mut manager = EntityManager::new();

        let id = manager.spawn_projectile([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]);

        manager.update(Duration::from_secs_f32(1.0));

        let entity = manager.get(id).unwrap();
        assert_eq!(entity.position(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_entity_manager_update_removes_inactive() {
        let mut manager = EntityManager::new();

        let id1 = manager.spawn_prop([0.0, 0.0, 0.0]);
        let id2 = manager.spawn_prop([10.0, 0.0, 0.0]);

        assert_eq!(manager.len(), 2);

        // Deactivate one entity
        manager.get_mut(id1).unwrap().deactivate();

        manager.update(Duration::from_secs_f32(1.0));

        assert_eq!(manager.len(), 1);
        assert!(manager.contains(id2));
        assert!(!manager.contains(id1));
    }

    #[test]
    fn test_entity_manager_clear() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(1), "Player1", [0.0, 0.0, 0.0]);
        manager.spawn_player(PlayerId::new(2), "Player2", [10.0, 0.0, 0.0]);
        manager.spawn_monster(1, [20.0, 0.0, 0.0]);

        assert_eq!(manager.len(), 3);

        manager.clear();

        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_entity_manager_by_type() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(1), "Player1", [0.0, 0.0, 0.0]);
        manager.spawn_player(PlayerId::new(2), "Player2", [10.0, 0.0, 0.0]);
        manager.spawn_monster(1, [20.0, 0.0, 0.0]);
        manager.spawn_prop([30.0, 0.0, 0.0]);

        let players = manager.by_type(EntityType::Player).collect::<Vec<_>>();
        assert_eq!(players.len(), 2);

        let monsters = manager.by_type(EntityType::Monster).collect::<Vec<_>>();
        assert_eq!(monsters.len(), 1);

        let props = manager.by_type(EntityType::Prop).collect::<Vec<_>>();
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn test_entity_manager_players() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(1), "Player1", [0.0, 0.0, 0.0]);
        manager.spawn_player(PlayerId::new(2), "Player2", [10.0, 0.0, 0.0]);
        manager.spawn_monster(1, [20.0, 0.0, 0.0]);

        let players = manager.players().collect::<Vec<_>>();
        assert_eq!(players.len(), 2);
    }

    #[test]
    fn test_entity_manager_monsters() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(1), "Player1", [0.0, 0.0, 0.0]);
        manager.spawn_monster(1, [10.0, 0.0, 0.0]);
        manager.spawn_monster(2, [20.0, 0.0, 0.0]);
        manager.spawn_monster(3, [30.0, 0.0, 0.0]);

        let monsters = manager.monsters().collect::<Vec<_>>();
        assert_eq!(monsters.len(), 3);
    }

    #[test]
    fn test_entity_manager_within_distance() {
        let mut manager = EntityManager::new();

        manager.spawn_prop([0.0, 0.0, 0.0]);
        manager.spawn_prop([5.0, 0.0, 0.0]);
        manager.spawn_prop([15.0, 0.0, 0.0]);
        manager.spawn_prop([25.0, 0.0, 0.0]);

        let nearby = manager.within_distance([0.0, 0.0, 0.0], 10.0).collect::<Vec<_>>();
        assert_eq!(nearby.len(), 2);
    }

    #[test]
    fn test_entity_manager_find_by_player_id() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(100), "Player1", [0.0, 0.0, 0.0]);
        manager.spawn_player(PlayerId::new(200), "Player2", [10.0, 0.0, 0.0]);
        manager.spawn_monster(1, [20.0, 0.0, 0.0]);

        let entity = manager.find_by_player_id(PlayerId::new(100));
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().position(), [0.0, 0.0, 0.0]);

        assert!(manager.find_by_player_id(PlayerId::new(999)).is_none());
    }

    #[test]
    fn test_entity_manager_find_by_player_id_mut() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(100), "Player1", [0.0, 0.0, 0.0]);

        let entity = manager.find_by_player_id_mut(PlayerId::new(100));
        assert!(entity.is_some());

        entity.unwrap().set_position([10.0, 20.0, 30.0]);

        assert_eq!(
            manager.find_by_player_id(PlayerId::new(100)).unwrap().position(),
            [10.0, 20.0, 30.0]
        );
    }

    #[test]
    fn test_entity_manager_create_updates() {
        let mut manager = EntityManager::new();

        manager.spawn_player(PlayerId::new(100), "Player1", [0.0, 0.0, 0.0]);
        manager.spawn_monster(1, [10.0, 0.0, 0.0]);

        let updates = manager.create_updates();
        assert_eq!(updates.len(), 2);
    }

    #[test]
    fn test_entity_manager_create_updates_in_range() {
        let mut manager = EntityManager::new();

        manager.spawn_prop([0.0, 0.0, 0.0]);
        manager.spawn_prop([5.0, 0.0, 0.0]);
        manager.spawn_prop([20.0, 0.0, 0.0]);

        let updates = manager.create_updates_in_range([0.0, 0.0, 0.0], 10.0);
        assert_eq!(updates.len(), 2);
    }

    #[test]
    fn test_entity_manager_entity_ids() {
        let mut manager = EntityManager::new();

        let id1 = manager.spawn_prop([0.0, 0.0, 0.0]);
        let id2 = manager.spawn_prop([10.0, 0.0, 0.0]);

        let ids = manager.entity_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}
