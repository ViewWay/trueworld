// crates/server/src/network.rs
//
// Network handling for TrueWorld server.
// Manages client connections, message routing, and packet broadcasting.

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::net::UdpSocket;
use std::time::Duration;
use renet::{RenetServer, ConnectionConfig, DefaultChannel, ServerEvent};

use trueworld_core::{
    PlayerId, EntityId,
    net::{ClientMessage, ServerMessage, ConnectMessage, ConnectResultMessage},
};
use trueworld_protocol::{serialize_packet, deserialize_packet};

use super::entity::EntityManager;

/// Default server port
pub const DEFAULT_SERVER_PORT: u16 = 5000;

/// Maximum number of clients
pub const MAX_CLIENTS: usize = 64;

/// Server tick rate (updates per second)
pub const SERVER_TICK_RATE: u64 = 60;

/// Network manager for the TrueWorld server
pub struct ServerNetwork {
    /// Renet server instance
    server: RenetServer,

    /// Socket for direct access
    socket: UdpSocket,

    /// Connected clients mapped from client_id to PlayerId
    clients: HashMap<u64, ConnectedClient>,

    /// Pending outgoing messages per client
    outgoing: HashMap<u64, VecDeque<ServerMessage>>,

    /// Current server time
    server_time: u64,

    /// Next player ID to assign
    next_player_id: u64,
}

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ConnectedClient {
    /// Client's network ID
    pub client_id: u64,

    /// Assigned player ID
    pub player_id: PlayerId,

    /// Player username
    pub username: String,

    /// Assigned entity ID
    pub entity_id: Option<EntityId>,

    /// Connection timestamp
    pub connected_at: u64,

    /// Last activity timestamp
    pub last_activity: u64,
}

impl ServerNetwork {
    /// Creates a new server network instance
    pub fn new(port: u16) -> anyhow::Result<Self> {
        let server_addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(server_addr)?;

        // Create connection config with channels
        let connection_config = ConnectionConfig {
            server_channels_config: DefaultChannel::config(),
            client_channels_config: DefaultChannel::config(),
            ..Default::default()
        };

        let server = RenetServer::new(connection_config);

        Ok(Self {
            server,
            socket,
            clients: HashMap::new(),
            outgoing: HashMap::new(),
            server_time: 0,
            next_player_id: 1,
        })
    }

    /// Returns the server's socket address
    pub fn local_addr(&self) -> anyhow::Result<std::net::SocketAddr> {
        Ok(self.socket.local_addr()?)
    }

    /// Returns the number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Returns all connected clients
    pub fn clients_iter(&self) -> impl Iterator<Item = &ConnectedClient> {
        self.clients.values()
    }

    /// Gets a client by client ID
    pub fn get_client(&self, client_id: u64) -> Option<&ConnectedClient> {
        self.clients.get(&client_id)
    }

    /// Gets the player ID for a client
    pub fn get_player_id(&self, client_id: u64) -> Option<PlayerId> {
        self.clients.get(&client_id).map(|c| c.player_id)
    }

    /// Gets the client ID for a player
    pub fn get_client_id(&self, player_id: PlayerId) -> Option<u64> {
        self.clients.values()
            .find(|c| c.player_id == player_id)
            .map(|c| c.client_id)
    }

    /// Updates the network (call this each frame)
    pub fn update(&mut self, delta_time: Duration) -> Vec<NetworkEvent> {
        self.server_time = self.server_time.saturating_add(delta_time.as_secs());
        let mut events = Vec::new();

        // Update server
        self.server.update(delta_time);

        // Process server events
        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    tracing::info!("Client connected: {}", client_id);
                    events.push(NetworkEvent::ClientConnected { client_id });
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    tracing::info!("Client disconnected: {} - {:?}", client_id, reason);
                    if let Some(client) = self.clients.remove(&client_id) {
                        events.push(NetworkEvent::ClientDisconnected {
                            client_id,
                            player_id: Some(client.player_id),
                        });
                    } else {
                        events.push(NetworkEvent::ClientDisconnected {
                            client_id,
                            player_id: None,
                        });
                    }
                }
            }
        }

        // Receive messages from all clients
        for client_id in self.server.clients_id() {
            // Channel 0: Reliable ordered
            while let Some(message) = self.server.receive_message(client_id, 0) {
                if let Ok(client_msg) = deserialize_packet::<ClientMessage>(&message) {
                    events.push(NetworkEvent::Message {
                        client_id,
                        message: client_msg,
                    });
                }
            }

            // Channel 1: Reliable unordered
            while let Some(message) = self.server.receive_message(client_id, 1) {
                if let Ok(client_msg) = deserialize_packet::<ClientMessage>(&message) {
                    events.push(NetworkEvent::Message {
                        client_id,
                        message: client_msg,
                    });
                }
            }
        }

        // Send queued messages
        self.send_queued_messages();

        events
    }

    /// Sends a message to a specific client
    pub fn send_to_client(&mut self, client_id: u64, message: ServerMessage) {
        self.outgoing
            .entry(client_id)
            .or_insert_with(VecDeque::new)
            .push_back(message);
    }

    /// Broadcasts a message to all connected clients
    pub fn broadcast(&mut self, message: ServerMessage) {
        let client_ids: Vec<_> = self.clients.keys().copied().collect();
        for client_id in client_ids {
            self.send_to_client(client_id, message.clone());
        }
    }

    /// Broadcasts a message to all clients except one
    pub fn broadcast_except(&mut self, exclude_client_id: u64, message: ServerMessage) {
        let client_ids: Vec<_> = self.clients.keys()
            .copied()
            .filter(|&id| id != exclude_client_id)
            .collect();
        for client_id in client_ids {
            self.send_to_client(client_id, message.clone());
        }
    }

    /// Handles a client connection (assigns player ID)
    pub fn handle_connection(&mut self, client_id: u64, connect_msg: ConnectMessage, entity_mgr: &mut EntityManager) -> ConnectResultMessage {
        // Check if username is taken
        let username_taken = self.clients.values()
            .any(|c| c.username == connect_msg.player_name);

        if username_taken {
            return ConnectResultMessage::failure("Username already taken", self.server_time);
        }

        // Assign player ID
        let player_id = PlayerId::new(self.next_player_id);
        self.next_player_id = self.next_player_id.wrapping_add(1);

        // Spawn player entity
        let spawn_pos = [100.0, 0.0, 200.0]; // Default spawn position
        let entity_id = entity_mgr.spawn_player(player_id, &connect_msg.player_name, spawn_pos);

        // Create connected client record
        let connected_client = ConnectedClient {
            client_id,
            player_id,
            username: connect_msg.player_name.clone(),
            entity_id: Some(entity_id),
            connected_at: self.server_time,
            last_activity: self.server_time,
        };

        self.clients.insert(client_id, connected_client);

        tracing::info!("Player '{}' connected as PlayerId={:?}, EntityId={:?}",
            connect_msg.player_name, player_id, entity_id);

        ConnectResultMessage::success(player_id, entity_id, spawn_pos, self.server_time)
    }

    /// Disconnects a client
    pub fn disconnect_client(&mut self, client_id: u64) {
        self.server.disconnect(client_id);
    }

    /// Sends all queued messages
    fn send_queued_messages(&mut self) {
        let mut empty_clients = Vec::new();

        for (&client_id, messages) in &mut self.outgoing {
            while let Some(message) = messages.pop_front() {
                match serialize_packet(&message) {
                    Ok(bytes) => {
                        self.server.send_message(client_id, 0, bytes);
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize message: {:?}", e);
                    }
                }
            }

            if messages.is_empty() {
                empty_clients.push(client_id);
            }
        }

        // Cleanup empty queues
        for client_id in empty_clients {
            self.outgoing.remove(&client_id);
        }
    }

    /// Cleanup empty outgoing queues
    pub fn cleanup(&mut self) {
        self.outgoing.retain(|_, messages| !messages.is_empty());
    }

    /// Gets current server time
    pub fn server_time(&self) -> u64 {
        self.server_time
    }
}

/// Network events from the server
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A new client connected
    ClientConnected {
        client_id: u64,
    },

    /// A client disconnected
    ClientDisconnected {
        client_id: u64,
        player_id: Option<PlayerId>,
    },

    /// Received a message from a client
    Message {
        client_id: u64,
        message: ClientMessage,
    },
}
