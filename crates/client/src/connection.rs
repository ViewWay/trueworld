// crates/client/src/connection.rs
//
// Connection management system for TrueWorld client.
// Handles initiating connection to server and the connection handshake.

#![allow(dead_code)]

use bevy::prelude::*;
use renet::{RenetClient, ConnectionConfig, DefaultChannel};
use renet_netcode::{NetcodeClientTransport, ClientAuthentication};
use std::net::UdpSocket;
use std::time::SystemTime;

use trueworld_core::net::{ClientMessage, ConnectMessage};

use crate::network::{NetworkResource, NetworkQueue};

/// Connection system - initiates connection when app starts
pub fn initiate_connection(
    mut network_res: ResMut<NetworkResource>,
    mut queue: ResMut<NetworkQueue>,
) {
    // Only initiate if we don't have a client yet
    if network_res.client.is_some() {
        return;
    }

    info!("Initiating connection to server at {}:{}...",
        network_res.server_addr, network_res.server_port);

    // Create Renet client
    let connection_config = ConnectionConfig {
        server_channels_config: DefaultChannel::config(),
        client_channels_config: DefaultChannel::config(),
        ..Default::default()
    };

    let client = RenetClient::new(connection_config);

    // Create UDP socket for transport
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");

    // Get current time
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // Create authentication for unsecure connection
    let authentication = ClientAuthentication::Unsecure {
        protocol_id: 1, // PROTOCOL_VERSION
        client_id: 0,   // Will be assigned by server
        server_addr: format!("{}:{}", network_res.server_addr, network_res.server_port)
            .parse().expect("Invalid server address"),
        user_data: None,
    };

    // Create transport
    let transport = NetcodeClientTransport::new(
        current_time,
        authentication,
        socket,
    ).expect("Failed to create transport");

    // Store client and transport
    network_res.client = Some(client);
    network_res.transport = Some(transport);

    // Send connection message
    let connect_msg = ClientMessage::Connect(ConnectMessage {
        player_id: None,
        player_name: "Player".to_string(), // TODO: Get from user input
        auth_token: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        room_id: None,
    });

    queue.outgoing.push(connect_msg);

    info!("Connection initiated, waiting for server response...");
}

/// Plugin for connection management
pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initiate_connection);
    }
}
