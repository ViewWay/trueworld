// Animation system for entity sprites
//
// This module handles:
// - Animation state management
// - Frame timing and transitions
// - Facing direction tracking

use bevy::prelude::*;
use std::time::Duration;

/// Animation state for entities
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    Running,
    Attacking,
    Hurt,
    Dying,
    Dead,
    Special,
}

/// Facing direction for 2D sprites (4-directional)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FacingDirection {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

impl FacingDirection {
    /// Get the opposite direction
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Get direction as a 2D vector
    #[must_use]
    pub const fn as_vec2(self) -> Vec2 {
        match self {
            Self::Down => Vec2::new(0.0, -1.0),
            Self::Up => Vec2::new(0.0, 1.0),
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
        }
    }

    /// Get facing direction from a movement vector
    #[must_use]
    pub fn from_velocity(velocity: Vec2) -> Self {
        if velocity.abs().x > velocity.abs().y {
            if velocity.x > 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else {
            if velocity.y > 0.0 {
                Self::Up
            } else {
                Self::Down
            }
        }
    }
}

/// Animation controller component
#[derive(Component, Debug, Clone)]
pub struct AnimationController {
    /// Current animation state
    pub current_state: AnimationState,
    /// Previous animation state
    pub previous_state: AnimationState,
    /// Current facing direction
    pub facing: FacingDirection,
    /// Timer for frame animation
    pub frame_timer: Timer,
    /// Current frame index
    pub current_frame: usize,
    /// Total frames in current animation
    pub total_frames: usize,
    /// Whether animation loops
    pub loops: bool,
    /// Animation speed multiplier
    pub speed_multiplier: f32,
    /// Time since last state change (for transitions)
    pub state_time: f32,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            facing: FacingDirection::Down,
            frame_timer: Timer::new(Duration::from_millis(150), TimerMode::Repeating),
            current_frame: 0,
            total_frames: 1,
            loops: true,
            speed_multiplier: 1.0,
            state_time: 0.0,
        }
    }
}

impl AnimationController {
    /// Create a new animation controller
    #[must_use]
    pub fn new(state: AnimationState, facing: FacingDirection) -> Self {
        Self {
            current_state: state,
            previous_state: state,
            facing,
            ..Default::default()
        }
    }

    /// Change to a new animation state
    pub fn change_state(&mut self, new_state: AnimationState) {
        if self.current_state != new_state {
            self.previous_state = self.current_state;
            self.current_state = new_state;
            self.current_frame = 0;
            self.state_time = 0.0;
            self.frame_timer.reset();

            // Configure animation based on state
            match new_state {
                AnimationState::Idle => {
                    self.total_frames = 1;
                    self.loops = true;
                }
                AnimationState::Walking => {
                    self.total_frames = 4;
                    self.loops = true;
                }
                AnimationState::Running => {
                    self.total_frames = 4;
                    self.loops = true;
                }
                AnimationState::Attacking => {
                    self.total_frames = 3;
                    self.loops = false;
                }
                AnimationState::Hurt => {
                    self.total_frames = 2;
                    self.loops = false;
                }
                AnimationState::Dying => {
                    self.total_frames = 4;
                    self.loops = false;
                }
                AnimationState::Dead => {
                    self.total_frames = 1;
                    self.loops = true;
                }
                AnimationState::Special => {
                    self.total_frames = 4;
                    self.loops = false;
                }
            }
        }
    }

    /// Set the facing direction
    pub fn set_facing(&mut self, direction: FacingDirection) {
        self.facing = direction;
    }

    /// Update facing direction based on velocity
    pub fn update_facing_from_velocity(&mut self, velocity: Vec2) {
        if velocity.length_squared() > 0.01 {
            self.facing = FacingDirection::from_velocity(velocity);
        }
    }

    /// Tick the animation timer
    pub fn tick(&mut self, delta: Duration) -> bool {
        let adjusted_delta = Duration::from_secs_f32(
            delta.as_secs_f32() * self.speed_multiplier
        );
        let finished = self.frame_timer.tick(adjusted_delta).just_finished();
        self.state_time += delta.as_secs_f32();

        if finished {
            if self.loops || self.current_frame < self.total_frames - 1 {
                self.current_frame = (self.current_frame + 1) % self.total_frames;
            } else if !self.loops && self.current_frame == self.total_frames - 1 {
                // Animation finished, potentially transition
                match self.current_state {
                    AnimationState::Dying => {
                        self.change_state(AnimationState::Dead);
                    }
                    AnimationState::Attacking | AnimationState::Hurt | AnimationState::Special => {
                        self.change_state(AnimationState::Idle);
                    }
                    _ => {}
                }
            }
        }

        finished
    }

    /// Get current animation progress (0.0 to 1.0)
    #[must_use]
    pub fn progress(&self) -> f32 {
        if self.total_frames > 0 {
            self.current_frame as f32 / self.total_frames as f32
        } else {
            0.0
        }
    }

    /// Check if animation is finished
    #[must_use]
    pub fn is_finished(&self) -> bool {
        !self.loops && self.current_frame == self.total_frames - 1
    }

    /// Set animation speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed_multiplier = speed.max(0.1);
    }
}

/// Plugin for animation system
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animations);
    }
}

/// Update all animation controllers
fn update_animations(
    mut query: Query<&mut AnimationController>,
    time: Res<Time>,
) {
    for mut controller in query.iter_mut() {
        controller.tick(time.delta());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facing_direction_opposite() {
        assert_eq!(FacingDirection::Down.opposite(), FacingDirection::Up);
        assert_eq!(FacingDirection::Left.opposite(), FacingDirection::Right);
    }

    #[test]
    fn test_facing_direction_from_velocity() {
        assert_eq!(FacingDirection::from_velocity(Vec2::new(1.0, 0.0)), FacingDirection::Right);
        assert_eq!(FacingDirection::from_velocity(Vec2::new(-1.0, 0.0)), FacingDirection::Left);
        assert_eq!(FacingDirection::from_velocity(Vec2::new(0.0, 1.0)), FacingDirection::Up);
        assert_eq!(FacingDirection::from_velocity(Vec2::new(0.0, -1.0)), FacingDirection::Down);
    }

    #[test]
    fn test_animation_controller_default() {
        let controller = AnimationController::default();
        assert_eq!(controller.current_state, AnimationState::Idle);
        assert_eq!(controller.facing, FacingDirection::Down);
    }

    #[test]
    fn test_animation_state_change() {
        let mut controller = AnimationController::default();
        controller.change_state(AnimationState::Walking);
        assert_eq!(controller.current_state, AnimationState::Walking);
        assert_eq!(controller.total_frames, 4);
    }
}
