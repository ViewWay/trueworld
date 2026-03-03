// crates/server/src/movement/mod.rs
//
// Server-side movement validation and position management system.
// Provides anti-cheat validation, position updates, and broadcast coordination.

mod config;
mod validation;
mod update;



pub use update::{
    MovementUpdateProcessor,
    ProcessInputResult,
};
