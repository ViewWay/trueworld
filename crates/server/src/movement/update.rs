// crates/server/src/movement/update.rs
//
// Position update processing for the TrueWorld server.
// Handles client input packets, updates positions, and sends acknowledgments.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use trueworld_core::{PlayerId, Position, Velocity, PlayerInput};
use trueworld_protocol::{ClientInputPacket, ServerPositionAck, ServerPositionCorrection, CorrectionReason};

use super::validation::{MovementValidator, ValidationResult, ServerPlayerMovement};
use super::config::{ServerMovementConfig, ViolationManager};

/// Processor for movement updates on the server
pub struct MovementUpdateProcessor {
    /// Movement validator
    validator: MovementValidator,

    /// Player movement states
    player_states: HashMap<PlayerId, ServerPlayerMovement>,

    /// Violation tracking manager
    violations: ViolationManager,

    /// Pending corrections to send
    pending_corrections: Vec<ServerPositionCorrection>,

    /// Current server tick
    current_tick: u64,

    /// Pending position acknowledgments
    pending_acks: Vec<ServerPositionAck>,
}

impl MovementUpdateProcessor {
    /// Creates a new movement update processor
    pub fn new(config: ServerMovementConfig) -> Self {
        Self {
            validator: MovementValidator::new(config.clone()),
            player_states: HashMap::new(),
            violations: ViolationManager::new(),
            pending_corrections: Vec::new(),
            current_tick: 0,
            pending_acks: Vec::new(),
        }
    }

    /// Creates with default config
    pub fn with_defaults() -> Self {
        Self::new(ServerMovementConfig::default())
    }

    /// Processes a client input packet
    pub fn process_client_input(
        &mut self,
        player_id: PlayerId,
        packet: &ClientInputPacket,
        delta_time: Duration,
    ) -> ProcessInputResult {
        // Clone the data we need before borrowing
        let current_tick = self.current_tick;
        let config = self.validator.config().clone();
        let violation_threshold = config.teleport_violation_threshold;

        // Get current state data before borrowing
        let (last_sequence, current_position, last_position) = {
            let state = self.player_states.get(&player_id);
            if let Some(s) = state {
                (s.last_client_sequence, s.position, s.last_position)
            } else {
                (0, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
            }
        };

        // Create PlayerInput from packet
        let player_input = PlayerInput {
            sequence: packet.sequence,
            movement: packet.movement,
            actions: packet.actions.clone(),
            view_direction: [0.0, 0.0, 0.0],
            timestamp: packet.timestamp,
        };

        // Check for old input first (without full validation)
        if packet.sequence <= last_sequence {
            return ProcessInputResult::IgnoredOldInput;
        }

        // Create a temp state for validation
        let temp_state = ServerPlayerMovement::new_full(
            current_position,
            [0.0, 0.0, 0.0],
            0.0,
            true,
            last_sequence,
            current_tick,
        );

        // Create a temporary validator for this check
        let validator = MovementValidator::new(config);

        // Validate the movement
        let result = validator.validate_movement(
            player_id,
            &temp_state,
            &player_input,
            delta_time.as_secs_f32(),
            current_tick,
        );

        // Calculate delta ahead of time (while we can still borrow self)
        let config_for_delta = self.validator.config().clone();
        let delta_for_calc = if matches!(result, ValidationResult::Valid) {
            // Get current state (immutable) for delta calculation
            let current_state_for_delta = self.player_states.get(&player_id);
            if let Some(s) = current_state_for_delta {
                Some(self.calculate_delta_from_input_impl(&player_input, s, delta_time.as_secs_f32(), &config_for_delta))
            } else {
                Some(self.calculate_delta_from_input_impl(&player_input, &ServerPlayerMovement::new([0.0, 0.0, 0.0]), delta_time.as_secs_f32(), &config_for_delta))
            }
        } else {
            None
        };

        // Now get mutable state (after validation)
        let mut state = self.player_states
            .entry(player_id)
            .or_insert_with(|| ServerPlayerMovement::new([0.0, 0.0, 0.0]));

        match result {
            ValidationResult::Valid => {
                // Use pre-calculated delta
                let delta = delta_for_calc.unwrap();
                let new_position = [
                    state.position[0] + delta[0],
                    state.position[1] + delta[1],
                    state.position[2] + delta[2],
                ];

                // Calculate velocity
                let dt = delta_time.as_secs_f32();
                let velocity = if dt > 0.0 {
                    [
                        delta[0] / dt,
                        delta[1] / dt,
                        delta[2] / dt,
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                };

                // Update state
                state.update(new_position, velocity, packet.sequence, self.current_tick);

                // Queue acknowledgment (drop state borrow first)
                drop(state);
                self.queue_position_ack(player_id, packet.sequence);

                ProcessInputResult::Success {
                    new_position,
                    velocity,
                }
            }

            ValidationResult::OldInput { .. } => {
                drop(state);
                ProcessInputResult::IgnoredOldInput
            }

            ValidationResult::TooFast { .. } => {
                let pos = state.position;
                drop(state);

                // Record violation and send correction
                let tracker = self.violations.get_tracker(player_id);
                tracker.record_speed_violation();

                self.queue_position_correction(
                    player_id,
                    pos,
                    CorrectionReason::SpeedLimitExceeded,
                );

                ProcessInputResult::Violation {
                    reason: result,
                    corrected_position: pos,
                }
            }

            ValidationResult::Collision { .. } => {
                let corrected = [state.position[0], 0.0, state.position[2]];
                state.position = corrected;
                drop(state);

                // Record violation and send correction
                let tracker = self.violations.get_tracker(player_id);
                tracker.record_collision_violation();

                self.queue_position_correction(
                    player_id,
                    corrected,
                    CorrectionReason::Collision,
                );

                ProcessInputResult::Violation {
                    reason: result,
                    corrected_position: corrected,
                }
            }

            ValidationResult::TeleportDetected { .. } => {
                let pos = state.position;
                drop(state);

                // Record violation - may kick
                let tracker = self.violations.get_tracker(player_id);
                let should_kick = tracker.record_teleport_violation(violation_threshold);

                if should_kick {
                    ProcessInputResult::KickRequired {
                        player_id,
                        reason: "Teleport cheat detected".to_string(),
                    }
                } else {
                    self.queue_position_correction(
                        player_id,
                        pos,
                        CorrectionReason::ServerRollback,
                    );

                    ProcessInputResult::Violation {
                        reason: result,
                        corrected_position: pos,
                    }
                }
            }

            ValidationResult::PlayerNotFound { .. } => {
                drop(state);
                ProcessInputResult::PlayerNotFound
            }
        }
    }

    /// Adds a new player to the processor
    pub fn add_player(&mut self, player_id: PlayerId, spawn_position: Position) {
        self.player_states.insert(
            player_id,
            ServerPlayerMovement::new(spawn_position),
        );
    }

    /// Removes a player from the processor
    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.player_states.remove(&player_id);
        self.violations.remove_tracker(player_id);
    }

    /// Gets a player's current position
    pub fn get_player_position(&self, player_id: PlayerId) -> Option<Position> {
        self.player_states.get(&player_id).map(|s| s.position)
    }

    /// Gets a player's current state
    pub fn get_player_state(&self, player_id: PlayerId) -> Option<&ServerPlayerMovement> {
        self.player_states.get(&player_id)
    }

    /// Updates the processor (called each tick)
    pub fn update(&mut self, delta_time: Duration) {
        self.current_tick = self.current_tick.wrapping_add(1);

        // Decay old violations
        self.violations.update(60); // Decay after 60 seconds of no violations

        // Update player states (gravity, etc.)
        let dt = delta_time.as_secs_f32();
        for state in self.player_states.values_mut() {
            // Apply gravity if not on ground
            if !state.on_ground {
                state.velocity[1] -= 9.8 * dt;
                state.position[1] += state.velocity[1] * dt;

                // Ground check
                if state.position[1] <= 0.0 {
                    state.position[1] = 0.0;
                    state.velocity[1] = 0.0;
                    state.on_ground = true;
                }
            }
        }
    }

    /// Returns pending corrections and clears the buffer
    pub fn take_corrections(&mut self) -> Vec<ServerPositionCorrection> {
        std::mem::take(&mut self.pending_corrections)
    }

    /// Returns pending acknowledgments and clears the buffer
    pub fn take_acks(&mut self) -> Vec<ServerPositionAck> {
        std::mem::take(&mut self.pending_acks)
    }

    /// Creates position update packets for all players
    pub fn create_position_updates(&self) -> Vec<(PlayerId, Position, Velocity)> {
        self.player_states
            .iter()
            .map(|(&id, state)| (id, state.position, state.velocity))
            .collect()
    }

    /// Calculates position delta from input
    fn calculate_delta_from_input(
        &self,
        input: &PlayerInput,
        state: &ServerPlayerMovement,
        delta_time: f32,
    ) -> Position {
        self.calculate_delta_from_input_impl(input, state, delta_time, self.validator.config())
    }

    /// Internal implementation of delta calculation (allows passing config directly)
    fn calculate_delta_from_input_impl(
        &self,
        input: &PlayerInput,
        state: &ServerPlayerMovement,
        delta_time: f32,
        config: &ServerMovementConfig,
    ) -> Position {
        let speed = if input.actions.iter().any(|a| a == &trueworld_core::InputAction::Sprint) {
            config.max_sprint_speed
        } else {
            config.max_speed
        };

        let mut delta = [0.0f32; 3];

        // Process movement direction
        for action in &input.actions {
            match action {
                trueworld_core::InputAction::MoveForward => delta[2] -= speed * delta_time,
                trueworld_core::InputAction::MoveBackward => delta[2] += speed * delta_time,
                trueworld_core::InputAction::MoveLeft => delta[0] -= speed * delta_time,
                trueworld_core::InputAction::MoveRight => delta[0] += speed * delta_time,
                trueworld_core::InputAction::Jump if state.on_ground => {
                    delta[1] = config.max_jump_velocity * delta_time;
                    // Note: We don't modify state.on_ground here - that's done in update()
                }
                _ => {}
            }
        }

        delta
    }

    /// Queues a position acknowledgment for a player
    fn queue_position_ack(&mut self, player_id: PlayerId, sequence: u32) {
        if let Some(state) = self.player_states.get(&player_id) {
            let ack = ServerPositionAck {
                player_id,
                ack_sequence: sequence,
                position: state.position,
                velocity: state.velocity,
                server_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            self.pending_acks.push(ack);
        }
    }

    /// Queues a position correction for a player
    fn queue_position_correction(
        &mut self,
        player_id: PlayerId,
        correct_position: Position,
        reason: CorrectionReason,
    ) {
        let correction = ServerPositionCorrection {
            player_id,
            correct_position,
            reason,
        };
        self.pending_corrections.push(correction);
    }

    /// Returns the current server tick
    #[must_use]
    pub const fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Returns the number of tracked players
    #[must_use]
    pub fn player_count(&self) -> usize {
        self.player_states.len()
    }

    /// Returns a reference to the validator
    #[must_use]
    pub const fn validator(&self) -> &MovementValidator {
        &self.validator
    }
}

impl Default for MovementUpdateProcessor {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Result of processing a client input
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessInputResult {
    /// Input processed successfully
    Success {
        new_position: Position,
        velocity: Velocity,
    },

    /// Old input ignored (duplicate/out of order)
    IgnoredOldInput,

    /// Violation detected and corrected
    Violation {
        reason: ValidationResult,
        corrected_position: Position,
    },

    /// Player should be kicked
    KickRequired {
        player_id: PlayerId,
        reason: String,
    },

    /// Player not found
    PlayerNotFound,
}

impl ProcessInputResult {
    /// Returns true if processing was successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Returns the new position if successful
    #[must_use]
    pub fn position(&self) -> Option<Position> {
        match self {
            Self::Success { new_position, .. } => Some(*new_position),
            Self::Violation { corrected_position, .. } => Some(*corrected_position),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_new() {
        let processor = MovementUpdateProcessor::with_defaults();
        assert_eq!(processor.current_tick(), 0);
        assert_eq!(processor.player_count(), 0);
    }

    #[test]
    fn test_processor_add_player() {
        let mut processor = MovementUpdateProcessor::with_defaults();
        let player_id = PlayerId::new(1);

        processor.add_player(player_id, [10.0, 0.0, 20.0]);

        assert_eq!(processor.player_count(), 1);
        assert_eq!(processor.get_player_position(player_id), Some([10.0, 0.0, 20.0]));
    }

    #[test]
    fn test_processor_remove_player() {
        let mut processor = MovementUpdateProcessor::with_defaults();
        let player_id = PlayerId::new(1);

        processor.add_player(player_id, [0.0, 0.0, 0.0]);
        assert_eq!(processor.player_count(), 1);

        processor.remove_player(player_id);
        assert_eq!(processor.player_count(), 0);
        assert!(processor.get_player_position(player_id).is_none());
    }

    #[test]
    fn test_processor_update_tick() {
        let mut processor = MovementUpdateProcessor::with_defaults();
        assert_eq!(processor.current_tick(), 0);

        processor.update(Duration::from_secs_f32(0.016));
        assert_eq!(processor.current_tick(), 1);

        processor.update(Duration::from_secs_f32(0.016));
        assert_eq!(processor.current_tick(), 2);
    }

    #[test]
    fn test_process_input_result_is_success() {
        let result = ProcessInputResult::Success {
            new_position: [5.0, 0.0, 10.0],
            velocity: [1.0, 0.0, 0.0],
        };
        assert!(result.is_success());
        assert_eq!(result.position(), Some([5.0, 0.0, 10.0]));
    }

    #[test]
    fn test_process_input_result_violation() {
        let result = ProcessInputResult::Violation {
            reason: ValidationResult::TooFast {
                max_speed: 5.0,
                actual_speed: 10.0,
            },
            corrected_position: [0.0, 0.0, 0.0],
        };
        assert!(!result.is_success());
        assert_eq!(result.position(), Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_process_input_result_old_input() {
        let result = ProcessInputResult::IgnoredOldInput;
        assert!(!result.is_success());
        assert_eq!(result.position(), None);
    }

    #[test]
    fn test_process_client_input_valid() {
        let mut processor = MovementUpdateProcessor::with_defaults();
        let player_id = PlayerId::new(1);
        processor.add_player(player_id, [0.0, 0.0, 0.0]);

        let packet = ClientInputPacket {
            sequence: 1,
            movement: [0.0, 0.0, -1.0], // Forward
            actions: vec![],
            timestamp: 1000,
        };

        let result = processor.process_client_input(
            player_id,
            &packet,
            Duration::from_secs_f32(0.016),
        );

        assert!(result.is_success());
    }

    #[test]
    fn test_process_client_input_old_sequence() {
        let mut processor = MovementUpdateProcessor::with_defaults();
        let player_id = PlayerId::new(1);
        processor.add_player(player_id, [0.0, 0.0, 0.0]);

        // First input
        let packet1 = ClientInputPacket {
            sequence: 10,
            movement: [0.0, 0.0, -1.0],
            actions: vec![],
            timestamp: 1000,
        };
        processor.process_client_input(player_id, &packet1, Duration::from_secs_f32(0.016));

        // Old input (lower sequence)
        let packet2 = ClientInputPacket {
            sequence: 5,
            movement: [0.0, 0.0, -1.0],
            actions: vec![],
            timestamp: 1001,
        };
        let result = processor.process_client_input(
            player_id,
            &packet2,
            Duration::from_secs_f32(0.016),
        );

        assert_eq!(result, ProcessInputResult::IgnoredOldInput);
    }
}
