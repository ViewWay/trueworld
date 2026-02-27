// services/matchmaker/src/matchmaker.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use super::{CreateTicketRequest, TicketStatusResponse, ServerInfo};

/// 匹配器
pub struct Matchmaker {
    config: MatchmakerConfig,
    tickets: HashMap<Uuid, Ticket>,
    queues: HashMap<String, Vec<Uuid>>, // mode -> ticket_ids
    matches: HashMap<Uuid, Match>,
    ticket_counter: u64,
}

#[derive(Clone)]
pub struct Ticket {
    pub ticket_id: Uuid,
    pub player_id: String,
    pub player_name: String,
    pub level: u32,
    pub region: Option<String>,
    pub modes: Vec<String>,
    pub party_id: Option<String>,
    pub party_size: Option<u32>,
    pub created_at: Instant,
    pub status: TicketStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TicketStatus {
    Waiting,
    Matched,
    Expired,
    Cancelled,
}

pub struct Match {
    pub match_id: Uuid,
    pub tickets: Vec<Uuid>,
    pub server: ServerInfo,
    pub created_at: Instant,
}

#[derive(Clone)]
pub struct MatchmakerConfig {
    pub ticket_ttl: Duration,
    pub match_timeout: Duration,
    pub max_queue_size: usize,
    pub min_players_per_match: usize,
    pub max_players_per_match: usize,
    pub level_range: u32, // 玩家等级差异范围
}

impl Default for MatchmakerConfig {
    fn default() -> Self {
        Self {
            ticket_ttl: Duration::from_secs(300),
            match_timeout: Duration::from_secs(30),
            max_queue_size: 1000,
            min_players_per_match: 2,
            max_players_per_match: 16,
            level_range: 10,
        }
    }
}

impl MatchmakerConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self::default())
    }
}

impl Matchmaker {
    pub fn new(config: MatchmakerConfig) -> Self {
        Self {
            config,
            tickets: HashMap::new(),
            queues: HashMap::new(),
            matches: HashMap::new(),
            ticket_counter: 0,
        }
    }

    /// 创建匹配票据
    pub fn create_ticket(&mut self, req: CreateTicketRequest) -> Uuid {
        let ticket_id = Uuid::new_v4();
        let ticket = Ticket {
            ticket_id,
            player_id: req.player_id,
            player_name: req.player_name,
            level: req.level,
            region: req.region,
            modes: req.modes.clone(),
            party_id: req.party_id,
            party_size: req.party_size,
            created_at: Instant::now(),
            status: TicketStatus::Waiting,
        };

        self.tickets.insert(ticket_id, ticket.clone());

        // 添加到队列
        for mode in &req.modes {
            self.queues
                .entry(mode.clone())
                .or_insert_with(Vec::new)
                .push(ticket_id);
        }

        ticket_id
    }

    /// 获取队列统计
    pub fn get_queue_stats(&self, ticket_id: &Uuid) -> (u32, u32) {
        if let Some(ticket) = self.tickets.get(ticket_id) {
            if let Some(queue) = ticket.modes.first().and_then(|m| self.queues.get(m)) {
                let position = queue.iter().position(|id| id == ticket_id).unwrap_or(0) as u32;
                let estimated_wait = position * 5; // 每个玩家估计5秒
                return (estimated_wait, queue.len() as u32);
            }
        }
        (0, 0)
    }

    /// 获取票据状态
    pub fn get_ticket_status(&self, ticket_id: &Uuid) -> TicketStatusResponse {
        if let Some(ticket) = self.tickets.get(ticket_id) {
            let status_str = match ticket.status {
                TicketStatus::Waiting => "waiting",
                TicketStatus::Matched => "matched",
                TicketStatus::Expired => "expired",
                TicketStatus::Cancelled => "cancelled",
            }.to_string();

            let server = if let Some(m) = self.matches.values().find(|m| m.tickets.contains(ticket_id)) {
                Some(m.server.clone())
            } else {
                None
            };

            let (estimated_wait, queue_size) = self.get_queue_stats(ticket_id);

            TicketStatusResponse {
                ticket_id: ticket_id.to_string(),
                status: status_str,
                server,
                position: None,
                estimated_wait,
            }
        } else {
            TicketStatusResponse {
                ticket_id: ticket_id.to_string(),
                status: "invalid".to_string(),
                server: None,
                position: None,
                estimated_wait: 0,
            }
        }
    }

    /// 加入匹配
    pub fn join_match(&mut self, ticket_id: &Uuid) -> Result<ServerInfo, MatchError> {
        // 清理过期票据
        self.cleanup_tickets();

        // 查找票据
        let ticket = self.tickets.get(ticket_id)
            .ok_or(MatchError::TicketNotFound)?;

        if ticket.status != TicketStatus::Waiting {
            return Err(MatchError::TicketNotWaiting);
        }

        // 检查是否已匹配
        if let Some(match_) = self.matches.values().find(|m| m.tickets.contains(ticket_id)) {
            return Ok(match_.server.clone());
        }

        // 尝试创建匹配
        let mode = ticket.modes.first().ok_or(MatchError::NoMode)?;
        let queue = self.queues.get(mode).ok_or(MatchError::QueueNotFound)?;

        // 查找匹配的玩家
        let matched_players = self.find_match(ticket, queue)?;

        // 创建匹配
        let match_id = Uuid::new_v4();
        let room_id = Uuid::new_v4().to_string();

        // 服务器地址 (从游戏服务器列表中选择)
        let server = self.select_server()?;

        let match_ = Match {
            match_id,
            tickets: matched_players.clone(),
            server: server.clone(),
            created_at: Instant::now(),
        };

        self.matches.insert(match_id, match_);

        // 更新票据状态
        for player_id in &matched_players {
            if let Some(ticket) = self.tickets.get_mut(player_id) {
                ticket.status = TicketStatus::Matched;
            }
        }

        Ok(server)
    }

    /// 查找匹配的玩家
    fn find_match(&self, ticket: &Ticket, queue: &[Uuid]) -> Result<Vec<Uuid>, MatchError> {
        let mut matched = vec![*ticket.ticket_id];

        // 如果是组队
        if let Some(party_id) = &ticket.party_id {
            let party_size = ticket.party_size.unwrap_or(1) as usize;

            // 查找同队的所有票据
            for other_id in queue {
                if matched.len() >= party_size {
                    break;
                }

                if let Some(other_ticket) = self.tickets.get(other_id) {
                    if other_ticket.party_id.as_ref() == Some(party_id) {
                        matched.push(*other_id);
                    }
                }
            }
        } else {
            // 单人匹配，找其他单人玩家
            let target_size = (self.config.min_players_per_match).max(2);

            for other_id in queue {
                if matched.len() >= target_size {
                    break;
                }

                if let Some(other_ticket) = self.tickets.get(other_id) {
                    // 检查等级范围
                    let level_diff = (ticket.level as i32 - other_ticket.level as i32).abs();
                    if level_diff <= self.config.level_range as i32 {
                        matched.push(*other_id);
                    }
                }
            }
        }

        if matched.len() < self.config.min_players_per_match {
            return Err(MatchError::NotEnoughPlayers);
        }

        Ok(matched)
    }

    /// 选择游戏服务器
    fn select_server(&self) -> Result<ServerInfo, MatchError> {
        // TODO: 从服务器列表中选择负载最低的
        Ok(ServerInfo {
            id: "server-1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 5000,
            room_id: Uuid::new_v4().to_string(),
        })
    }

    /// 清理过期票据
    fn cleanup_tickets(&mut self) {
        let now = Instant::now();

        self.tickets.retain(|_, ticket| {
            if now.duration_since(ticket.created_at) > self.config.ticket_ttl {
                // 从队列中移除
                for mode in &ticket.modes {
                    if let Some(queue) = self.queues.get_mut(mode) {
                        queue.retain(|id| id != &ticket.ticket_id);
                    }
                }
                false
            } else {
                true
            }
        });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MatchError {
    #[error("Ticket not found")]
    TicketNotFound,

    #[error("Ticket is not in waiting status")]
    TicketNotWaiting,

    #[error("No game mode specified")]
    NoMode,

    #[error("Queue not found")]
    QueueNotFound,

    #[error("Not enough players to match")]
    NotEnoughPlayers,

    #[error("Server not available")]
    ServerNotAvailable,
}
