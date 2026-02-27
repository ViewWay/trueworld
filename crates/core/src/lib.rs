// TrueWorld Core Library
//
// This library provides shared types, error handling, math utilities,
// and network definitions for the TrueWorld game project.

#![warn(missing_docs)]
#![warn(clippy::all)]

// Public modules
pub mod error;
pub mod types;
pub mod math;
pub mod net;
pub mod time;
pub mod id;

// Re-exports commonly used types
pub use error::{Error, Result};
pub use id::{EntityId, PlayerId, RoomId, SkillId, ItemId, MonsterId, ServerId, PetId, ObjectId, AttackTypeId};
pub use time::{GameTime, Duration, Timestamp};
pub use math::{Vec2, Vec3, Vec4, Quat, Mat4};
pub use types::{
    Coord2, Coord3, Direction, Element, GameState, InputAction, PlayerInput, Position,
    Rarity, Rotation, TransformState, Velocity,
};
pub use net::{
    ClientMessage, ServerMessage, EntityUpdate, EntityType, EntityData,
    PlayerEntityData, MonsterEntityData, ConnectMessage, ConnectResultMessage,
    PlayerInputMessage, PingMessage, PongMessage, WorldUpdateMessage,
    NetAddress, PacketType,
    serialize_client_message, deserialize_client_message,
    serialize_server_message, deserialize_server_message,
    MessageError,
};

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::id::{EntityId, PlayerId, RoomId, SkillId, ItemId, MonsterId, ServerId, PetId, ObjectId, AttackTypeId};
    pub use crate::time::{GameTime, Duration, Timestamp};
    pub use crate::types::{
        Coord2, Coord3, Direction, Element, GameState, InputAction, PlayerInput, Position,
        Rarity, Rotation, TransformState, Velocity,
    };
    pub use crate::math::{Vec2, Vec3, Vec4, Quat, Mat4};
    pub use crate::net::{
        ClientMessage, ServerMessage, EntityUpdate, EntityType, EntityData,
        ConnectMessage, ConnectResultMessage, PingMessage, PongMessage,
        WorldUpdateMessage, NetAddress, PacketType,
        serialize_client_message, deserialize_client_message,
        serialize_server_message, deserialize_server_message,
    };
}
