// crates/client/src/network.rs
//
// Network plugin for the TrueWorld client.
// Handles connection to server, packet sending/receiving, and network statistics.

#![allow(dead_code)]

use bevy::prelude::*;
use renet::RenetClient;
use renet_netcode::{NetcodeClientTransport, NetcodeTransportError};

use trueworld_protocol::{serialize_packet, deserialize_packet};
use trueworld_core::{PlayerId, EntityId, net::{ClientMessage, ServerMessage}};

use crate::state::NetworkStats;

/// Network plugin
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (
                handle_connection,
                receive_packets,
            ).chain())
            .add_systems(PostUpdate, (send_packets, update_network_stats).chain())
            .init_resource::<NetworkResource>()
            .init_resource::<NetworkQueue>()
            .init_resource::<NetworkStats>()
            .add_event::<ConnectionResult>()
            .add_event::<PlayerSpawnEvent>()
            .add_event::<EntitySpawnEvent>()
            .add_event::<EntityDespawnEvent>()
            .add_event::<WorldUpdateEvent>()
            .add_event::<PositionAckEvent>()
            .add_event::<PositionCorrectionEvent>();
    }
}

/// Network resource containing client and transport
#[derive(Resource)]
pub struct NetworkResource {
    /// Renet client for packet handling
    pub client: Option<RenetClient>,
    /// Netcode transport for connection handling
    pub transport: Option<NetcodeClientTransport>,
    /// Server address
    pub server_addr: String,
    /// Server port
    pub server_port: u16,
    /// Current connection ID
    pub current_connection_id: Option<u64>,
    /// Local player ID (assigned by server)
    pub local_player_id: Option<PlayerId>,
    /// Local entity ID (assigned by server)
    pub local_entity_id: Option<EntityId>,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            client: None,
            transport: None,
            server_addr: "127.0.0.1".to_string(),
            server_port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5000),
            current_connection_id: None,
            local_player_id: None,
            local_entity_id: None,
        }
    }
}

impl NetworkResource {
    /// Check if connected to server
    pub fn is_connected(&self) -> bool {
        self.client.is_some() && self.local_player_id.is_some()
    }

    /// Get the local player ID
    pub fn player_id(&self) -> Option<PlayerId> {
        self.local_player_id
    }

    /// Get the local entity ID
    pub fn entity_id(&self) -> Option<EntityId> {
        self.local_entity_id
    }
}

/// Network queue for outgoing/incoming messages
#[derive(Resource, Default)]
pub struct NetworkQueue {
    /// Outgoing messages to server
    pub outgoing: Vec<ClientMessage>,
    /// Incoming server messages (processed by systems)
    pub incoming_server: Vec<ServerMessage>,
}

/// Connection result event
#[derive(Event, Debug, Clone)]
pub struct ConnectionResult {
    /// Whether connection succeeded
    pub success: bool,
    /// Player ID (if successful)
    pub player_id: Option<PlayerId>,
    /// Entity ID (if successful)
    pub entity_id: Option<EntityId>,
    /// Initial spawn position (if successful)
    pub spawn_position: Option<[f32; 3]>,
    /// Reason for failure (if any)
    pub reason: Option<String>,
}

/// Player spawn event - when a new player enters the game
#[derive(Event, Debug, Clone)]
pub struct PlayerSpawnEvent {
    pub player_id: PlayerId,
    pub entity_id: EntityId,
    pub username: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

/// Entity spawn event - when a game entity is created
#[derive(Event, Debug, Clone)]
pub struct EntitySpawnEvent {
    pub entity_id: EntityId,
    pub entity_type: u8,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub data: Vec<u8>,
}

/// Entity despawn event
#[derive(Event, Debug, Clone)]
pub struct EntityDespawnEvent {
    pub entity_id: EntityId,
}

/// World update event - batched entity state updates
#[derive(Event, Debug, Clone)]
pub struct WorldUpdateEvent {
    pub tick: u64,
    pub updates: Vec<EntityStateUpdate>,
}

/// Position acknowledgment event - server confirms player position
#[derive(Event, Debug, Clone)]
pub struct PositionAckEvent {
    pub player_id: PlayerId,
    pub ack_sequence: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub server_time: u64,
}

/// Position correction event - server forcibly corrects position
#[derive(Event, Debug, Clone)]
pub struct PositionCorrectionEvent {
    pub player_id: PlayerId,
    pub correct_position: [f32; 3],
    pub reason: trueworld_core::net::CorrectionReason,
}

/// Individual entity state update
#[derive(Debug, Clone)]
pub struct EntityStateUpdate {
    pub entity_id: EntityId,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
}

/// Handle connection state and transport updates
fn handle_connection(
    _commands: Commands,
    mut network_res: ResMut<NetworkResource>,
    mut connection_events: EventWriter<ConnectionResult>,
    time: Res<Time<Real>>,
) {
    let delta_duration = time.delta();

    // Check if transport exists
    if network_res.transport.is_none() || network_res.client.is_none() {
        return;
    }

    // Track if we need to send a disconnect event
    let mut disconnect_reason: Option<String> = None;

    // Update transport with client
    let mut client = network_res.client.take().unwrap();
    let transport_err = network_res.transport.as_mut().and_then(|t| {
        t.update(delta_duration, &mut client).err()
    });

    // Check for disconnect
    if let Some(reason) = client.disconnect_reason() {
        disconnect_reason = Some(format!("{:?}", reason));
    }

    // Put client back
    network_res.client = Some(client);

    // Handle transport errors
    if let Some(e) = transport_err {
        match e {
            NetcodeTransportError::Renet(err) => {
                warn!("Renet disconnect: {}", err);
                disconnect_reason = Some(format!("Renet: {}", err));
            }
            NetcodeTransportError::Netcode(err) => {
                warn!("Netcode error: {}", err);
            }
            NetcodeTransportError::IO(err) => {
                warn!("IO error: {}", err);
            }
        }
    }

    // Send disconnect event if needed
    if disconnect_reason.is_some() {
        network_res.local_player_id = None;
        network_res.local_entity_id = None;
        connection_events.send(ConnectionResult {
            success: false,
            player_id: None,
            entity_id: None,
            spawn_position: None,
            reason: disconnect_reason,
        });
    }

    // Update client (now we can have a single mutable reference)
    if let Some(client) = &mut network_res.client {
        client.update(delta_duration);
    }
}

/// Receive packets from server
fn receive_packets(
    mut network_res: ResMut<NetworkResource>,
    mut queue: ResMut<NetworkQueue>,
    mut connection_events: EventWriter<ConnectionResult>,
    _player_spawn_events: EventWriter<PlayerSpawnEvent>,
    _entity_spawn_events: EventWriter<EntitySpawnEvent>,
    mut entity_despawn_events: EventWriter<EntityDespawnEvent>,
    mut world_update_events: EventWriter<WorldUpdateEvent>,
    mut position_ack_events: EventWriter<PositionAckEvent>,
    mut position_correction_events: EventWriter<PositionCorrectionEvent>,
) {
    let client = match &mut network_res.client {
        Some(c) => c,
        None => return,
    };

    // Receive messages from channel 0 (Reliable Ordered)
    while let Some(message) = client.receive_message(0) {
        match deserialize_packet::<ServerMessage>(&message) {
            Ok(server_msg) => {
                queue.incoming_server.push(server_msg);
            }
            Err(e) => {
                warn!("Failed to deserialize server message: {:?}", e);
            }
        }
    }

    // Receive messages from channel 1 (Reliable Unordered)
    while let Some(message) = client.receive_message(1) {
        match deserialize_packet::<ServerMessage>(&message) {
            Ok(server_msg) => {
                queue.incoming_server.push(server_msg);
            }
            Err(e) => {
                warn!("Failed to deserialize server message: {:?}", e);
            }
        }
    }

    // Receive messages from channel 2 (Unreliable - PositionAck)
    while let Some(message) = client.receive_message(2) {
        match deserialize_packet::<ServerMessage>(&message) {
            Ok(server_msg) => {
                queue.incoming_server.push(server_msg);
            }
            Err(e) => {
                warn!("Failed to deserialize server message: {:?}", e);
            }
        }
    }

    // Process incoming messages
    for msg in queue.incoming_server.drain(..) {
        match msg {
            ServerMessage::ConnectResult(result) => {
                if result.success {
                    network_res.local_player_id = result.player_id;
                    network_res.local_entity_id = result.entity_id;

                    info!("Connected to server! Player ID: {:?}", result.player_id);

                    connection_events.send(ConnectionResult {
                        success: true,
                        player_id: result.player_id,
                        entity_id: result.entity_id,
                        spawn_position: result.spawn_position,
                        reason: None,
                    });
                } else {
                    warn!("Connection failed: {:?}", result.error);
                    connection_events.send(ConnectionResult {
                        success: false,
                        player_id: None,
                        entity_id: None,
                        spawn_position: None,
                        reason: result.error,
                    });
                }
            }
            ServerMessage::WorldUpdate(update) => {
                // Convert WorldUpdateMessage to WorldUpdateEvent
                let entity_updates: Vec<EntityStateUpdate> = update.entities
                    .into_iter()
                    .map(|e| EntityStateUpdate {
                        entity_id: e.entity_id,
                        position: e.transform.position,
                        rotation: [
                            e.transform.rotation[0],
                            e.transform.rotation[1],
                            e.transform.rotation[2],
                            1.0, // quaternion w
                        ],
                        velocity: e.velocity,
                    })
                    .collect();

                // Handle despawned entities
                for entity_id in update.removed_entities {
                    entity_despawn_events.send(EntityDespawnEvent { entity_id });
                }

                if !entity_updates.is_empty() {
                    world_update_events.send(WorldUpdateEvent {
                        tick: update.server_timestamp,
                        updates: entity_updates,
                    });
                }
            }
            ServerMessage::Pong(pong) => {
                info!("Received pong: sequence={}, rtt={}ms", pong.ping_sequence, pong.rtt());
            }
            ServerMessage::PositionAck(ack) => {
                // Server acknowledgment of player position
                debug!("PositionAck: player_id={}, seq={}, pos={:?}",
                    ack.player_id, ack.ack_sequence, ack.position);

                // Send event for movement system to handle
                position_ack_events.send(PositionAckEvent {
                    player_id: ack.player_id,
                    ack_sequence: ack.ack_sequence,
                    position: ack.position,
                    velocity: ack.velocity,
                    server_time: ack.server_time,
                });
            }
            ServerMessage::PositionCorrection(correction) => {
                // Server forcibly correcting position (anti-cheat or collision)
                info!("PositionCorrection: player_id={}, pos={:?}, reason={:?}",
                    correction.player_id, correction.correct_position, correction.reason);

                // Send event for movement system to handle
                position_correction_events.send(PositionCorrectionEvent {
                    player_id: correction.player_id,
                    correct_position: correction.correct_position,
                    reason: correction.reason,
                });
            }
        }
    }
}

/// Send packets to server
fn send_packets(mut network_res: ResMut<NetworkResource>, mut queue: ResMut<NetworkQueue>) {
    let client = match &mut network_res.client {
        Some(c) => c,
        None => return,
    };

    for msg in queue.outgoing.drain(..) {
        match serialize_packet(&msg) {
            Ok(bytes) => {
                // Send on channel 0 for reliable ordered delivery
                client.send_message(0, bytes);
            }
            Err(e) => {
                warn!("Failed to serialize message: {:?}", e);
            }
        }
    }
}

/// Update network statistics
fn update_network_stats(mut stats: ResMut<NetworkStats>, network_res: Res<NetworkResource>, time: Res<Time<Real>>) {
    stats.last_update = time.elapsed_secs() as f64;

    if let Some(client) = &network_res.client {
        let info = client.network_info();
        stats.update_ping(info.rtt as f32);
        stats.packet_loss = info.packet_loss as f32;
        // Renet 1.2 doesn't provide total bytes/messages, only rate per second
        // We'll track these in NetworkStats separately if needed
    }
}
