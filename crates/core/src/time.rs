// Time types for TrueWorld

use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

/// Game time representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameTime(u64);

impl GameTime {
    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub fn as_millis(self) -> u64 {
        self.0
    }
}

/// Duration type
pub type Duration = StdDuration;

/// Timestamp type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn from_secs(secs: i64) -> Self {
        Self(secs)
    }

    pub fn as_secs(self) -> i64 {
        self.0
    }

    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp())
    }
}
