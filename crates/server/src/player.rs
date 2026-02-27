// crates/server/src/player.rs

use trueworld_core::PlayerId;

pub struct Player {
    pub session: PlayerSession,
}

pub struct PlayerSession {
    pub client_id: u64,
    pub player_id: Option<PlayerId>,
    pub room_id: Option<String>,
    pub authenticated: bool,
}
