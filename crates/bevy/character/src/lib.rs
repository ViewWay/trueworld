// bevy-character: Character system plugin for Bevy

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CharacterSystem {
    Update,
    Animate,
    State,
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, _app: &mut App) {
        // Character systems will be added here
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
