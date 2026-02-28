// crates/client/src/network.rs
//
// Network plugin for the TrueWorld client.
// Handles connection to server, packet sending/receiving, and network statistics.

use bevy::prelude::*;
use renet::RenetClient;
use renet_netcode::{NetcodeClientTransport, NetcodeTransportError};

use crate::state::NetworkStats;

/// Network plugin
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, handle_connection)
            .add_systems(PostUpdate, (send_packets, update_network_stats).chain())
            .init_resource::<NetworkQueue>()
            .init_resource::<NetworkStats>();
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
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            client: None,
            transport: None,
            server_addr: "127.0.0.1".to_string(),
            server_port: 5000,
            current_connection_id: None,
        }
    }
}

/// Network queue for outgoing/incoming messages
#[derive(Resource, Default)]
pub struct NetworkQueue {
    /// Outgoing messages to server
    pub outgoing: Vec<ServerMessage>,
    /// Incoming messages from server
    pub incoming: Vec<ClientMessage>,
}

/// Connection result event
#[derive(Event)]
pub struct ConnectionResult {
    /// Whether connection succeeded
    pub success: bool,
    /// Player ID (if successful)
    pub player_id: Option<PlayerId>,
    /// Reason for failure (if any)
    pub reason: Option<String>,
}

// Placeholder types for messages
// TODO: Import from protocol crate when fully implemented
type ServerMessage = ();
type ClientMessage = ();
type PlayerId = u64;

/// Handle connection state
fn handle_connection(
    _commands: Commands,
    mut network_res: ResMut<NetworkResource>,
    _connection_events: EventWriter<ConnectionResult>,
    time: Res<Time<Real>>,
) {
    // In renet_netcode 1.2, update takes a Duration
    let delta_duration = time.delta();

    // Check if transport exists
    if network_res.transport.is_none() || network_res.client.is_none() {
        return;
    }

    // Update transport with client - needs to be done carefully
    // We take the client out temporarily, update transport, then put client back
    let mut client = network_res.client.take().unwrap();
    let transport_err = network_res.transport.as_mut().and_then(|t| {
        t.update(delta_duration, &mut client).err()
    });
    network_res.client = Some(client);

    if let Some(e) = transport_err {
        match e {
            NetcodeTransportError::Renet(_) => {
                warn!("Renet disconnect");
            }
            NetcodeTransportError::Netcode(_) => {
                warn!("Netcode error");
            }
            NetcodeTransportError::IO(_) => {
                warn!("IO error");
            }
        }
    }

    // Update client (now we can have a single mutable reference)
    if let Some(client) = &mut network_res.client {
        client.update(delta_duration);
    }

    let delta_secs = delta_duration.as_secs_f64();
    info!("Network update: delta={:.3}s", delta_secs);
}

/// Send packets to server
fn send_packets(mut network_res: ResMut<NetworkResource>, mut queue: ResMut<NetworkQueue>) {
    let client = match &mut network_res.client {
        Some(c) => c,
        None => return,
    };

    for msg in queue.outgoing.drain(..) {
        // TODO: Serialize and send message
        let _ = msg;
        let _ = client; // Use client when implemented
    }
}

/// Update network statistics
fn update_network_stats(mut stats: ResMut<NetworkStats>, _network_res: Res<NetworkResource>, time: Res<Time<Real>>) {
    stats.last_update = time.elapsed_secs() as f64;
}
