// crates/server/src/database.rs

use anyhow::Result;

pub struct DatabaseManager;

impl DatabaseManager {
    pub async fn new(_config: &crate::config::DatabaseConfig) -> Result<Self> {
        Ok(Self)
    }

    pub async fn load_player_by_token(&self, _token: &str) -> Result<Option<PlayerData>> {
        Ok(None)
    }
}

pub struct PlayerData;
