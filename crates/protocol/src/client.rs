// Client → Server packets

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use trueworld_core::{PlayerId, EntityId, SkillId, ItemId, Vec3, Quat};

/// Initial connection packet from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    pub protocol_version: u32,
    pub username: String,
    pub timestamp: DateTime<Utc>,
}

/// Input state from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInput {
    pub sequence: u32,
    pub move_direction: Vec3,
    pub look_direction: Vec3,
    pub jumping: bool,
    pub sprinting: bool,
    pub crouching: bool,
}

impl ClientInput {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            move_direction: Vec3::ZERO,
            look_direction: Vec3::Z,
            jumping: false,
            sprinting: false,
            crouching: false,
        }
    }
}

impl Default for ClientInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Action packet from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAction {
    pub action_type: ActionType,
    pub target: Option<EntityId>,
    pub position: Option<Vec3>,
    pub timestamp: DateTime<Utc>,
}

/// Types of actions a client can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Basic attack
    Attack,
    /// Block/defend
    Block,
    /// Dodge/roll
    Dodge(Vec3),
    /// Interact with object
    Interact,
    /// Use item
    UseItem(ItemId),
    /// Cast skill
    CastSkill(SkillId),
    /// Emote
    Emote(String),
}

/// Chat message from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientChat {
    pub message: String,
    pub channel: u8, // 0=local, 1=world, 2=party, 3=guild
}

/// Emote packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEmote {
    pub emote: String,
    pub target: Option<PlayerId>,
}

/// Interaction packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInteract {
    pub target: EntityId,
    pub interaction_type: u8, // 0=use, 1=talk, 2=attack, 3=examine, 4=pickup
}

/// Movement packet (separate from input for reliability)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMove {
    pub sequence: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
}

/// Jump packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientJump {
    pub position: Vec3,
    pub velocity: Vec3,
}

/// Attack packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAttack {
    pub target: Option<EntityId>,
    pub position: Vec3,
    pub direction: Vec3,
    pub attack_type: u8, // 0=light, 1=heavy, 2=special
}

/// Use skill packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUseSkill {
    pub skill_id: SkillId,
    pub target: Option<EntityId>,
    pub position: Option<Vec3>,
}

/// Ping packet for latency measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPing {
    pub timestamp: DateTime<Utc>,
    pub sequence: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_input_defaults() {
        let input = ClientInput::new();
        assert_eq!(input.move_direction, Vec3::ZERO);
        assert_eq!(input.look_direction, Vec3::Z);
        assert!(!input.jumping);
    }
}
