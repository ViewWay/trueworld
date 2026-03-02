// crates/server/src/movement/mod.rs
//
// Server-side movement validation and position management system.
// Provides anti-cheat validation, position updates, and broadcast coordination.

mod config;
mod validation;
mod update;

pub use config::{
    ServerMovementConfig,
    PlayerViolationTracker,
    ViolationManager,
};

pub use validation::{
    ValidationResult,
    ServerPlayerMovement,
    MovementValidator,
};

pub use update::{
    MovementUpdateProcessor,
    ProcessInputResult,
};
