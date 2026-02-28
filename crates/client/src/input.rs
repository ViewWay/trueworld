// crates/client/src/input.rs
//
// Input collection and processing for the TrueWorld client.
// Collects keyboard, mouse, and gamepad input and converts it to PlayerInput.

use bevy::{
    input::{
        gamepad::Gamepad,
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion},
        ButtonInput,
    },
    prelude::*,
};

use trueworld_core::{InputAction, PlayerInput};

use crate::state::{ClientConfig, InputConfig};

/// Input plugin for collecting player input
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>().add_systems(
            Update,
            (
                collect_keyboard_input,
                collect_mouse_input,
                collect_gamepad_input,
                update_input_state,
            )
                .chain(),
        );
    }
}

/// Collected input state
#[derive(Resource, Default)]
pub struct InputState {
    /// Current frame's input
    pub current: PlayerInput,

    /// Previous frame's input
    pub previous: PlayerInput,

    /// Input sequence number
    pub sequence: u32,

    /// Current key states
    pub keys: ButtonInput<KeyCode>,

    /// Mouse button states
    pub mouse_buttons: ButtonInput<MouseButton>,

    /// Connected gamepads
    pub connected_gamepads: Vec<Gamepad>,

    /// Mouse position
    pub mouse_position: Vec2,

    /// Mouse movement delta
    pub mouse_delta: Vec2,

    /// Mouse wheel scroll
    pub mouse_wheel: f32,
}

impl InputState {
    /// Check if a key is pressed
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.keys.pressed(key)
    }

    /// Check if a key was just pressed
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.keys.just_pressed(key)
    }

    /// Check if a key was just released
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.keys.just_released(key)
    }

    /// Check if a mouse button is pressed
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.pressed(button)
    }

    /// Check if a mouse button was just pressed
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.just_pressed(button)
    }

    /// Get the 2D movement direction from keyboard input
    pub fn move_direction(&self) -> Vec2 {
        let mut dir = Vec2::ZERO;

        if self.pressed(KeyCode::KeyW) || self.pressed(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if self.pressed(KeyCode::KeyS) || self.pressed(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if self.pressed(KeyCode::KeyA) || self.pressed(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if self.pressed(KeyCode::KeyD) || self.pressed(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        if dir != Vec2::ZERO {
            dir = dir.normalize();
        }

        dir
    }

    /// Check if player is sprinting
    pub fn is_sprinting(&self) -> bool {
        self.pressed(KeyCode::ShiftLeft) || self.pressed(KeyCode::ShiftRight)
    }

    /// Check if player is crouching
    pub fn is_crouching(&self) -> bool {
        self.pressed(KeyCode::ControlLeft) || self.pressed(KeyCode::KeyC)
    }

    /// Check if player is aiming (right mouse button)
    pub fn is_aiming(&self) -> bool {
        self.mouse_pressed(MouseButton::Right)
    }

    /// Check if player is attacking (left mouse button)
    pub fn is_attacking(&self) -> bool {
        self.mouse_pressed(MouseButton::Left)
    }

    /// Get the skill slot (1-0) that was just pressed
    pub fn get_skill_slot(&self) -> Option<usize> {
        let skill_keys = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
            KeyCode::Digit0,
        ];

        for (i, &key) in skill_keys.iter().enumerate() {
            if self.just_pressed(key) {
                return Some(i);
            }
        }

        None
    }

    /// Get the item slot that was just pressed
    pub fn get_item_slot(&self) -> Option<usize> {
        if self.just_pressed(KeyCode::KeyQ) {
            return Some(0);
        }
        if self.just_pressed(KeyCode::KeyR) {
            return Some(1);
        }

        None
    }

    /// Collect current input as PlayerInput
    pub fn collect_input(&mut self, _config: &InputConfig) -> PlayerInput {
        let move_dir = self.move_direction();

        let mut input = PlayerInput::new(self.sequence);

        // Movement - convert 2D direction to 3D movement array
        input.movement = [move_dir.x, 0.0, move_dir.y];

        // Forward/backward
        if move_dir.y > 0.0 {
            input.add_action(InputAction::MoveForward);
        } else if move_dir.y < 0.0 {
            input.add_action(InputAction::MoveBackward);
        }

        // Strafe left/right
        if move_dir.x > 0.0 {
            input.add_action(InputAction::MoveRight);
        } else if move_dir.x < 0.0 {
            input.add_action(InputAction::MoveLeft);
        }

        // Sprint
        if self.is_sprinting() {
            input.add_action(InputAction::Sprint);
        }

        // Crouch
        if self.is_crouching() {
            input.add_action(InputAction::Crouch);
        }

        // Jump
        if self.just_pressed(KeyCode::Space) {
            input.add_action(InputAction::Jump);
        }

        // Attack
        if self.is_attacking() {
            input.add_action(InputAction::Attack);
        }

        // Block
        if self.is_aiming() {
            input.add_action(InputAction::Block);
        }

        // Dodge
        if self.just_pressed(KeyCode::AltLeft) {
            input.add_action(InputAction::Dodge);
        }

        // Skills
        if let Some(slot) = self.get_skill_slot() {
            match slot {
                0 => input.add_action(InputAction::Skill1),
                1 => input.add_action(InputAction::Skill2),
                2 => input.add_action(InputAction::Skill3),
                3 => input.add_action(InputAction::Skill4),
                _ => {}
            }
        }

        // Interact
        if self.just_pressed(KeyCode::KeyE) {
            input.add_action(InputAction::Interact);
        }

        // Use item
        if self.get_item_slot().is_some() {
            input.add_action(InputAction::UseItem);
        }

        // Menu toggles
        if self.just_pressed(KeyCode::Tab) || self.just_pressed(KeyCode::KeyI) {
            input.add_action(InputAction::ToggleInventory);
        }

        if self.just_pressed(KeyCode::KeyM) {
            input.add_action(InputAction::ToggleMap);
        }

        if self.just_pressed(KeyCode::Escape) {
            input.add_action(InputAction::ToggleMenu);
        }

        // Set timestamp (using milliseconds since UNIX epoch)
        use std::time::{SystemTime, UNIX_EPOCH};
        input.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        input
    }

    /// Advance to the next frame
    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        self.sequence = self.sequence.wrapping_add(1);

        // Reset one-frame state
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel = 0.0;
    }
}

/// Collect keyboard input
fn collect_keyboard_input(mut input_state: ResMut<InputState>, keys: Res<ButtonInput<KeyCode>>) {
    input_state.keys = keys.clone();
}

/// Collect mouse input
fn collect_mouse_input(
    mut input_state: ResMut<InputState>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion_reader: EventReader<MouseMotion>,
    mut scroll_reader: EventReader<bevy::input::mouse::MouseWheel>,
) {
    input_state.mouse_buttons = buttons.clone();

    // Mouse motion
    for event in motion_reader.read() {
        input_state.mouse_delta += event.delta;
    }

    // Mouse wheel
    for event in scroll_reader.read() {
        input_state.mouse_wheel += event.y;
    }

    // TODO: Mouse position requires window queries
    // Will be added when bevy_window feature is properly configured
}

/// Collect gamepad input
fn collect_gamepad_input(
    mut input_state: ResMut<InputState>,
    gamepads: Query<(), With<Gamepad>>,
) {
    // In Bevy 0.15, we can just count gamepads
    // The actual gamepad-specific input would be handled in a different system
    let gamepad_count = gamepads.iter().len();
    input_state.connected_gamepads.clear();
    // Reserve space for connected gamepads
    input_state.connected_gamepads.reserve(gamepad_count);
}

/// Update input state with collected input
fn update_input_state(mut input_state: ResMut<InputState>, config: Res<ClientConfig>) {
    input_state.current = input_state.collect_input(&config.input);
    input_state.advance();
}
