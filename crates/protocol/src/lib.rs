// trueworld-protocol: Network protocol definitions for TrueWorld
//
// This crate defines all network packets and serialization logic for
// client-server communication.

mod client;
mod server;
mod channel;

pub use client::*;
pub use server::*;
pub use channel::*;

use renet::ChannelConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use trueworld_core::*;

/// Protocol-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid packet ID: {0}")]
    InvalidPacketId(u8),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Packet too large: {0} bytes (max {1})")]
    PacketTooLarge(usize, usize),

    #[error("Protocol version mismatch: client={0}, server={1}")]
    VersionMismatch(u32, u32),
}

/// Result type for protocol operations (use std::result::Result to avoid conflict)
pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum payload size for packets
pub const MAX_PACKET_SIZE: usize = 64 * 1024;

/// Default channel configurations for renet 0.0.16
pub fn default_channels() -> [ChannelConfig; 3] {
    [
        // Reliable Ordered - for critical game state updates
        ChannelConfig {
            channel_id: 0,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: renet::SendType::ReliableOrdered {
                resend_time: Duration::from_millis(100),
            }
        },
        // Reliable Unordered - for non-critical updates
        ChannelConfig {
            channel_id: 1,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: renet::SendType::ReliableUnordered {
                resend_time: Duration::from_millis(100),
            }
        },
        // Unreliable - for position updates (can interpolate)
        ChannelConfig {
            channel_id: 2,
            max_memory_usage_bytes: 10 * 1024 * 1024,
            send_type: renet::SendType::Unreliable,
        },
    ]
}

/// Packet ID for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketId {
    // Client → Server packets
    ClientHello = 0,
    ClientInput = 1,
    ClientAction = 2,
    ClientChat = 3,
    ClientEmote = 4,
    ClientInteract = 5,
    ClientMove = 6,
    ClientJump = 7,
    ClientAttack = 8,
    ClientUseSkill = 9,
    ClientPing = 10,

    // Server → Client packets
    ServerHello = 100,
    ServerWelcome = 101,
    ServerPlayerSpawn = 102,
    ServerPlayerDespawn = 103,
    ServerPlayerUpdate = 104,
    ServerEntitySpawn = 105,
    ServerEntityDespawn = 106,
    ServerEntityUpdate = 107,
    ServerWorldUpdate = 108,
    ServerChat = 109,
    ServerGameState = 110,
    ServerHealthUpdate = 111,
    ServerManaUpdate = 112,
    ServerExpUpdate = 113,
    ServerLevelUp = 114,
    ServerDamage = 115,
    ServerHeal = 116,
    ServerPong = 117,
    ServerError = 118,
    ServerKick = 119,
}

impl TryFrom<u8> for PacketId {
    type Error = ProtocolError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(PacketId::ClientHello),
            1 => Ok(PacketId::ClientInput),
            2 => Ok(PacketId::ClientAction),
            3 => Ok(PacketId::ClientChat),
            4 => Ok(PacketId::ClientEmote),
            5 => Ok(PacketId::ClientInteract),
            6 => Ok(PacketId::ClientMove),
            7 => Ok(PacketId::ClientJump),
            8 => Ok(PacketId::ClientAttack),
            9 => Ok(PacketId::ClientUseSkill),
            10 => Ok(PacketId::ClientPing),
            100 => Ok(PacketId::ServerHello),
            101 => Ok(PacketId::ServerWelcome),
            102 => Ok(PacketId::ServerPlayerSpawn),
            103 => Ok(PacketId::ServerPlayerDespawn),
            104 => Ok(PacketId::ServerPlayerUpdate),
            105 => Ok(PacketId::ServerEntitySpawn),
            106 => Ok(PacketId::ServerEntityDespawn),
            107 => Ok(PacketId::ServerEntityUpdate),
            108 => Ok(PacketId::ServerWorldUpdate),
            109 => Ok(PacketId::ServerChat),
            110 => Ok(PacketId::ServerGameState),
            111 => Ok(PacketId::ServerHealthUpdate),
            112 => Ok(PacketId::ServerManaUpdate),
            113 => Ok(PacketId::ServerExpUpdate),
            114 => Ok(PacketId::ServerLevelUp),
            115 => Ok(PacketId::ServerDamage),
            116 => Ok(PacketId::ServerHeal),
            117 => Ok(PacketId::ServerPong),
            118 => Ok(PacketId::ServerError),
            119 => Ok(PacketId::ServerKick),
            _ => Err(ProtocolError::InvalidPacketId(value)),
        }
    }
}

/// Serialize a packet to bytes
pub fn serialize_packet<T: Serialize>(packet: &T) -> ProtocolResult<Vec<u8>> {
    bincode::serialize(packet)
        .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))
}

/// Deserialize a packet from bytes
pub fn deserialize_packet<'a, T: Deserialize<'a>>(
    bytes: &'a [u8],
) -> ProtocolResult<T> {
    bincode::deserialize(bytes)
        .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_id_conversion() {
        assert_eq!(PacketId::try_from(0), Ok(PacketId::ClientHello));
        assert_eq!(PacketId::try_from(100), Ok(PacketId::ServerHello));
        assert!(PacketId::try_from(255).is_err());
    }

    #[test]
    fn test_serialize_deserialize() {
        use chrono::Utc;

        let hello = ClientHello {
            protocol_version: PROTOCOL_VERSION,
            username: "TestPlayer".to_string(),
            timestamp: Utc::now(),
        };

        let bytes = serialize_packet(&hello).unwrap();
        let decoded: ClientHello = deserialize_packet(&bytes).unwrap();

        assert_eq!(hello.username, decoded.username);
        assert_eq!(hello.protocol_version, decoded.protocol_version);
    }

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
