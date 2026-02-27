// crates/server/src/network.rs

use trueworld_core::net::ClientMessage;

pub enum NetworkPacket {
    ClientConnected { client_id: u64 },
    ClientDisconnected { client_id: u64 },
    Message { client_id: u64, message: ClientMessage },
}

pub struct ServerNetwork;
