// Local prediction system for client-side movement

#![allow(dead_code)]

use bevy::prelude::*;
use trueworld_core::PlayerInput;
use std::collections::VecDeque;

use super::MAX_INPUT_HISTORY;

/// Local predicted state for the player
///
/// This component tracks:
/// - The client's predicted position
/// - The last sequence number confirmed by server
/// - History of unconfirmed inputs for rollback
#[derive(Component, Debug, Clone)]
pub struct PredictedState {
    /// Current predicted position
    pub position: Vec3,
    /// Current velocity
    pub velocity: Vec3,
    /// Last sequence number acknowledged by server
    pub last_ack_sequence: u32,
    /// Input history for potential rollback
    pub input_history: VecDeque<InputSnapshot>,
}

impl Default for PredictedState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            last_ack_sequence: 0,
            input_history: VecDeque::with_capacity(MAX_INPUT_HISTORY),
        }
    }
}

impl PredictedState {
    /// Create a new predicted state at origin
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create at specific position
    #[must_use]
    pub fn at_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Add an input snapshot to history
    pub fn add_snapshot(&mut self, snapshot: InputSnapshot) {
        self.input_history.push_back(snapshot);

        // Keep history size bounded
        while self.input_history.len() > MAX_INPUT_HISTORY {
            self.input_history.pop_front();
        }
    }

    /// Remove snapshots up to and including the given sequence
    ///
    /// Called when server confirms a sequence number.
    pub fn ack_sequence(&mut self, sequence: u32) {
        self.last_ack_sequence = sequence;

        // Remove old snapshots
        while let Some(front) = self.input_history.front() {
            if front.sequence <= sequence {
                self.input_history.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get the latest unconfirmed sequence
    #[must_use]
    pub fn latest_sequence(&self) -> Option<u32> {
        self.input_history.back().map(|s| s.sequence)
    }

    /// Update position based on velocity and delta time
    pub fn update_position(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
    }
}

/// Input snapshot for rollback/reconciliation
///
/// Stores the state at each input so we can replay inputs
/// if server position differs from our prediction.
#[derive(Debug, Clone)]
pub struct InputSnapshot {
    /// Input sequence number
    pub sequence: u32,
    /// Player input that generated this snapshot
    pub input: PlayerInput,
    /// Position after applying this input
    pub position: Vec3,
    /// Velocity after applying this input
    pub velocity: Vec3,
}

impl InputSnapshot {
    /// Create a new input snapshot
    #[must_use]
    pub fn new(sequence: u32, input: PlayerInput, position: Vec3, velocity: Vec3) -> Self {
        Self {
            sequence,
            input,
            position,
            velocity,
        }
    }
}

/// Predict movement based on input
///
/// This is the core client-side prediction function.
/// It simulates what the server will do with the input.
pub fn predict_movement(
    current_position: Vec3,
    current_velocity: Vec3,
    input: &PlayerInput,
    config: &super::MovementConfig,
    delta_time: f32,
) -> (Vec3, Vec3) {
    let mut new_position = current_position;
    let mut new_velocity = current_velocity;

    // Extract movement direction from input
    let move_dir = Vec3::from_array(input.movement);

    // Check if sprinting
    let is_sprinting = input.has_action(trueworld_core::InputAction::Sprint);
    let speed = if is_sprinting {
        config.run_speed
    } else {
        config.walk_speed
    };

    // Apply horizontal movement
    if move_dir.length_squared() > 0.01 {
        let horizontal_dir = Vec3::new(move_dir.x, 0.0, move_dir.z);
        let normalized = horizontal_dir.normalize();

        new_velocity.x = normalized.x * speed;
        new_velocity.z = normalized.z * speed;
    } else {
        // Friction when not moving
        new_velocity.x *= 0.8;
        new_velocity.z *= 0.8;
    }

    // Apply gravity
    new_velocity.y -= config.gravity * delta_time;

    // Check for jump
    let is_jumping = input.has_action(trueworld_core::InputAction::Jump);
    let was_on_ground = current_position.y <= 0.01;

    if is_jumping && was_on_ground {
        new_velocity.y = config.jump_velocity;
    }

    // Ground collision
    if new_position.y <= 0.0 {
        new_position.y = 0.0;
        if new_velocity.y < 0.0 {
            new_velocity.y = 0.0;
        }
    }

    // Update position
    new_position += new_velocity * delta_time;

    (new_position, new_velocity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trueworld_core::{InputAction};

    #[test]
    fn test_predicted_state_default() {
        let state = PredictedState::default();
        assert_eq!(state.position, Vec3::ZERO);
        assert_eq!(state.velocity, Vec3::ZERO);
        assert_eq!(state.last_ack_sequence, 0);
        assert_eq!(state.input_history.len(), 0);
    }

    #[test]
    fn test_predicted_state_at_position() {
        let pos = Vec3::new(10.0, 0.0, 20.0);
        let state = PredictedState::at_position(pos);
        assert_eq!(state.position, pos);
    }

    #[test]
    fn test_add_snapshot() {
        let mut state = PredictedState::default();
        let input = PlayerInput::new(1);
        let snapshot = InputSnapshot::new(1, input, Vec3::ZERO, Vec3::ZERO);

        state.add_snapshot(snapshot.clone());

        assert_eq!(state.input_history.len(), 1);
        assert_eq!(state.input_history.front().unwrap().sequence, 1);
    }

    #[test]
    fn test_ack_sequence() {
        let mut state = PredictedState::default();

        // Add snapshots with sequences 1, 2, 3
        for seq in 1..=3 {
            let input = PlayerInput::new(seq);
            let snapshot = InputSnapshot::new(seq, input, Vec3::ZERO, Vec3::ZERO);
            state.add_snapshot(snapshot);
        }

        // Acknowledge sequence 2
        state.ack_sequence(2);

        // Should have removed 1 and 2, keeping 3
        assert_eq!(state.input_history.len(), 1);
        assert_eq!(state.last_ack_sequence, 2);
        assert_eq!(state.input_history.front().unwrap().sequence, 3);
    }

    #[test]
    fn test_predict_movement_forward() {
        let config = crate::movement::MovementConfig::default();

        let mut input = PlayerInput::new(1);
        input.movement = [0.0, 0.0, 1.0]; // Forward

        let (pos, vel) = predict_movement(Vec3::ZERO, Vec3::ZERO, &input, &config, 1.0);

        assert!(pos.z > 0.0, "Should move forward");
        assert!(vel.z > 0.0, "Should have forward velocity");
    }

    #[test]
    fn test_predict_movement_jump() {
        let config = crate::movement::MovementConfig::default();

        let mut input = PlayerInput::new(1);
        input.add_action(InputAction::Jump);

        let (_pos, vel) = predict_movement(Vec3::ZERO, Vec3::ZERO, &input, &config, 1.0);

        assert!(vel.y > 0.0, "Should have upward velocity when jumping");
    }
}
