// Client movement module for TrueWorld
//
// This module handles:
// - Local prediction of player movement
// - Sending input to server
// - Server position reconciliation
// - Smooth position correction

pub mod plugin;
pub mod prediction;
pub mod correction;

// Re-export commonly used types
pub use plugin::{ClientMovementPlugin, MovementConfig};


/// Maximum input history size (frames)
pub const MAX_INPUT_HISTORY: usize = 60;

/// Default position correction threshold (world units)
pub const DEFAULT_CORRECTION_THRESHOLD: f32 = 2.0;

/// Default correction lerp factor (0-1)
pub const DEFAULT_CORRECTION_LERP: f32 = 0.3;
