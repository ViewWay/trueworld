// Channel configuration for network communication

use renet::{ChannelConfig, SendType};
use std::time::Duration;

/// Network channel types for different packet categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkChannel {
    /// Reliable ordered - for critical game state changes
    ReliableOrdered = 0,
    /// Reliable unordered - for non-critical but important data
    ReliableUnordered = 1,
    /// Unreliable - for frequent position updates (can interpolate)
    Unreliable = 2,
}

impl NetworkChannel {
    /// Get the default channel configurations for renet 0.0.16
    pub fn default_configs() -> [ChannelConfig; 3] {
        [
            Self::reliable_ordered(0),
            Self::reliable_unordered(1),
            Self::unreliable(2),
        ]
    }

    /// Reliable ordered channel configuration
    pub fn reliable_ordered(id: u8) -> ChannelConfig {
        ChannelConfig {
            channel_id: id,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(100),
            },
        }
    }

    /// Reliable unordered channel configuration
    pub fn reliable_unordered(id: u8) -> ChannelConfig {
        ChannelConfig {
            channel_id: id,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableUnordered {
                resend_time: Duration::from_millis(100),
            },
        }
    }

    /// Unreliable channel configuration
    pub fn unreliable(id: u8) -> ChannelConfig {
        ChannelConfig {
            channel_id: id,
            max_memory_usage_bytes: 10 * 1024 * 1024,
            send_type: SendType::Unreliable,
        }
    }
}

impl From<NetworkChannel> for u8 {
    fn from(channel: NetworkChannel) -> Self {
        channel as u8
    }
}

impl TryFrom<u8> for NetworkChannel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NetworkChannel::ReliableOrdered),
            1 => Ok(NetworkChannel::ReliableUnordered),
            2 => Ok(NetworkChannel::Unreliable),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_conversion() {
        assert_eq!(u8::from(NetworkChannel::ReliableOrdered), 0);
        assert_eq!(u8::from(NetworkChannel::Unreliable), 2);
    }

    #[test]
    fn test_channel_try_from() {
        assert_eq!(
            NetworkChannel::try_from(1),
            Ok(NetworkChannel::ReliableUnordered)
        );
        assert!(NetworkChannel::try_from(99).is_err());
    }

    #[test]
    fn test_default_configs_length() {
        let configs = NetworkChannel::default_configs();
        assert_eq!(configs.len(), 3);
    }
}
