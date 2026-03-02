// crates/server/src/movement/config.rs
//
// Movement validation configuration for the TrueWorld server.
// Provides tuning parameters for anti-cheat and movement validation.

use trueworld_core::PlayerId;
use std::collections::HashMap;

/// Server-side movement validation configuration
#[derive(Debug, Clone)]
pub struct ServerMovementConfig {
    /// Maximum movement speed (units per second)
    pub max_speed: f32,

    /// Maximum sprint speed (units per second)
    pub max_sprint_speed: f32,

    /// Maximum jump velocity
    pub max_jump_velocity: f32,

    /// Maximum allowed position delta per frame (anti-teleport)
    pub max_delta_per_frame: f32,

    /// Position update broadcast rate (Hz)
    pub broadcast_rate: f32,

    /// Maximum time to wait for client input before marking as AFK
    pub afk_timeout_seconds: u64,

    /// Speed violation threshold (how many violations before kick)
    pub speed_violation_threshold: u32,

    /// Teleport violation threshold
    pub teleport_violation_threshold: u32,
}

impl Default for ServerMovementConfig {
    fn default() -> Self {
        Self {
            max_speed: 5.0,          // Walking speed
            max_sprint_speed: 8.0,   // Sprinting speed
            max_jump_velocity: 10.0, // Jump velocity
            max_delta_per_frame: 2.0, // Max movement per tick (at 60Hz)
            broadcast_rate: 20.0,     // 20Hz position updates
            afk_timeout_seconds: 300, // 5 minutes
            speed_violation_threshold: 10,
            teleport_violation_threshold: 3,
        }
    }
}

impl ServerMovementConfig {
    /// Creates a new config with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the maximum allowed speed based on state
    pub const fn max_allowed_speed(&self, sprinting: bool) -> f32 {
        if sprinting {
            self.max_sprint_speed
        } else {
            self.max_speed
        }
    }

    /// Returns the broadcast interval in seconds
    pub fn broadcast_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f32(1.0 / self.broadcast_rate)
    }
}

/// Violation tracking for a player
#[derive(Debug, Clone)]
pub struct PlayerViolationTracker {
    /// Speed violation count
    pub speed_violations: u32,

    /// Teleport violation count
    pub teleport_violations: u32,

    /// Collision violation count
    pub collision_violations: u32,

    /// Last violation time
    pub last_violation_time: Option<u64>,

    /// Whether player is marked for kick
    pub mark_for_kick: bool,
}

impl Default for PlayerViolationTracker {
    fn default() -> Self {
        Self {
            speed_violations: 0,
            teleport_violations: 0,
            collision_violations: 0,
            last_violation_time: None,
            mark_for_kick: false,
        }
    }
}

impl PlayerViolationTracker {
    /// Creates a new violation tracker
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a speed violation
    pub fn record_speed_violation(&mut self) -> bool {
        self.speed_violations += 1;
        self.last_violation_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
        false // Don't kick yet
    }

    /// Records a teleport violation
    pub fn record_teleport_violation(&mut self, threshold: u32) -> bool {
        self.teleport_violations += 1;
        self.last_violation_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        if self.teleport_violations >= threshold {
            self.mark_for_kick = true;
            true // Should kick
        } else {
            false
        }
    }

    /// Records a collision violation
    pub fn record_collision_violation(&mut self) {
        self.collision_violations += 1;
        self.last_violation_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
    }

    /// Resets violation counts (for admins or after legitimate teleport)
    pub fn reset(&mut self) {
        self.speed_violations = 0;
        self.teleport_violations = 0;
        self.collision_violations = 0;
        self.last_violation_time = None;
        self.mark_for_kick = false;
    }

    /// Returns true if violations should decay (time passed)
    pub fn should_decay_violations(&self, decay_seconds: u64) -> bool {
        if let Some(last_time) = self.last_violation_time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now.saturating_sub(last_time) > decay_seconds
        } else {
            false
        }
    }
}

/// Manager for all player violation trackers
#[derive(Debug, Default)]
pub struct ViolationManager {
    trackers: HashMap<PlayerId, PlayerViolationTracker>,
}

impl ViolationManager {
    /// Creates a new violation manager
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets or creates a tracker for a player
    pub fn get_tracker(&mut self, player_id: PlayerId) -> &mut PlayerViolationTracker {
        self.trackers.entry(player_id).or_insert_with(PlayerViolationTracker::new)
    }

    /// Gets a tracker if it exists
    #[must_use]
    pub fn get_tracker_ref(&self, player_id: PlayerId) -> Option<&PlayerViolationTracker> {
        self.trackers.get(&player_id)
    }

    /// Removes a tracker (player disconnected)
    pub fn remove_tracker(&mut self, player_id: PlayerId) {
        self.trackers.remove(&player_id);
    }

    /// Updates all trackers (decays old violations)
    pub fn update(&mut self, decay_seconds: u64) {
        for tracker in self.trackers.values_mut() {
            if tracker.should_decay_violations(decay_seconds) {
                // Decay violations
                tracker.speed_violations = tracker.speed_violations.saturating_sub(1);
                if tracker.teleport_violations > 0 {
                    tracker.teleport_violations = tracker.teleport_violations.saturating_sub(1);
                }
            }
        }
    }

    /// Returns the number of tracked players
    #[must_use]
    pub fn len(&self) -> usize {
        self.trackers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ServerMovementConfig::default();
        assert_eq!(config.max_speed, 5.0);
        assert_eq!(config.max_sprint_speed, 8.0);
        assert_eq!(config.max_jump_velocity, 10.0);
    }

    #[test]
    fn test_config_max_allowed_speed() {
        let config = ServerMovementConfig::default();
        assert_eq!(config.max_allowed_speed(false), 5.0);
        assert_eq!(config.max_allowed_speed(true), 8.0);
    }

    #[test]
    fn test_violation_tracker_new() {
        let tracker = PlayerViolationTracker::new();
        assert_eq!(tracker.speed_violations, 0);
        assert_eq!(tracker.teleport_violations, 0);
        assert!(!tracker.mark_for_kick);
    }

    #[test]
    fn test_violation_tracker_speed() {
        let mut tracker = PlayerViolationTracker::new();
        assert!(!tracker.record_speed_violation());
        assert_eq!(tracker.speed_violations, 1);
        assert!(!tracker.record_speed_violation());
        assert_eq!(tracker.speed_violations, 2);
    }

    #[test]
    fn test_violation_tracker_teleport_kick() {
        let mut tracker = PlayerViolationTracker::new();
        assert!(!tracker.record_teleport_violation(3));
        assert_eq!(tracker.teleport_violations, 1);
        assert!(!tracker.record_teleport_violation(3));
        assert_eq!(tracker.teleport_violations, 2);
        // Third violation triggers kick
        assert!(tracker.record_teleport_violation(3));
        assert!(tracker.mark_for_kick);
    }

    #[test]
    fn test_violation_tracker_reset() {
        let mut tracker = PlayerViolationTracker::new();
        tracker.record_speed_violation();
        tracker.record_teleport_violation(10);
        tracker.reset();
        assert_eq!(tracker.speed_violations, 0);
        assert_eq!(tracker.teleport_violations, 0);
        assert!(!tracker.mark_for_kick);
    }

    #[test]
    fn test_violation_manager() {
        let mut manager = ViolationManager::new();
        let player_id = PlayerId::new(1);

        let tracker = manager.get_tracker(player_id);
        tracker.record_speed_violation();

        assert_eq!(manager.len(), 1);
        assert!(manager.get_tracker_ref(player_id).is_some());

        manager.remove_tracker(player_id);
        assert_eq!(manager.len(), 0);
        assert!(manager.get_tracker_ref(player_id).is_none());
    }
}
