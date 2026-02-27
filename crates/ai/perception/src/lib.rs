// trueworld-perception: Camera-based perception for TrueWorld
//
// This crate handles camera input processing for motion detection,
// gesture recognition, and object tracking.

pub mod camera;
pub mod detector;
pub mod gesture;
pub mod tracker;

pub use camera::{CameraCapture, CameraConfig};
pub use detector::{DetectorPipeline, MotionDetector, SkinDetector};
pub use gesture::{GestureRecognizer, GestureState};
pub use tracker::{ObjectTracker, TrackedObject};

use std::time::Instant;

/// Perception engine configuration
#[derive(Debug, Clone)]
pub struct PerceptionConfig {
    pub camera: CameraConfig,
    pub enable_motion: bool,
    pub enable_skin: bool,
    pub enable_tracking: bool,
    pub smoothing: bool,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            camera: CameraConfig::default(),
            enable_motion: true,
            enable_skin: false,
            enable_tracking: true,
            smoothing: true,
        }
    }
}

/// Perception event
#[derive(Debug, Clone)]
pub enum PerceptionEvent {
    MotionDetected {
        regions: Vec<MotionRegion>,
        timestamp: Instant,
    },
    GestureRecognized {
        gesture: GestureState,
        confidence: f32,
        hand_position: (f32, f32),
    },
    ObjectTracked {
        id: u32,
        position: (f32, f32),
        velocity: (f32, f32),
        trajectory: Vec<(f32, f32)>,
    },
}

#[derive(Debug, Clone)]
pub struct MotionRegion {
    pub bbox: BoundingBox,
    pub magnitude: f32,
    pub center: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = PerceptionConfig::default();
        assert!(config.enable_motion);
        assert!(config.enable_tracking);
    }
}
