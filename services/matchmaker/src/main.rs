// Matchmaker service for TrueWorld

use std::sync::Arc;
use std::time::Instant;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Matchmaker configuration
#[derive(Debug, Clone)]
pub struct MatchmakerConfig {
    pub bind_address: String,
    pub redis_url: String,
}

impl MatchmakerConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            bind_address: "0.0.0.0:3002".to_string(),
            redis_url: "redis://127.0.0.1".to_string(),
        })
    }
}

/// Match ticket
#[derive(Debug, Clone)]
pub struct MatchTicket {
    pub ticket_id: Uuid,
    pub player_id: String,
    pub player_name: String,
    pub created_at: Instant,
}

/// Matchmaker service
pub struct Matchmaker {
    _config: MatchmakerConfig,
    tickets: RwLock<Vec<MatchTicket>>,
}

impl Matchmaker {
    pub fn new(config: MatchmakerConfig) -> Self {
        Self {
            _config: config,
            tickets: RwLock::new(Vec::new()),
        }
    }

    pub fn create_ticket(&mut self, req: CreateTicketRequest) -> Uuid {
        let ticket_id = Uuid::new_v4();
        let ticket = MatchTicket {
            ticket_id,
            player_id: req.player_id,
            player_name: req.player_name,
            created_at: Instant::now(),
        };
        self.tickets.blocking_write().push(ticket);
        ticket_id
    }

    pub fn get_ticket_status(&self, ticket_id: Uuid) -> TicketStatusResponse {
        let tickets = self.tickets.blocking_read();
        let exists = tickets.iter().any(|t| t.ticket_id == ticket_id);

        TicketStatusResponse {
            ticket_id: ticket_id.to_string(),
            status: if exists { "waiting".to_string() } else { "unknown".to_string() },
            server: None,
            position: None,
            estimated_wait: 0,
        }
    }

    pub fn get_queue_stats(&self, _ticket_id: &Uuid) -> (u32, u32) {
        let tickets = self.tickets.blocking_read();
        (tickets.len() as u32, tickets.len() as u32)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub room_id: String,
}

/// Create ticket request
#[derive(Deserialize)]
pub struct CreateTicketRequest {
    pub player_id: String,
    pub player_name: String,
    pub level: u32,
    pub region: Option<String>,
    pub modes: Vec<String>,
    pub party_id: Option<String>,
    pub party_size: Option<u32>,
}

/// Create ticket response
#[derive(Serialize)]
pub struct CreateTicketResponse {
    pub ticket_id: String,
    pub estimated_wait: u32,
    pub queue_size: u32,
}

/// Ticket status response
#[derive(Serialize)]
pub struct TicketStatusResponse {
    pub ticket_id: String,
    pub status: String,
    pub server: Option<ServerInfo>,
    pub position: Option<u32>,
    pub estimated_wait: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting Matchmaker Service...");

    let config = MatchmakerConfig::load()?;
    let matchmaker = Arc::new(RwLock::new(Matchmaker::new(config)));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/match/ticket", post(create_ticket))
        .route("/match/ticket/:ticket_id", get(get_ticket_status))
        .with_state(matchmaker);

    let addr = "0.0.0.0:3002";
    tracing::info!("Matchmaker service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "matchmaker"
    }))
}

async fn create_ticket(
    State(matchmaker): State<Arc<RwLock<Matchmaker>>>,
    Json(req): Json<CreateTicketRequest>,
) -> impl IntoResponse {
    let mut mm = matchmaker.write().await;
    let ticket = mm.create_ticket(req);
    let (estimated_wait, queue_size) = mm.get_queue_stats(&ticket);

    Json(CreateTicketResponse {
        ticket_id: ticket.to_string(),
        estimated_wait,
        queue_size,
    })
}

async fn get_ticket_status(
    State(matchmaker): State<Arc<RwLock<Matchmaker>>>,
    Path(ticket_id): Path<String>,
) -> impl IntoResponse {
    let mm = matchmaker.read().await;
    let id = match Uuid::parse_str(&ticket_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(TicketStatusResponse {
                    ticket_id,
                    status: "invalid".to_string(),
                    server: None,
                    position: None,
                    estimated_wait: 0,
                }),
            )
        }
    };

    (StatusCode::OK, Json(mm.get_ticket_status(id)))
}
