// services/signaling/src/signaling.rs

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use uuid::Uuid;
use super::SignalingMessage;

/// 信令服务器配置
#[derive(Debug, Clone)]
pub struct SignalingConfig {
    pub max_peers: usize,
    pub peer_timeout_secs: u64,
}

impl Default for SignalingConfig {
    fn default() -> Self {
        Self {
            max_peers: 1000,
            peer_timeout_secs: 300,
        }
    }
}

/// 连接的对等端信息
#[derive(Clone)]
pub struct PeerInfo {
    pub peer_id: Uuid,
    pub room_id: Option<String>,
    pub connected_at: std::time::Instant,
    pub sender: Option<tokio::sync::mpsc::UnboundedSender<SignalingMessage>>,
}

/// 信令服务器
pub struct SignalingServer {
    config: SignalingConfig,
    peers: HashMap<Uuid, PeerInfo>,
    rooms: HashMap<String, Room>,
}

/// 房间
pub struct Room {
    pub room_id: String,
    pub peers: Vec<Uuid>,
    pub created_at: std::time::Instant,
}

impl SignalingServer {
    pub fn new(config: SignalingConfig) -> Self {
        Self {
            config,
            peers: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    /// 添加对等端
    pub fn add_peer(&mut self, peer_id: Uuid) -> Result<(), SignalingError> {
        if self.peers.len() >= self.config.max_peers {
            return Err(SignalingError::ServerFull);
        }

        self.peers.insert(peer_id, PeerInfo {
            peer_id,
            room_id: None,
            connected_at: std::time::Instant::now(),
            sender: None,
        });

        Ok(())
    }

    /// 移除对等端
    pub fn remove_peer(&mut self, peer_id: &Uuid) {
        if let Some(peer) = self.peers.remove(peer_id) {
            // 从房间中移除
            if let Some(room_id) = peer.room_id {
                if let Some(room) = self.rooms.get_mut(&room_id) {
                    room.peers.retain(|id| id != peer_id);

                    // 如果房间为空，删除房间
                    if room.peers.is_empty() {
                        self.rooms.remove(&room_id);
                    }
                }
            }
        }
    }

    /// 获取对等端
    pub fn get_peer(&self, peer_id: &str) -> Option<&PeerInfo> {
        let id = Uuid::parse_str(peer_id).ok()?;
        self.peers.get(&id)
    }

    /// 加入房间
    pub fn join_room(&mut self, peer_id: Uuid, room_id: String) -> Result<(), SignalingError> {
        let room = self.rooms.entry(room_id.clone()).or_insert_with(|| Room {
            room_id,
            peers: Vec::new(),
            created_at: std::time::Instant::now(),
        });

        if room.peers.len() >= 16 {
            return Err(SignalingError::RoomFull);
        }

        room.peers.push(peer_id);

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.room_id = Some(room_id);
        }

        Ok(())
    }

    /// 离开房间
    pub fn leave_room(&mut self, peer_id: &Uuid) {
        if let Some(peer) = self.peers.get(peer_id) {
            if let Some(room_id) = &peer.room_id {
                if let Some(room) = self.rooms.get_mut(room_id) {
                    room.peers.retain(|id| id != peer_id);
                }
            }
        }
    }

    /// 获取房间中的所有对等端
    pub fn get_room_peers(&self, room_id: &str) -> Vec<&PeerInfo> {
        if let Some(room) = self.rooms.get(room_id) {
            room.peers.iter()
                .filter_map(|id| self.peers.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SignalingError {
    #[error("Server is full")]
    ServerFull,

    #[error("Room is full")]
    RoomFull,

    #[error("Peer not found")]
    PeerNotFound,

    #[error("Room not found")]
    RoomNotFound,
}
