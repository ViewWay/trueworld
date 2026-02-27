// bevy-combat: Combat system plugin for Bevy

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CombatSystem {
    Input,
    Process,
    Update,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, _app: &mut App) {
        // Combat systems will be added here
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
