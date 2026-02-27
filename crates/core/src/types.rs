// Common types for TrueWorld

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// ============================================================================
// Type Aliases
// ============================================================================

/// 3D Position in world space
pub type Position = [f32; 3];

/// 3D Rotation as Euler angles (pitch, yaw, roll) in radians
pub type Rotation = [f32; 3];

/// 2D coordinate
pub type Coord2 = [f32; 2];

/// 3D coordinate
pub type Coord3 = [f32; 3];

/// 3D Velocity
pub type Velocity = [f32; 3];

// ============================================================================
// Game State Enum
// ============================================================================

/// The current state of a game session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// Players are gathering in the lobby
    Lobby,
    /// Game is loading resources and initializing
    Loading,
    /// Active gameplay
    Playing,
    /// Game is temporarily paused
    Paused,
    /// Game has concluded
    Ended,
}

impl GameState {
    /// Returns true if the game is in an active state
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Playing | Self::Paused)
    }

    /// Returns true if the game can accept new players
    #[must_use]
    pub const fn is_joinable(self) -> bool {
        matches!(self, Self::Lobby)
    }
}

// ============================================================================
// Element Enum
// ============================================================================

/// Elemental types for game entities and effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    /// Physical damage (no element)
    Physical,
    /// Fire element
    Fire,
    /// Ice element
    Ice,
    /// Lightning element
    Lightning,
    /// Earth element
    Earth,
    /// Wind element
    Wind,
    /// Light element
    Light,
    /// Dark element
    Dark,
}

impl Element {
    /// Returns the element that is strong against this element
    #[must_use]
    pub const fn weakness(self) -> Option<Element> {
        match self {
            Self::Fire => Some(Element::Ice),
            Self::Ice => Some(Element::Lightning),
            Self::Lightning => Some(Element::Earth),
            Self::Earth => Some(Element::Wind),
            Self::Wind => Some(Element::Fire),
            Self::Light => Some(Element::Dark),
            Self::Dark => Some(Element::Light),
            Self::Physical => None,
        }
    }

    /// Returns the element this is strong against
    #[must_use]
    pub const fn strength(self) -> Option<Element> {
        match self {
            Self::Fire => Some(Element::Wind),
            Self::Ice => Some(Element::Fire),
            Self::Lightning => Some(Element::Ice),
            Self::Earth => Some(Element::Lightning),
            Self::Wind => Some(Element::Earth),
            Self::Light => Some(Element::Dark),
            Self::Dark => Some(Element::Light),
            Self::Physical => None,
        }
    }
}

// ============================================================================
// Rarity Enum
// ============================================================================

/// Rarity tier for items, skills, and entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    /// Common - most basic tier
    Common,
    /// Uncommon - slightly better
    Uncommon,
    /// Rare - moderately rare
    Rare,
    /// Epic - very rare
    Epic,
    /// Legendary - extremely rare
    Legendary,
}

impl Rarity {
    /// Returns the numeric tier value for comparison
    #[must_use]
    pub const fn tier(self) -> u8 {
        match self {
            Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
            Self::Legendary => 4,
        }
    }

    /// Returns the color code associated with this rarity
    #[must_use]
    pub fn color_code(self) -> &'static str {
        match self {
            Self::Common => "\x1b[37m",     // White
            Self::Uncommon => "\x1b[32m",   // Green
            Self::Rare => "\x1b[34m",       // Blue
            Self::Epic => "\x1b[35m",       // Purple
            Self::Legendary => "\x1b[33m",  // Gold
        }
    }
}

impl PartialOrd for Rarity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.tier().cmp(&other.tier()))
    }
}

impl Ord for Rarity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tier().cmp(&other.tier())
    }
}

// ============================================================================
// Input Action Enum
// ============================================================================

/// Player input actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    /// No input
    None,

    // Movement
    /// Move forward
    MoveForward,
    /// Move backward
    MoveBackward,
    /// Move left (strafe)
    MoveLeft,
    /// Move right (strafe)
    MoveRight,
    /// Jump
    Jump,
    /// Crouch
    Crouch,
    /// Sprint
    Sprint,

    // Combat
    /// Basic attack
    Attack,
    /// Block/defend
    Block,
    /// Dodge roll
    Dodge,
    /// Use skill slot 1
    Skill1,
    /// Use skill slot 2
    Skill2,
    /// Use skill slot 3
    Skill3,
    /// Use skill slot 4
    Skill4,

    // Item
    /// Use consumable item
    UseItem,
    /// Interact with object
    Interact,

    // UI/System
    /// Open/close inventory
    ToggleInventory,
    /// Open/close map
    ToggleMap,
    /// Open/close menu
    ToggleMenu,
    /// Chat message
    Chat,
}

impl InputAction {
    /// Returns true if this is a movement action
    #[must_use]
    pub const fn is_movement(self) -> bool {
        matches!(
            self,
            Self::MoveForward
                | Self::MoveBackward
                | Self::MoveLeft
                | Self::MoveRight
                | Self::Jump
                | Self::Crouch
                | Self::Sprint
        )
    }

    /// Returns true if this is a combat action
    #[must_use]
    pub const fn is_combat(self) -> bool {
        matches!(
            self,
            Self::Attack | Self::Block | Self::Dodge | Self::Skill1 | Self::Skill2 | Self::Skill3 | Self::Skill4
        )
    }
}

// ============================================================================
// Player Input Struct
// ============================================================================

/// Player input state for a single frame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInput {
    /// The sequence number of this input (for server reconciliation)
    pub sequence: u32,
    /// Active input actions
    pub actions: Vec<InputAction>,
    /// View direction (pitch, yaw)
    pub view_direction: Rotation,
    /// Movement vector (x, y, z)
    pub movement: [f32; 3],
    /// Timestamp when this input was generated
    pub timestamp: u64,
}

impl PlayerInput {
    /// Creates a new player input
    #[must_use]
    pub fn new(sequence: u32) -> Self {
        Self {
            sequence,
            actions: Vec::new(),
            view_direction: [0.0, 0.0, 0.0],
            movement: [0.0, 0.0, 0.0],
            timestamp: 0,
        }
    }

    /// Returns true if a specific action is active
    #[must_use]
    pub fn has_action(&self, action: InputAction) -> bool {
        self.actions.contains(&action)
    }

    /// Adds an action to the input
    pub fn add_action(&mut self, action: InputAction) {
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
    }

    /// Removes an action from the input
    pub fn remove_action(&mut self, action: InputAction) {
        self.actions.retain(|a| a != &action);
    }

    /// Clears all actions
    pub fn clear_actions(&mut self) {
        self.actions.clear();
    }
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self::new(0)
    }
}

// ============================================================================
// Transform State Struct
// ============================================================================

/// Transform state for an entity in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransformState {
    /// Position in world space
    pub position: Position,
    /// Rotation as Euler angles (pitch, yaw, roll) in radians
    pub rotation: Rotation,
    /// Scale factor
    pub scale: f32,
}

impl TransformState {
    /// Creates a new transform state
    #[must_use]
    pub const fn new(position: Position, rotation: Rotation, scale: f32) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates a transform at origin with no rotation and unit scale
    #[must_use]
    pub fn identity() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        }
    }

    /// Returns the forward direction vector based on rotation
    #[must_use]
    pub fn forward(&self) -> [f32; 3] {
        let yaw = self.rotation[1];
        [
            yaw.sin(),
            0.0,
            -yaw.cos(),
        ]
    }

    /// Returns the right direction vector based on rotation
    #[must_use]
    pub fn right(&self) -> [f32; 3] {
        let yaw = self.rotation[1];
        [
            yaw.cos(),
            0.0,
            yaw.sin(),
        ]
    }

    /// Returns the up direction vector (always Y-up)
    #[must_use]
    pub const fn up(&self) -> [f32; 3] {
        [0.0, 1.0, 0.0]
    }

    /// Translates the transform by a given offset
    pub fn translate(&mut self, offset: [f32; 3]) {
        self.position[0] += offset[0];
        self.position[1] += offset[1];
        self.position[2] += offset[2];
    }

    /// Rotates the transform by a given Euler angle offset
    pub fn rotate(&mut self, offset: Rotation) {
        self.rotation[0] += offset[0];
        self.rotation[1] += offset[1];
        self.rotation[2] += offset[2];
    }
}

impl Default for TransformState {
    fn default() -> Self {
        Self::identity()
    }
}

// ============================================================================
// Direction Enum (existing, kept for compatibility)
// ============================================================================

/// Cardinal direction enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    /// Returns the opposite direction
    #[must_use]
    pub const fn opposite(self) -> Direction {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }

    /// Returns the vector representation of this direction
    #[must_use]
    pub fn vector(self) -> [i32; 3] {
        match self {
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::East => [1, 0, 0],
            Self::West => [-1, 0, 0],
            Self::Up => [0, 1, 0],
            Self::Down => [0, -1, 0],
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // TransformState Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_transform_default() {
        let transform = TransformState::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, 1.0);
    }

    #[test]
    fn test_transform_identity() {
        let transform = TransformState::identity();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, 1.0);
    }

    #[test]
    fn test_transform_new() {
        let pos = [1.0, 2.0, 3.0];
        let rot = [0.1, 0.2, 0.3];
        let transform = TransformState::new(pos, rot, 2.0);
        assert_eq!(transform.position, pos);
        assert_eq!(transform.rotation, rot);
        assert_eq!(transform.scale, 2.0);
    }

    #[test]
    fn test_transform_translate() {
        let mut transform = TransformState::default();
        transform.translate([1.0, 2.0, 3.0]);
        assert_eq!(transform.position, [1.0, 2.0, 3.0]);

        transform.translate([-1.0, -2.0, -3.0]);
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_transform_rotate() {
        let mut transform = TransformState::default();
        transform.rotate([0.1, 0.2, 0.3]);
        assert_eq!(transform.rotation, [0.1, 0.2, 0.3]);

        transform.rotate([-0.1, -0.2, -0.3]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_transform_forward() {
        let mut transform = TransformState::default();

        // Facing north (default)
        let forward = transform.forward();
        assert!((forward[0] - 0.0).abs() < f32::EPSILON);
        assert!((forward[1] - 0.0).abs() < f32::EPSILON);
        assert!((forward[2] - (-1.0)).abs() < f32::EPSILON);

        // Rotate 90 degrees (pi/2)
        transform.rotation[1] = std::f32::consts::PI / 2.0;
        let forward = transform.forward();
        assert!((forward[0] - 1.0).abs() < 0.001);
        assert!((forward[1] - 0.0).abs() < f32::EPSILON);
        assert!((forward[2] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_right() {
        let transform = TransformState::default();
        let right = transform.right();
        assert!((right[0] - 1.0).abs() < f32::EPSILON);
        assert!((right[1] - 0.0).abs() < f32::EPSILON);
        assert!((right[2] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_up() {
        let transform = TransformState::default();
        assert_eq!(transform.up(), [0.0, 1.0, 0.0]);
    }

    // ------------------------------------------------------------------------
    // Rarity Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_rarity_tier() {
        assert_eq!(Rarity::Common.tier(), 0);
        assert_eq!(Rarity::Uncommon.tier(), 1);
        assert_eq!(Rarity::Rare.tier(), 2);
        assert_eq!(Rarity::Epic.tier(), 3);
        assert_eq!(Rarity::Legendary.tier(), 4);
    }

    #[test]
    fn test_rarity_ordering() {
        assert!(Rarity::Common < Rarity::Uncommon);
        assert!(Rarity::Uncommon < Rarity::Rare);
        assert!(Rarity::Rare < Rarity::Epic);
        assert!(Rarity::Epic < Rarity::Legendary);
        assert!(Rarity::Common < Rarity::Legendary);
    }

    #[test]
    fn test_rarity_equality() {
        assert_eq!(Rarity::Rare, Rarity::Rare);
        assert_ne!(Rarity::Common, Rarity::Legendary);
    }

    #[test]
    fn test_rarity_sorting() {
        let mut rarities = vec![
            Rarity::Legendary,
            Rarity::Common,
            Rarity::Epic,
            Rarity::Rare,
            Rarity::Uncommon,
        ];
        rarities.sort();
        assert_eq!(
            rarities,
            vec![
                Rarity::Common,
                Rarity::Uncommon,
                Rarity::Rare,
                Rarity::Epic,
                Rarity::Legendary,
            ]
        );
    }

    // ------------------------------------------------------------------------
    // Element Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_element_weakness() {
        assert_eq!(Element::Fire.weakness(), Some(Element::Ice));
        assert_eq!(Element::Ice.weakness(), Some(Element::Lightning));
        assert_eq!(Element::Physical.weakness(), None);
    }

    #[test]
    fn test_element_strength() {
        assert_eq!(Element::Fire.strength(), Some(Element::Wind));
        assert_eq!(Element::Ice.strength(), Some(Element::Fire));
        assert_eq!(Element::Physical.strength(), None);
    }

    // ------------------------------------------------------------------------
    // GameState Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_game_state_is_active() {
        assert!(!GameState::Lobby.is_active());
        assert!(!GameState::Loading.is_active());
        assert!(GameState::Playing.is_active());
        assert!(GameState::Paused.is_active());
        assert!(!GameState::Ended.is_active());
    }

    #[test]
    fn test_game_state_is_joinable() {
        assert!(GameState::Lobby.is_joinable());
        assert!(!GameState::Loading.is_joinable());
        assert!(!GameState::Playing.is_joinable());
        assert!(!GameState::Paused.is_joinable());
        assert!(!GameState::Ended.is_joinable());
    }

    // ------------------------------------------------------------------------
    // InputAction Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_input_action_is_movement() {
        assert!(InputAction::MoveForward.is_movement());
        assert!(InputAction::MoveBackward.is_movement());
        assert!(InputAction::Jump.is_movement());
        assert!(!InputAction::Attack.is_movement());
    }

    #[test]
    fn test_input_action_is_combat() {
        assert!(InputAction::Attack.is_combat());
        assert!(InputAction::Block.is_combat());
        assert!(InputAction::Skill1.is_combat());
        assert!(!InputAction::MoveForward.is_combat());
    }

    // ------------------------------------------------------------------------
    // PlayerInput Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_player_input_new() {
        let input = PlayerInput::new(5);
        assert_eq!(input.sequence, 5);
        assert!(input.actions.is_empty());
        assert_eq!(input.view_direction, [0.0, 0.0, 0.0]);
        assert_eq!(input.movement, [0.0, 0.0, 0.0]);
        assert_eq!(input.timestamp, 0);
    }

    #[test]
    fn test_player_input_add_action() {
        let mut input = PlayerInput::new(0);
        input.add_action(InputAction::Jump);
        assert!(input.has_action(InputAction::Jump));
        assert_eq!(input.actions.len(), 1);
    }

    #[test]
    fn test_player_input_remove_action() {
        let mut input = PlayerInput::new(0);
        input.add_action(InputAction::Jump);
        input.remove_action(InputAction::Jump);
        assert!(!input.has_action(InputAction::Jump));
        assert!(input.actions.is_empty());
    }

    #[test]
    fn test_player_input_clear_actions() {
        let mut input = PlayerInput::new(0);
        input.add_action(InputAction::Jump);
        input.add_action(InputAction::Sprint);
        input.clear_actions();
        assert!(input.actions.is_empty());
    }

    #[test]
    fn test_player_input_duplicate_action() {
        let mut input = PlayerInput::new(0);
        input.add_action(InputAction::Jump);
        input.add_action(InputAction::Jump);
        assert_eq!(input.actions.len(), 1);
    }

    // ------------------------------------------------------------------------
    // Direction Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::Up.opposite(), Direction::Down);
    }

    #[test]
    fn test_direction_vector() {
        assert_eq!(Direction::North.vector(), [0, 0, -1]);
        assert_eq!(Direction::East.vector(), [1, 0, 0]);
        assert_eq!(Direction::Up.vector(), [0, 1, 0]);
    }
}
