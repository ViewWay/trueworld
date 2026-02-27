// bevy-voice-camera: Voice and camera input plugin for Bevy

use bevy::prelude::*;

pub struct VoiceCameraPlugin;

impl Plugin for VoiceCameraPlugin {
    fn build(&self, _app: &mut App) {
        // Voice and camera input systems will be added here
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
