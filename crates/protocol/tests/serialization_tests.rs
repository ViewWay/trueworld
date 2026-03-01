// Protocol serialization tests

use trueworld_protocol::{serialize_packet, deserialize_packet, ClientHello, PROTOCOL_VERSION};
use chrono::Utc;

#[test]
fn test_packet_serialization_roundtrip() {
    let original = ClientHello {
        protocol_version: PROTOCOL_VERSION,
        username: "TestPlayer".to_string(),
        timestamp: Utc::now(),
    };

    let bytes = serialize_packet(&original).expect("Serialization failed");
    let decoded: ClientHello = deserialize_packet(&bytes).expect("Deserialization failed");

    assert_eq!(original.username, decoded.username);
    assert_eq!(original.protocol_version, decoded.protocol_version);
}

#[test]
fn test_empty_username_serialization() {
    let hello = ClientHello {
        protocol_version: PROTOCOL_VERSION,
        username: String::new(),
        timestamp: Utc::now(),
    };

    let bytes = serialize_packet(&hello).expect("Serialization failed");
    let decoded: ClientHello = deserialize_packet(&bytes).expect("Deserialization failed");

    assert_eq!(decoded.username, "");
}
