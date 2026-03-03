// Movement plugin and configuration

#![allow(dead_code)]

use bevy::prelude::*;

use super::prediction::PredictedState;
use super::correction::PositionCorrection;

use crate::network::NetworkResource;
use trueworld_protocol::ClientInputPacket;

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
/// Uses unreliable channel 2 since dropped packets are acceptable.
fn send_input_to_server(
    input_state: Res<crate::input::InputState>,
    mut network: ResMut<NetworkResource>,
) {
    // Only send if connected
    if !network.is_connected() {
        return;
    }

    let Some(client) = &mut network.client else {
        return;
    };

    // Create ClientInputPacket from current input
    let packet = ClientInputPacket {
        sequence: input_state.sequence,
        movement: input_state.current.movement,
        actions: input_state.current.actions.clone(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Serialize the packet
    let bytes = match bincode::serialize(&packet) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to serialize ClientInputPacket: {}", e);
            return;
        }
    };

    // Send on unreliable channel 2
    client.send_message(2, bytes);

    // Debug logging (only when there's actual movement)
    if !input_state.current.movement.is_empty() || !input_state.current.actions.is_empty() {
        let has_movement = input_state.current.movement.iter()
            .any(|&v| v.abs() > 0.01);

        if has_movement || !input_state.current.actions.is_empty() {
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
