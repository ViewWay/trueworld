// Tests for core ID types

use trueworld_core::PlayerId;

#[test]
fn test_player_id_new() {
    let id = PlayerId::new(12345);
    assert_eq!(id.raw(), 12345);
}

#[test]
fn test_player_id_from() {
    let raw = 12345u64;
    let id = PlayerId::new(raw);
    assert_eq!(id.raw(), raw);
}

#[test]
fn test_player_id_equality() {
    let id1 = PlayerId::new(100);
    let id2 = PlayerId::new(100);
    let id3 = PlayerId::new(200);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_player_id_display() {
    let id = PlayerId::new(42);
    assert_eq!(format!("{}", id), "Player(42)");
}
