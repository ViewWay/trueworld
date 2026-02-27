// Server → Client packets

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use trueworld_core::{PlayerId, EntityId, SkillId, ServerId, Vec3, Quat};

/// Hello response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    pub protocol_version: u32,
    pub server_id: ServerId,
    pub timestamp: DateTime<Utc>,
}

/// Welcome packet with player spawn info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerWelcome {
    pub player_id: PlayerId,
    pub entity_id: EntityId,
    pub position: Vec3,
    pub rotation: Quat,
    /// Server tick rate
    pub tick_rate: u32,
    /// Current game time
    pub game_time: f32,
}

/// Player spawn notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPlayerSpawn {
    pub player_id: PlayerId,
    pub entity_id: EntityId,
    pub username: String,
    pub position: Vec3,
    pub rotation: Quat,
    /// Player appearance data (simplified)
    pub appearance: Vec<u8>,
}

/// Player despawn notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPlayerDespawn {
    pub player_id: PlayerId,
    pub reason: u8, // 0=disconnect, 1=kick, 2=logout, 3=death
}

/// Player state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPlayerUpdate {
    pub player_id: PlayerId,
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    /// Animation state (simplified)
    pub animation: u8,
    pub action: u8,
}

/// Entity spawn notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntitySpawn {
    pub entity_id: EntityId,
    pub entity_type: u8, // 0=npc, 1=monster, 2=pet, 3=object
    pub position: Vec3,
    pub rotation: Quat,
    /// Entity-specific data
    pub data: Vec<u8>,
}

/// Entity despawn notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntityDespawn {
    pub entity_id: EntityId,
}

/// Entity state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntityUpdate {
    pub entity_id: EntityId,
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub health_percent: Option<f32>,
}

/// World state update (batched entity updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerWorldUpdate {
    pub tick: u64,
    pub updates: Vec<WorldUpdateEntry>,
}

/// Individual world update entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldUpdateEntry {
    Player(ServerPlayerUpdate),
    Entity(ServerEntityUpdate),
}

/// Chat message from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChat {
    pub sender_id: Option<PlayerId>,
    pub sender_name: Option<String>,
    pub message: String,
    pub channel: u8,
}

/// Game state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGameState {
    pub tick: u64,
    pub players: Vec<PlayerState>,
    pub entities: Vec<EntityState>,
}

/// Compact player state for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: PlayerId,
    pub position: Vec3,
    pub rotation: Quat,
    pub health: i32,
    pub max_health: i32,
}

/// Compact entity state for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub entity_id: EntityId,
    pub position: Vec3,
    pub rotation: Quat,
    pub health_percent: f32,
}

/// Health update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealthUpdate {
    pub current: i32,
    pub maximum: i32,
    pub delta: i32,
}

/// Mana update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerManaUpdate {
    pub current: i32,
    pub maximum: i32,
    pub delta: i32,
}

/// Experience update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerExpUpdate {
    pub current: u64,
    pub maximum: u64,
    pub gained: u64,
}

/// Level up notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLevelUp {
    pub new_level: u8,
    pub skill_points: u8,
    pub stat_points: u8,
}

/// Damage notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDamage {
    pub target_id: EntityId,
    pub damage: i32,
    pub damage_type: u8,
    pub is_critical: bool,
}

/// Heal notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHeal {
    pub target_id: EntityId,
    pub amount: i32,
    pub heal_type: u8,
}

/// Pong response to ping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPong {
    pub ping_sequence: u32,
    pub server_timestamp: DateTime<Utc>,
}

/// Error packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    pub code: u8,
    pub message: String,
}

/// Kick notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerKick {
    pub reason: u8,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_packet() {
        let welcome = ServerWelcome {
            player_id: PlayerId::new(1),
            entity_id: EntityId::new(100),
            position: Vec3::new(100.0, 0.0, 200.0),
            rotation: Quat::IDENTITY,
            tick_rate: 60,
            game_time: 0.0,
        };

        let bytes = crate::serialize_packet(&welcome).unwrap();
        let decoded: ServerWelcome = crate::deserialize_packet(&bytes).unwrap();

        assert_eq!(welcome.player_id, decoded.player_id);
        assert_eq!(welcome.tick_rate, decoded.tick_rate);
    }
}
