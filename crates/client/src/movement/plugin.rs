// Movement plugin and configuration

use bevy::prelude::*;
use trueworld_core::PlayerInput;

use super::prediction::PredictedState;
use super::correction::PositionCorrection;

/// Movement configuration resource
#[derive(Resource, Clone, Debug)]
pub struct MovementConfig {
    /// Walking speed (units/second)
    pub walk_speed: f32,
    /// Running/sprinting speed (units/second)
    pub run_speed: f32,
    /// Jump velocity (units/second)
    pub jump_velocity: f32,
    /// Gravity (units/second²)
    pub gravity: f32,
    /// Position correction threshold (units)
    ///
    /// If server position differs by more than this, apply correction.
    pub correction_threshold: f32,
    /// Correction interpolation factor (0-1)
    ///
    /// Higher = faster correction, Lower = smoother correction
    pub correction_lerp: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            walk_speed: 5.0,
            run_speed: 8.0,
            jump_velocity: 5.0,
            gravity: 20.0,
            correction_threshold: super::DEFAULT_CORRECTION_THRESHOLD,
            correction_lerp: super::DEFAULT_CORRECTION_LERP,
        }
    }
}

/// Client movement plugin
///
/// Sets up systems for:
/// - Collecting input from InputState
/// - Predicting local movement
/// - Sending input to server
/// - Correcting position based on server updates
pub struct ClientMovementPlugin;

impl Plugin for ClientMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementConfig>()
            .init_resource::<PositionCorrection>()
            .add_systems(Startup, setup_movement)
            .add_systems(Update, (
                send_input_to_server,
            ));
    }
}

/// Setup movement systems
fn setup_movement(mut commands: Commands) {
    // Spawn the local player entity with PredictedState
    commands.spawn((
        PredictedState::default(),
        Transform::from_translation(Vec3::ZERO),
        Visibility::default(),
    ));

    info!("Client movement system initialized");
}

/// Send input to server (60Hz)
///
/// This runs every frame to send the current input state to the server.
/// Uses unreliable channel since dropped packets are acceptable.
fn send_input_to_server(
    input_state: Res<crate::input::InputState>,
    // TODO: Add Renet client resource when available
) {
    // TODO: Implement actual network sending
    // let packet = ClientInputPacket::from_player_input(&input_state.current);
    // let bytes = serialize_packet(&packet)?;
    // client.send_message(DefaultChannel::Unreliable, bytes);

    // For now, just log the input (will be removed when network is connected)
    if !input_state.current.movement.is_empty() {
        let has_movement = input_state.current.movement.iter()
            .any(|&v| v.abs() > 0.01);

        if has_movement {
            debug!(
                "seq={}, movement={:?}, actions={}",
                input_state.sequence,
                input_state.current.movement,
                input_state.current.actions.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_config_default() {
        let config = MovementConfig::default();
        assert_eq!(config.walk_speed, 5.0);
        assert_eq!(config.run_speed, 8.0);
        assert_eq!(config.jump_velocity, 5.0);
        assert_eq!(config.gravity, 20.0);
        assert_eq!(config.correction_threshold, 2.0);
        assert_eq!(config.correction_lerp, 0.3);
    }

    #[test]
    fn test_movement_config_clone() {
        let config = MovementConfig::default();
        let cloned = config.clone();
        assert_eq!(config.walk_speed, cloned.walk_speed);
    }
}
