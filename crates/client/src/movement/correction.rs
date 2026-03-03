// Server position correction system

#![allow(dead_code)]

use bevy::prelude::*;
use trueworld_core::PlayerId;

use super::prediction::PredictedState;

/// Position correction state
///
/// Tracks correction state for smooth interpolation to server position.
#[derive(Resource, Default, Debug, Clone)]
pub struct PositionCorrection {
    /// Active correction target (if any)
    pub target: Option<CorrectionTarget>,
    /// Last received server position
    pub last_server_position: Option<Vec3>,
}

/// Active correction target
#[derive(Debug, Clone)]
pub struct CorrectionTarget {
    /// Target position from server
    pub position: Vec3,
    /// Reason for correction
    pub reason: CorrectionReason,
}

/// Reason for position correction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionReason {
    /// Speed limit exceeded
    SpeedLimitExceeded,
    /// Collision detected
    Collision,
    /// Server rollback
    ServerRollback,
    /// Teleport (gameplay mechanic)
    Teleport,
}

/// Correct position based on server update
///
/// This system runs when we receive a position update from the server.
/// It determines if correction is needed and applies it smoothly.
pub fn correct_position(
    mut predicted_query: Query<&mut PredictedState>,
    mut correction: ResMut<PositionCorrection>,
    config: Res<super::MovementConfig>,
    time: Res<Time>,
) {
    let Ok(mut predicted) = predicted_query.get_single_mut() else {
        return;
    };

    // Check if we have an active correction (clone to avoid borrow conflict)
    if let Some(target) = correction.target.clone() {
        match target.reason {
            CorrectionReason::Teleport => {
                // Instant teleport - no interpolation
                predicted.position = target.position;
                predicted.velocity = Vec3::ZERO;
                correction.target = None;
                info!("Teleported to {:?}", target.position);
            }
            _ => {
                // Smooth interpolation for other corrections
                let diff = target.position - predicted.position;
                let distance = diff.length();

                if distance < 0.01 {
                    // Close enough - correction complete
                    correction.target = None;
                } else {
                    // Lerp towards target
                    let lerp_amount = config.correction_lerp * time.delta_secs();
                    let step = diff * lerp_amount.min(1.0);
                    predicted.position += step;
                }
            }
        }
    }
}

/// Apply immediate position correction (for teleport/critical fixes)
pub fn apply_immediate_correction(
    predicted: &mut PredictedState,
    correction: &mut PositionCorrection,
    position: Vec3,
    reason: CorrectionReason,
) {
    predicted.position = position;
    correction.target = None;
    correction.last_server_position = Some(position);

    match reason {
        CorrectionReason::Teleport => {
            info!("Teleported to {:?}", position);
        }
        CorrectionReason::Collision => {
            debug!("Collision correction to {:?}", position);
        }
        CorrectionReason::SpeedLimitExceeded => {
            debug!("Speed limit correction to {:?}", position);
        }
        CorrectionReason::ServerRollback => {
            debug!("Server rollback to {:?}", position);
        }
    }
}

/// Check if position correction is needed
///
/// Returns the correction target if the difference is significant.
pub fn check_correction_needed(
    predicted: &PredictedState,
    server_position: Vec3,
    threshold: f32,
) -> Option<Vec3> {
    let diff = (server_position - predicted.position).length();

    if diff > threshold {
        Some(server_position)
    } else {
        None
    }
}

/// Process server position acknowledgment
///
/// Called when we receive ServerPositionAck from server.
pub fn process_position_ack(
    predicted: &mut PredictedState,
    correction: &mut PositionCorrection,
    _player_id: PlayerId,
    ack_sequence: u32,
    server_position: Vec3,
    server_velocity: Vec3,
    threshold: f32,
) -> bool {
    // Update acknowledged sequence
    predicted.ack_sequence(ack_sequence);

    // Store last server position
    correction.last_server_position = Some(server_position);

    // Check if correction is needed
    if let Some(correction_pos) = check_correction_needed(predicted, server_position, threshold) {
        // Determine reason based on velocity mismatch
        let vel_diff = (server_velocity - predicted.velocity).length();
        let reason = if vel_diff > 5.0 {
            CorrectionReason::SpeedLimitExceeded
        } else {
            CorrectionReason::ServerRollback
        };

        correction.target = Some(CorrectionTarget {
            position: correction_pos,
            reason,
        });

        true
    } else {
        // No correction needed
        correction.target = None;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::prediction::InputSnapshot;

    #[test]
    fn test_position_correction_default() {
        let correction = PositionCorrection::default();
        assert!(correction.target.is_none());
        assert!(correction.last_server_position.is_none());
    }

    #[test]
    fn test_check_correction_needed_within_threshold() {
        let predicted = PredictedState::at_position(Vec3::new(10.0, 0.0, 20.0));
        let server_pos = Vec3::new(10.5, 0.0, 20.0);

        // Threshold is 2.0 by default, difference is 0.5
        let result = check_correction_needed(&predicted, server_pos, 2.0);
        assert!(result.is_none(), "Should not correct for small differences");
    }

    #[test]
    fn test_check_correction_needed_exceeds_threshold() {
        let predicted = PredictedState::at_position(Vec3::new(10.0, 0.0, 20.0));
        let server_pos = Vec3::new(15.0, 0.0, 20.0);

        // Threshold is 2.0, difference is 5.0
        let result = check_correction_needed(&predicted, server_pos, 2.0);
        assert!(result.is_some(), "Should correct for large differences");
        assert_eq!(result.unwrap(), server_pos);
    }

    #[test]
    fn test_ack_sequence_clears_history() {
        let mut state = PredictedState::default();

        // Add snapshots
        for seq in 1..=5 {
            let input = trueworld_core::PlayerInput::new(seq);
            let snapshot = InputSnapshot::new(seq, input, Vec3::ZERO, Vec3::ZERO);
            state.add_snapshot(snapshot);
        }

        // Ack up to sequence 3
        state.ack_sequence(3);

        // Should only have 4 and 5 left
        assert_eq!(state.input_history.len(), 2);
        assert_eq!(state.last_ack_sequence, 3);
    }
}
