// Server → Client packets

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use trueworld_core::{PlayerId, EntityId, ServerId, Vec3, Quat};

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

/// Server position acknowledgment (unreliable channel)
///
/// Sent periodically (~20Hz) to confirm the player's position on the server.
/// Client uses this for reconciliation with local prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPositionAck {
    /// Player ID being acknowledged
    pub player_id: PlayerId,
    /// Last confirmed input sequence number
    pub ack_sequence: u32,
    /// Authoritative server position
    pub position: [f32; 3],
    /// Current velocity
    pub velocity: [f32; 3],
    /// Server timestamp
    pub server_time: u64,
}

/// Position correction (reliable channel - critical)
///
/// Sent when server needs to forcibly correct client position
/// (e.g., after collision detection or anti-cheat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPositionCorrection {
    /// Player ID being corrected
    pub player_id: PlayerId,
    /// Correct position (immediate jump)
    pub correct_position: [f32; 3],
    /// Reason for correction
    pub reason: CorrectionReason,
}

/// Reason for position correction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionReason {
    /// Client moved too fast (speed limit exceeded)
    SpeedLimitExceeded,
    /// Collision detected (client in wall)
    Collision,
    /// Server rollback (state mismatch)
    ServerRollback,
    /// Teleport (gameplay mechanic)
    Teleport,
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

    #[test]
    fn test_position_ack_serialization() {
        let ack = ServerPositionAck {
            player_id: PlayerId::new(42),
            ack_sequence: 123,
            position: [10.0, 0.0, 20.0],
            velocity: [1.0, 0.0, 0.5],
            server_time: 9876543210,
        };

        let bytes = crate::serialize_packet(&ack).unwrap();
        let decoded: ServerPositionAck = crate::deserialize_packet(&bytes).unwrap();

        assert_eq!(ack.player_id, decoded.player_id);
        assert_eq!(ack.ack_sequence, decoded.ack_sequence);
        assert_eq!(ack.position, decoded.position);
    }

    #[test]
    fn test_position_correction_serialization() {
        let correction = ServerPositionCorrection {
            player_id: PlayerId::new(42),
            correct_position: [15.0, 0.0, 25.0],
            reason: CorrectionReason::Collision,
        };

        let bytes = crate::serialize_packet(&correction).unwrap();
        let decoded: ServerPositionCorrection = crate::deserialize_packet(&bytes).unwrap();

        assert_eq!(correction.player_id, decoded.player_id);
        assert_eq!(correction.correct_position, decoded.correct_position);
        assert_eq!(correction.reason, decoded.reason);
    }

    #[test]
    fn test_correction_reason_variants() {
        assert_eq!(CorrectionReason::SpeedLimitExceeded, CorrectionReason::SpeedLimitExceeded);
        assert_ne!(CorrectionReason::Collision, CorrectionReason::Teleport);
    }
}
