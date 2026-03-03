// crates/server/src/movement/validation.rs
//
// Movement validation system for the TrueWorld server.
// Validates client movement inputs for anti-cheat and physics consistency.

#![allow(dead_code)]

use trueworld_core::{PlayerId, Position, Velocity, PlayerInput, InputAction};
use super::config::ServerMovementConfig;

/// Result of movement validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Movement is valid
    Valid,

    /// Movement too fast (speed limit exceeded)
    TooFast {
        max_speed: f32,
        actual_speed: f32,
    },

    /// Collision detected (would clip through wall)
    Collision {
        at: Position,
    },

    /// Teleport detected (position jump too large)
    TeleportDetected {
        from: Position,
        to: Position,
        distance: f32,
    },

    /// Old input (ignored due to age)
    OldInput {
        sequence: u32,
        last_sequence: u32,
    },

    /// Player not found
    PlayerNotFound {
        player_id: PlayerId,
    },
}

impl ValidationResult {
    /// Returns true if the movement is valid
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Returns true if the violation should trigger a correction
    #[must_use]
    pub const fn requires_correction(&self) -> bool {
        matches!(self, Self::TooFast { .. } | Self::Collision { .. } | Self::TeleportDetected { .. })
    }

    /// Returns the reason as a string
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::TooFast { .. } => "too_fast",
            Self::Collision { .. } => "collision",
            Self::TeleportDetected { .. } => "teleport",
            Self::OldInput { .. } => "old_input",
            Self::PlayerNotFound { .. } => "player_not_found",
        }
    }
}

/// Server-side player movement state
#[derive(Debug, Clone, PartialEq)]
pub struct ServerPlayerMovement {
    /// Current authoritative position
    pub position: Position,

    /// Current velocity
    pub velocity: Velocity,

    /// Current rotation (Y-axis only for simplicity)
    pub rotation: f32,

    /// Is player on ground?
    pub on_ground: bool,

    /// Is player sprinting?
    pub is_sprinting: bool,

    /// Last confirmed input sequence from client
    pub last_client_sequence: u32,

    /// Last update time (server ticks)
    pub last_update_tick: u64,

    /// Position at last update
    pub last_position: Position,
}

impl Default for ServerPlayerMovement {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            rotation: 0.0,
            on_ground: true,
            is_sprinting: false,
            last_client_sequence: 0,
            last_update_tick: 0,
            last_position: [0.0, 0.0, 0.0],
        }
    }
}

impl ServerPlayerMovement {
    /// Creates a new movement state at a position
    #[must_use]
    pub fn new(position: Position) -> Self {
        Self {
            position,
            last_position: position,
            ..Default::default()
        }
    }

    /// Creates a new movement state with full parameters
    #[must_use]
    pub const fn new_full(
        position: Position,
        velocity: Velocity,
        rotation: f32,
        on_ground: bool,
        sequence: u32,
        tick: u64,
    ) -> Self {
        Self {
            position,
            velocity,
            rotation,
            on_ground,
            is_sprinting: false,
            last_client_sequence: sequence,
            last_update_tick: tick,
            last_position: position,
        }
    }

    /// Updates the movement state based on validated input
    pub fn update(&mut self, new_position: Position, new_velocity: Velocity, sequence: u32, tick: u64) {
        self.last_position = self.position;
        self.position = new_position;
        self.velocity = new_velocity;
        self.last_client_sequence = sequence;
        self.last_update_tick = tick;
    }

    /// Sets the sprinting state
    pub fn set_sprinting(&mut self, sprinting: bool) {
        self.is_sprinting = sprinting;
    }

    /// Returns the distance from the last position
    #[must_use]
    pub fn distance_from_last(&self) -> f32 {
        let dx = self.position[0] - self.last_position[0];
        let dy = self.position[1] - self.last_position[1];
        let dz = self.position[2] - self.last_position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Movement validator for server-side validation
pub struct MovementValidator {
    config: ServerMovementConfig,
}

impl MovementValidator {
    /// Creates a new movement validator
    #[must_use]
    pub const fn new(config: ServerMovementConfig) -> Self {
        Self { config }
    }

    /// Creates with default config
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            config: ServerMovementConfig::default(),
        }
    }

    /// Validates a player's movement input
    ///
    /// # Arguments
    /// * `player_id` - The player's ID
    /// * `current_state` - Current server-side movement state
    /// * `input` - The player's input
    /// * `delta_time` - Time since last update (seconds)
    /// * `current_tick` - Current server tick
    pub fn validate_movement(
        &self,
        _player_id: PlayerId,
        current_state: &ServerPlayerMovement,
        input: &PlayerInput,
        delta_time: f32,
        _current_tick: u64,
    ) -> ValidationResult {
        // Check for old/dupe input
        if input.sequence <= current_state.last_client_sequence {
            return ValidationResult::OldInput {
                sequence: input.sequence,
                last_sequence: current_state.last_client_sequence,
            };
        }

        // Calculate expected position based on input
        let expected_delta = self.calculate_movement_delta(input, current_state, delta_time);

        // Calculate expected position
        let expected_position = [
            current_state.position[0] + expected_delta[0],
            current_state.position[1] + expected_delta[1],
            current_state.position[2] + expected_delta[2],
        ];

        // Check for teleport (excessive position change)
        // Note: We don't have client's claimed position here, but we can check
        // if the calculated movement exceeds physical limits
        let movement_distance = (expected_delta[0].powi(2)
            + expected_delta[1].powi(2)
            + expected_delta[2].powi(2)).sqrt();

        let max_delta = self.config.max_delta_per_frame;
        if movement_distance > max_delta && !input.has_action(InputAction::Jump) {
            // Possible teleport/cheat
            return ValidationResult::TeleportDetected {
                from: current_state.position,
                to: expected_position,
                distance: movement_distance,
            };
        }

        // Check speed limits
        let is_sprinting = input.has_action(InputAction::Sprint);
        let max_speed = self.config.max_allowed_speed(is_sprinting);

        // Calculate speed from movement
        let speed = if delta_time > 0.0 {
            movement_distance / delta_time
        } else {
            0.0
        };

        // Jump adds vertical velocity
        let has_jump = input.has_action(InputAction::Jump);
        let _vertical_component = expected_delta[1].abs();

        // Allow higher vertical speed for jumping
        if !has_jump && speed > max_speed * 1.5 {
            // 50% tolerance for network jitter
            return ValidationResult::TooFast { max_speed, actual_speed: speed };
        }

        // Check for collision (basic ground collision)
        if expected_position[1] < 0.0 && !has_jump {
            // Below ground, would collide
            return ValidationResult::Collision {
                at: expected_position,
            };
        }

        ValidationResult::Valid
    }

    /// Validates a claimed client position against server state
    ///
    /// This is used when the client sends their position (for reconciliation)
    pub fn validate_position_claim(
        &self,
        server_position: Position,
        client_claimed_position: Position,
        delta_time: f32,
        is_sprinting: bool,
    ) -> ValidationResult {
        let max_speed = self.config.max_allowed_speed(is_sprinting);
        let max_distance = max_speed * delta_time * 2.0; // 2x tolerance

        let dx = client_claimed_position[0] - server_position[0];
        let dy = client_claimed_position[1] - server_position[1];
        let dz = client_claimed_position[2] - server_position[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        if distance > max_distance {
            return ValidationResult::TooFast {
                max_speed: max_distance / delta_time,
                actual_speed: distance / delta_time,
            };
        }

        // Ground collision check
        if client_claimed_position[1] < 0.0 {
            return ValidationResult::Collision {
                at: client_claimed_position,
            };
        }

        ValidationResult::Valid
    }

    /// Calculates the movement delta based on input
    fn calculate_movement_delta(
        &self,
        input: &PlayerInput,
        state: &ServerPlayerMovement,
        delta_time: f32,
    ) -> Position {
        let mut delta = [0.0f32; 3];

        // Base speed
        let speed = if input.has_action(InputAction::Sprint) {
            self.config.max_sprint_speed
        } else {
            self.config.max_speed
        };

        // Movement direction from input
        let has_forward = input.has_action(InputAction::MoveForward);
        let has_backward = input.has_action(InputAction::MoveBackward);
        let has_left = input.has_action(InputAction::MoveLeft);
        let has_right = input.has_action(InputAction::MoveRight);

        // Calculate movement vector
        let mut move_x = 0.0;
        let mut move_z = 0.0;

        if has_forward {
            move_z -= 1.0;
        }
        if has_backward {
            move_z += 1.0;
        }
        if has_left {
            move_x -= 1.0;
        }
        if has_right {
            move_x += 1.0;
        }

        // Normalize diagonal movement
        let length_sq: f32 = move_x * move_x + move_z * move_z;
        let length = length_sq.sqrt();
        if length > 0.0 {
            move_x /= length;
            move_z /= length;
        }

        delta[0] = move_x * speed * delta_time;
        delta[2] = move_z * speed * delta_time;

        // Jump movement
        if input.has_action(InputAction::Jump) && state.on_ground {
            delta[1] = self.config.max_jump_velocity * delta_time;
        } else if !state.on_ground {
            // Gravity when in air
            delta[1] = -9.8 * delta_time;
        }

        delta
    }

    /// Returns a reference to the config
    #[must_use]
    pub const fn config(&self) -> &ServerMovementConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input(sequence: u32) -> PlayerInput {
        let mut input = PlayerInput::new(sequence);
        input.add_action(InputAction::MoveForward);
        input
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::Valid;
        assert!(result.is_valid());
        assert!(!result.requires_correction());
        assert_eq!(result.reason(), "valid");
    }

    #[test]
    fn test_validation_result_too_fast() {
        let result = ValidationResult::TooFast {
            max_speed: 5.0,
            actual_speed: 10.0,
        };
        assert!(!result.is_valid());
        assert!(result.requires_correction());
        assert_eq!(result.reason(), "too_fast");
    }

    #[test]
    fn test_validation_result_teleport() {
        let result = ValidationResult::TeleportDetected {
            from: [0.0, 0.0, 0.0],
            to: [100.0, 0.0, 0.0],
            distance: 100.0,
        };
        assert!(!result.is_valid());
        assert!(result.requires_correction());
        assert_eq!(result.reason(), "teleport");
    }

    #[test]
    fn test_server_player_movement_default() {
        let movement = ServerPlayerMovement::default();
        assert_eq!(movement.position, [0.0, 0.0, 0.0]);
        assert_eq!(movement.velocity, [0.0, 0.0, 0.0]);
        assert!(movement.on_ground);
        assert_eq!(movement.last_client_sequence, 0);
    }

    #[test]
    fn test_server_player_movement_new() {
        let movement = ServerPlayerMovement::new([10.0, 5.0, 20.0]);
        assert_eq!(movement.position, [10.0, 5.0, 20.0]);
        assert_eq!(movement.last_position, [10.0, 5.0, 20.0]);
    }

    #[test]
    fn test_server_player_movement_update() {
        let mut movement = ServerPlayerMovement::new([0.0, 0.0, 0.0]);
        movement.update([5.0, 0.0, 10.0], [1.0, 0.0, 2.0], 5, 100);

        assert_eq!(movement.position, [5.0, 0.0, 10.0]);
        assert_eq!(movement.velocity, [1.0, 0.0, 2.0]);
        assert_eq!(movement.last_client_sequence, 5);
        assert_eq!(movement.last_update_tick, 100);
    }

    #[test]
    fn test_validator_with_defaults() {
        let validator = MovementValidator::with_defaults();
        assert_eq!(validator.config().max_speed, 5.0);
    }

    #[test]
    fn test_validate_movement_old_input() {
        let validator = MovementValidator::with_defaults();
        let state = ServerPlayerMovement::new_full(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            true,
            10,
            0,
        );
        let input = create_test_input(5); // Old sequence

        let result = validator.validate_movement(PlayerId::new(1), &state, &input, 0.016, 1);
        assert!(!result.is_valid());
        match result {
            ValidationResult::OldInput { sequence, .. } => assert_eq!(sequence, 5),
            _ => panic!("Expected OldInput result"),
        }
    }

    #[test]
    fn test_validate_movement_valid() {
        let validator = MovementValidator::with_defaults();
        let state = ServerPlayerMovement::new_full(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            true,
            0,
            0,
        );
        let mut input = create_test_input(1);
        input.movement = [0.0, 0.0, -1.0]; // Forward

        let result = validator.validate_movement(PlayerId::new(1), &state, &input, 0.016, 1);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_position_claim_valid() {
        let validator = MovementValidator::with_defaults();
        let server_pos = [0.0, 0.0, 0.0];
        let client_pos = [0.5, 0.0, 0.0]; // Small movement

        let result = validator.validate_position_claim(server_pos, client_pos, 0.1, false);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_position_claim_too_far() {
        let validator = MovementValidator::with_defaults();
        let server_pos = [0.0, 0.0, 0.0];
        let client_pos = [100.0, 0.0, 0.0]; // Teleport

        let result = validator.validate_position_claim(server_pos, client_pos, 0.1, false);
        assert!(!result.is_valid());
        assert!(result.requires_correction());
    }

    #[test]
    fn test_validate_position_claim_ground_collision() {
        let validator = MovementValidator::with_defaults();
        let server_pos = [0.0, 0.0, 0.0];
        // Position slightly below ground (small delta)
        let client_pos = [0.0, -0.1, 0.0];

        let result = validator.validate_position_claim(server_pos, client_pos, 0.1, false);
        // Distance is 0.1, max_distance is 1.0, so speed check passes
        // But y < 0 triggers collision
        assert!(!result.is_valid());
        match result {
            ValidationResult::Collision { .. } => {},
            other => panic!("Expected Collision result, got: {:?}", other),
        }
    }
}
