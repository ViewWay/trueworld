// Network types for TrueWorld

use serde::{Deserialize, Serialize};
use crate::id::{EntityId, PlayerId};
use crate::types::{PlayerInput, TransformState, Position};

// ============================================================================
// Network Address
// ============================================================================

/// Network address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetAddress {
    pub ip: String,
    pub port: u16,
}

impl NetAddress {
    /// Creates a new network address
    #[must_use]
    pub fn new(ip: impl Into<String>, port: u16) -> Self {
        Self {
            ip: ip.into(),
            port,
        }
    }
}

// ============================================================================
// Network Packet Type
// ============================================================================

/// Network packet type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketType {
    /// Initial handshake
    Handshake,
    /// Game data transmission
    GameData,
    /// Voice data
    Voice,
}

// ============================================================================
// Client Messages
// ============================================================================

/// Messages sent from client to server
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Connection request with player info
    Connect(ConnectMessage),

    /// Player input for a specific frame
    PlayerInput(PlayerInputMessage),

    /// Raw input packet (60Hz, Unreliable channel)
    /// The actual packet is defined in trueworld_protocol::ClientInputPacket
    /// This variant uses a serialized byte representation for efficiency
    ClientInputPacket(Vec<u8>),  // Serialized ClientInputPacket

    /// Ping for latency measurement
    Ping(PingMessage),
}

/// Connection request message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectMessage {
    /// Player ID (empty for new players)
    pub player_id: Option<PlayerId>,
    /// Player name
    pub player_name: String,
    /// Authentication token (if any)
    pub auth_token: Option<String>,
    /// Client version
    pub version: String,
    /// Requested room ID (if any)
    pub room_id: Option<String>,
}

impl ConnectMessage {
    /// Creates a new connect message for a new player
    #[must_use]
    pub fn new(player_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            player_id: None,
            player_name: player_name.into(),
            auth_token: None,
            version: version.into(),
            room_id: None,
        }
    }

    /// Sets the player ID (for reconnection)
    pub fn with_player_id(mut self, player_id: PlayerId) -> Self {
        self.player_id = Some(player_id);
        self
    }

    /// Sets the authentication token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Sets the requested room ID
    pub fn with_room_id(mut self, room_id: impl Into<String>) -> Self {
        self.room_id = Some(room_id.into());
        self
    }
}

/// Player input message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInputMessage {
    /// The player's input state
    pub input: PlayerInput,
    /// Estimated client timestamp for prediction
    pub client_timestamp: u64,
}

impl PlayerInputMessage {
    /// Creates a new player input message
    #[must_use]
    pub fn new(input: PlayerInput, client_timestamp: u64) -> Self {
        Self {
            input,
            client_timestamp,
        }
    }
}

/// Ping message for latency measurement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingMessage {
    /// Ping sequence number
    pub sequence: u32,
    /// Timestamp when ping was sent
    pub timestamp: u64,
}

impl PingMessage {
    /// Creates a new ping message
    #[must_use]
    pub const fn new(sequence: u32, timestamp: u64) -> Self {
        Self { sequence, timestamp }
    }
}

// ============================================================================
// Server Messages
// ============================================================================

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

/// Server position acknowledgment (unreliable channel)
///
/// Sent periodically (~20Hz) to confirm the player's position on the server.
/// Client uses this for reconciliation with local prediction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerPositionAck {
    /// Player ID being acknowledged
    pub player_id: PlayerId,
    /// Last confirmed input sequence number
    pub ack_sequence: u32,
    /// Authoritative server position
    pub position: Position,
    /// Current velocity
    pub velocity: [f32; 3],
    /// Server timestamp
    pub server_time: u64,
}

/// Position correction (reliable channel - critical)
///
/// Sent when server needs to forcibly correct client position
/// (e.g., after collision detection or anti-cheat).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerPositionCorrection {
    /// Player ID being corrected
    pub player_id: PlayerId,
    /// Correct position (immediate jump)
    pub correct_position: Position,
    /// Reason for correction
    pub reason: CorrectionReason,
}

/// Messages sent from server to client
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Result of connection attempt
    ConnectResult(ConnectResultMessage),

    /// World state update
    WorldUpdate(WorldUpdateMessage),

    /// Pong response to ping
    Pong(PongMessage),

    /// Position acknowledgment (sent ~20Hz to confirm player position)
    PositionAck(ServerPositionAck),

    /// Position correction (sent when server needs to forcibly correct position)
    PositionCorrection(ServerPositionCorrection),
}

/// Connection result message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectResultMessage {
    /// Whether connection was successful
    pub success: bool,

    /// Assigned player ID
    pub player_id: Option<PlayerId>,

    /// Assigned entity ID
    pub entity_id: Option<EntityId>,

    /// Initial spawn position
    pub spawn_position: Option<Position>,

    /// Error message if connection failed
    pub error: Option<String>,

    /// Server timestamp
    pub server_timestamp: u64,
}

impl ConnectResultMessage {
    /// Creates a successful connection result
    #[must_use]
    pub fn success(
        player_id: PlayerId,
        entity_id: EntityId,
        spawn_position: Position,
        server_timestamp: u64,
    ) -> Self {
        Self {
            success: true,
            player_id: Some(player_id),
            entity_id: Some(entity_id),
            spawn_position: Some(spawn_position),
            error: None,
            server_timestamp,
        }
    }

    /// Creates a failed connection result
    #[must_use]
    pub fn failure(error: impl Into<String>, server_timestamp: u64) -> Self {
        Self {
            success: false,
            player_id: None,
            entity_id: None,
            spawn_position: None,
            error: Some(error.into()),
            server_timestamp,
        }
    }
}

/// World update message containing entity state changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldUpdateMessage {
    /// Server timestamp for this update
    pub server_timestamp: u64,

    /// Last acknowledged input sequence
    pub last_ack: u32,

    /// Entity updates
    pub entities: Vec<EntityUpdate>,

    /// Entities that were removed
    pub removed_entities: Vec<EntityId>,
}

impl WorldUpdateMessage {
    /// Creates a new world update message
    #[must_use]
    pub fn new(server_timestamp: u64, last_ack: u32) -> Self {
        Self {
            server_timestamp,
            last_ack,
            entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    /// Adds an entity update
    pub fn add_entity(&mut self, update: EntityUpdate) {
        self.entities.push(update);
    }

    /// Adds multiple entity updates
    pub fn add_entities(&mut self, updates: impl IntoIterator<Item = EntityUpdate>) {
        self.entities.extend(updates);
    }

    /// Marks an entity as removed
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        self.removed_entities.push(entity_id);
    }

    /// Returns true if this update has any entity changes
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.entities.is_empty() || !self.removed_entities.is_empty()
    }
}

/// Pong response message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PongMessage {
    /// Original ping sequence number
    pub ping_sequence: u32,

    /// Timestamp when ping was received
    pub ping_timestamp: u64,

    /// Timestamp when pong is sent
    pub pong_timestamp: u64,
}

impl PongMessage {
    /// Creates a new pong message
    #[must_use]
    pub const fn new(ping_sequence: u32, ping_timestamp: u64, pong_timestamp: u64) -> Self {
        Self {
            ping_sequence,
            ping_timestamp,
            pong_timestamp,
        }
    }

    /// Calculates round-trip time from timestamps
    #[must_use]
    pub fn rtt(&self) -> u64 {
        self.pong_timestamp.saturating_sub(self.ping_timestamp)
    }
}

// ============================================================================
// Entity Update
// ============================================================================

/// Update data for a single entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityUpdate {
    /// Entity ID
    pub entity_id: EntityId,

    /// Entity type for deserialization
    pub entity_type: EntityType,

    /// Transform state (position, rotation, scale)
    pub transform: TransformState,

    /// Velocity vector
    pub velocity: [f32; 3],

    /// Update sequence number
    pub sequence: u32,

    /// Entity-specific data
    pub data: Option<EntityData>,
}

impl EntityUpdate {
    /// Creates a new entity update
    #[must_use]
    pub fn new(entity_id: EntityId, entity_type: EntityType, transform: TransformState) -> Self {
        Self {
            entity_id,
            entity_type,
            transform,
            velocity: [0.0, 0.0, 0.0],
            sequence: 0,
            data: None,
        }
    }

    /// Sets the velocity
    pub fn with_velocity(mut self, velocity: [f32; 3]) -> Self {
        self.velocity = velocity;
        self
    }

    /// Sets the sequence number
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = sequence;
        self
    }

    /// Sets the entity-specific data
    pub fn with_data(mut self, data: EntityData) -> Self {
        self.data = Some(data);
        self
    }
}

/// Type of entity for network serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// Player entity
    Player,
    /// Monster/NPC entity
    Monster,
    /// Prop/Static object
    Prop,
    /// Item pickup
    Item,
    /// Projectile
    Projectile,
    /// Effect/Visual
    Effect,
}

/// Entity-specific data for updates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityData {
    /// Player-specific data
    Player(PlayerEntityData),
    /// Monster-specific data
    Monster(MonsterEntityData),
}

/// Player entity specific data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerEntityData {
    /// Player ID
    pub player_id: PlayerId,
    /// Player name
    pub name: String,
    /// Current health
    pub health: u32,
    /// Max health
    pub max_health: u32,
    /// Current level
    pub level: u8,
}

impl PlayerEntityData {
    /// Creates new player entity data
    #[must_use]
    pub fn new(player_id: PlayerId, name: impl Into<String>) -> Self {
        Self {
            player_id,
            name: name.into(),
            health: 100,
            max_health: 100,
            level: 1,
        }
    }

    /// Sets health values
    pub fn with_health(mut self, health: u32, max_health: u32) -> Self {
        self.health = health;
        self.max_health = max_health;
        self
    }

    /// Sets level
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }
}

/// Monster entity specific data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonsterEntityData {
    /// Monster type ID
    pub monster_type: u32,
    /// Current health
    pub health: u32,
    /// Max health
    pub max_health: u32,
    /// Current target entity ID (if any)
    pub target: Option<EntityId>,
}

impl MonsterEntityData {
    /// Creates new monster entity data
    #[must_use]
    pub fn new(monster_type: u32) -> Self {
        Self {
            monster_type,
            health: 100,
            max_health: 100,
            target: None,
        }
    }

    /// Sets health values
    pub fn with_health(mut self, health: u32, max_health: u32) -> Self {
        self.health = health;
        self.max_health = max_health;
        self
    }

    /// Sets the target
    pub fn with_target(mut self, target: EntityId) -> Self {
        self.target = Some(target);
        self
    }
}

// ============================================================================
// Message Serialization
// ============================================================================

/// Error type for message serialization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageError {
    /// JSON serialization failed
    SerializationError,
    /// JSON deserialization failed
    DeserializationError,
    /// Invalid message format
    InvalidFormat,
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError => write!(f, "Failed to serialize message"),
            Self::DeserializationError => write!(f, "Failed to deserialize message"),
            Self::InvalidFormat => write!(f, "Invalid message format"),
        }
    }
}

impl std::error::Error for MessageError {}

/// Serializes a client message to JSON bytes
pub fn serialize_client_message(message: &ClientMessage) -> Result<Vec<u8>, MessageError> {
    serde_json::to_vec(message).map_err(|_| MessageError::SerializationError)
}

/// Deserializes a client message from JSON bytes
pub fn deserialize_client_message(data: &[u8]) -> Result<ClientMessage, MessageError> {
    serde_json::from_slice(data).map_err(|_| MessageError::DeserializationError)
}

/// Serializes a server message to JSON bytes
pub fn serialize_server_message(message: &ServerMessage) -> Result<Vec<u8>, MessageError> {
    serde_json::to_vec(message).map_err(|_| MessageError::SerializationError)
}

/// Deserializes a server message from JSON bytes
pub fn deserialize_server_message(data: &[u8]) -> Result<ServerMessage, MessageError> {
    serde_json::from_slice(data).map_err(|_| MessageError::DeserializationError)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // NetAddress Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_net_address_new() {
        let addr = NetAddress::new("127.0.0.1", 8080);
        assert_eq!(addr.ip, "127.0.0.1");
        assert_eq!(addr.port, 8080);
    }

    // ------------------------------------------------------------------------
    // ConnectMessage Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_connect_message_new() {
        let msg = ConnectMessage::new("TestPlayer", "1.0.0");
        assert!(msg.player_id.is_none());
        assert_eq!(msg.player_name, "TestPlayer");
        assert_eq!(msg.version, "1.0.0");
        assert!(msg.auth_token.is_none());
        assert!(msg.room_id.is_none());
    }

    #[test]
    fn test_connect_message_with_player_id() {
        let msg = ConnectMessage::new("TestPlayer", "1.0.0")
            .with_player_id(PlayerId::new(123));
        assert_eq!(msg.player_id, Some(PlayerId::new(123)));
    }

    #[test]
    fn test_connect_message_with_auth_token() {
        let msg = ConnectMessage::new("TestPlayer", "1.0.0")
            .with_auth_token("token123");
        assert_eq!(msg.auth_token, Some(String::from("token123")));
    }

    #[test]
    fn test_connect_message_with_room_id() {
        let msg = ConnectMessage::new("TestPlayer", "1.0.0")
            .with_room_id("room456");
        assert_eq!(msg.room_id, Some(String::from("room456")));
    }

    // ------------------------------------------------------------------------
    // PlayerInputMessage Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_player_input_message_new() {
        let input = PlayerInput::new(1);
        let msg = PlayerInputMessage::new(input.clone(), 1000);
        assert_eq!(msg.input.sequence, 1);
        assert_eq!(msg.client_timestamp, 1000);
    }

    // ------------------------------------------------------------------------
    // PingMessage Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_ping_message_new() {
        let msg = PingMessage::new(5, 12345);
        assert_eq!(msg.sequence, 5);
        assert_eq!(msg.timestamp, 12345);
    }

    // ------------------------------------------------------------------------
    // ConnectResultMessage Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_connect_result_success() {
        let result = ConnectResultMessage::success(
            PlayerId::new(1),
            EntityId::new(100),
            [10.0, 20.0, 30.0],
            5000,
        );
        assert!(result.success);
        assert_eq!(result.player_id, Some(PlayerId::new(1)));
        assert_eq!(result.entity_id, Some(EntityId::new(100)));
        assert_eq!(result.spawn_position, Some([10.0, 20.0, 30.0]));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_connect_result_failure() {
        let result = ConnectResultMessage::failure("Server full", 5000);
        assert!(!result.success);
        assert_eq!(result.error, Some(String::from("Server full")));
        assert!(result.player_id.is_none());
    }

    // ------------------------------------------------------------------------
    // WorldUpdateMessage Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_world_update_new() {
        let update = WorldUpdateMessage::new(1000, 5);
        assert_eq!(update.server_timestamp, 1000);
        assert_eq!(update.last_ack, 5);
        assert!(update.entities.is_empty());
        assert!(update.removed_entities.is_empty());
    }

    #[test]
    fn test_world_update_add_entity() {
        let mut update = WorldUpdateMessage::new(1000, 5);
        let entity_update = EntityUpdate::new(
            EntityId::new(1),
            EntityType::Player,
            TransformState::identity(),
        );
        update.add_entity(entity_update);
        assert_eq!(update.entities.len(), 1);
    }

    #[test]
    fn test_world_update_remove_entity() {
        let mut update = WorldUpdateMessage::new(1000, 5);
        update.remove_entity(EntityId::new(1));
        assert_eq!(update.removed_entities.len(), 1);
    }

    #[test]
    fn test_world_update_has_changes() {
        let mut update = WorldUpdateMessage::new(1000, 5);
        assert!(!update.has_changes());

        update.remove_entity(EntityId::new(1));
        assert!(update.has_changes());
    }

    // ------------------------------------------------------------------------
    // PongMessage Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_pong_message_new() {
        let pong = PongMessage::new(1, 1000, 1500);
        assert_eq!(pong.ping_sequence, 1);
        assert_eq!(pong.ping_timestamp, 1000);
        assert_eq!(pong.pong_timestamp, 1500);
    }

    #[test]
    fn test_pong_message_rtt() {
        let pong = PongMessage::new(1, 1000, 1500);
        assert_eq!(pong.rtt(), 500);
    }

    // ------------------------------------------------------------------------
    // EntityUpdate Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_entity_update_new() {
        let update = EntityUpdate::new(
            EntityId::new(1),
            EntityType::Player,
            TransformState::identity(),
        );
        assert_eq!(update.entity_id, EntityId::new(1));
        assert_eq!(update.entity_type, EntityType::Player);
        assert_eq!(update.transform, TransformState::identity());
        assert_eq!(update.velocity, [0.0, 0.0, 0.0]);
        assert!(update.data.is_none());
    }

    #[test]
    fn test_entity_update_builder() {
        let update = EntityUpdate::new(
            EntityId::new(1),
            EntityType::Player,
            TransformState::identity(),
        )
        .with_velocity([1.0, 2.0, 3.0])
        .with_sequence(10);

        assert_eq!(update.velocity, [1.0, 2.0, 3.0]);
        assert_eq!(update.sequence, 10);
    }

    // ------------------------------------------------------------------------
    // PlayerEntityData Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_player_entity_data_new() {
        let data = PlayerEntityData::new(PlayerId::new(1), "TestPlayer");
        assert_eq!(data.player_id, PlayerId::new(1));
        assert_eq!(data.name, "TestPlayer");
        assert_eq!(data.health, 100);
        assert_eq!(data.max_health, 100);
        assert_eq!(data.level, 1);
    }

    #[test]
    fn test_player_entity_data_builder() {
        let data = PlayerEntityData::new(PlayerId::new(1), "TestPlayer")
            .with_health(80, 100)
            .with_level(5);

        assert_eq!(data.health, 80);
        assert_eq!(data.max_health, 100);
        assert_eq!(data.level, 5);
    }

    // ------------------------------------------------------------------------
    // MonsterEntityData Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_monster_entity_data_new() {
        let data = MonsterEntityData::new(42);
        assert_eq!(data.monster_type, 42);
        assert_eq!(data.health, 100);
        assert!(data.target.is_none());
    }

    #[test]
    fn test_monster_entity_data_builder() {
        let data = MonsterEntityData::new(42)
            .with_health(50, 100)
            .with_target(EntityId::new(1));

        assert_eq!(data.health, 50);
        assert_eq!(data.target, Some(EntityId::new(1)));
    }

    // ------------------------------------------------------------------------
    // Message Serialization Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_serialize_client_message_connect() {
        let connect = ConnectMessage::new("Player1", "1.0.0");
        let message = ClientMessage::Connect(connect);
        let serialized = serialize_client_message(&message).expect("Serialization failed");
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_deserialize_client_message_connect() {
        let connect = ConnectMessage::new("Player1", "1.0.0");
        let message = ClientMessage::Connect(connect);
        let serialized = serialize_client_message(&message).expect("Serialization failed");
        let deserialized = deserialize_client_message(&serialized).expect("Deserialization failed");

        match deserialized {
            ClientMessage::Connect(msg) => {
                assert_eq!(msg.player_name, "Player1");
                assert_eq!(msg.version, "1.0.0");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_serialize_client_message_ping() {
        let ping = PingMessage::new(1, 1000);
        let message = ClientMessage::Ping(ping);
        let serialized = serialize_client_message(&message).expect("Serialization failed");
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_deserialize_client_message_ping() {
        let ping = PingMessage::new(1, 1000);
        let message = ClientMessage::Ping(ping);
        let serialized = serialize_client_message(&message).expect("Serialization failed");
        let deserialized = deserialize_client_message(&serialized).expect("Deserialization failed");

        match deserialized {
            ClientMessage::Ping(msg) => {
                assert_eq!(msg.sequence, 1);
                assert_eq!(msg.timestamp, 1000);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_serialize_server_message_connect_result() {
        let result = ConnectResultMessage::success(
            PlayerId::new(1),
            EntityId::new(100),
            [0.0, 0.0, 0.0],
            5000,
        );
        let message = ServerMessage::ConnectResult(result);
        let serialized = serialize_server_message(&message).expect("Serialization failed");
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_deserialize_server_message_connect_result() {
        let result = ConnectResultMessage::success(
            PlayerId::new(1),
            EntityId::new(100),
            [0.0, 0.0, 0.0],
            5000,
        );
        let message = ServerMessage::ConnectResult(result);
        let serialized = serialize_server_message(&message).expect("Serialization failed");
        let deserialized = deserialize_server_message(&serialized).expect("Deserialization failed");

        match deserialized {
            ServerMessage::ConnectResult(msg) => {
                assert!(msg.success);
                assert_eq!(msg.player_id, Some(PlayerId::new(1)));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_serialize_server_message_world_update() {
        let mut update = WorldUpdateMessage::new(1000, 5);
        update.add_entity(EntityUpdate::new(
            EntityId::new(1),
            EntityType::Player,
            TransformState::identity(),
        ));
        let message = ServerMessage::WorldUpdate(update);
        let serialized = serialize_server_message(&message).expect("Serialization failed");
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_deserialize_server_message_world_update() {
        let mut update = WorldUpdateMessage::new(1000, 5);
        update.add_entity(EntityUpdate::new(
            EntityId::new(1),
            EntityType::Player,
            TransformState::identity(),
        ));
        let message = ServerMessage::WorldUpdate(update);
        let serialized = serialize_server_message(&message).expect("Serialization failed");
        let deserialized = deserialize_server_message(&serialized).expect("Deserialization failed");

        match deserialized {
            ServerMessage::WorldUpdate(msg) => {
                assert_eq!(msg.server_timestamp, 1000);
                assert_eq!(msg.entities.len(), 1);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_serialize_server_message_pong() {
        let pong = PongMessage::new(1, 1000, 1500);
        let message = ServerMessage::Pong(pong);
        let serialized = serialize_server_message(&message).expect("Serialization failed");
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_deserialize_server_message_pong() {
        let pong = PongMessage::new(1, 1000, 1500);
        let message = ServerMessage::Pong(pong);
        let serialized = serialize_server_message(&message).expect("Serialization failed");
        let deserialized = deserialize_server_message(&serialized).expect("Deserialization failed");

        match deserialized {
            ServerMessage::Pong(msg) => {
                assert_eq!(msg.ping_sequence, 1);
                assert_eq!(msg.rtt(), 500);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_serialization_roundtrip_client_message() {
        let original = ClientMessage::Connect(
            ConnectMessage::new("TestPlayer", "1.0.0")
                .with_player_id(PlayerId::new(123))
        );

        let serialized = serialize_client_message(&original).expect("Serialization failed");
        let deserialized = deserialize_client_message(&serialized).expect("Deserialization failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serialization_roundtrip_server_message() {
        let mut update = WorldUpdateMessage::new(1000, 5);
        let entity_update = EntityUpdate::new(
            EntityId::new(1),
            EntityType::Player,
            TransformState::new([10.0, 20.0, 30.0], [0.0, 1.5, 0.0], 1.0),
        ).with_data(EntityData::Player(
            PlayerEntityData::new(PlayerId::new(100), "PlayerName")
                .with_health(75, 100)
        ));
        update.add_entity(entity_update);

        let original = ServerMessage::WorldUpdate(update);

        let serialized = serialize_server_message(&original).expect("Serialization failed");
        let deserialized = deserialize_server_message(&serialized).expect("Deserialization failed");

        match (original, deserialized) {
            (ServerMessage::WorldUpdate(orig), ServerMessage::WorldUpdate(de)) => {
                assert_eq!(orig.server_timestamp, de.server_timestamp);
                assert_eq!(orig.entities.len(), de.entities.len());
                assert_eq!(orig.entities[0].entity_id, de.entities[0].entity_id);
            }
            _ => panic!("Message type mismatch"),
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        let invalid_data = b"not valid json";
        let result = deserialize_client_message(invalid_data);
        assert!(matches!(result, Err(MessageError::DeserializationError)));
    }
}
