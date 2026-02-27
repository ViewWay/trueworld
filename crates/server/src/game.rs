// crates/server/src/game.rs

use trueworld_core::{PlayerId, PlayerInput, net::WorldUpdateMessage};

pub struct GameWorld {
    _players: std::collections::HashMap<PlayerId, PlayerInput>,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            _players: std::collections::HashMap::new(),
        }
    }

    pub fn update(&mut self, _tick: u64) {
        // TODO: Implement game world update
    }

    pub fn create_update_packet(&self, tick: u64) -> trueworld_core::net::ServerMessage {
        let update = WorldUpdateMessage::new(tick, 0);
        trueworld_core::net::ServerMessage::WorldUpdate(update)
    }

    pub fn set_player_input(&mut self, _player_id: PlayerId, _input: PlayerInput) {
        // TODO: Implement
    }
}
