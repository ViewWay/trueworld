// crates/server/src/room.rs

use trueworld_core::PlayerId;
use std::collections::HashMap;

pub struct RoomManager {
    max_rooms: usize,
    max_players_per_room: usize,
    rooms: HashMap<String, Room>,
}

pub struct Room {
    _id: String,
    _players: Vec<PlayerId>,
}

impl RoomManager {
    pub fn new(max_rooms: usize, max_players_per_room: usize) -> Self {
        Self {
            max_rooms,
            max_players_per_room,
            rooms: HashMap::new(),
        }
    }

    pub fn remove_player(&mut self, _player_id: &PlayerId) {
        // TODO: Implement
    }
}
