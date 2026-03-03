// crates/server/src/config.rs

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 绑定地址
    pub bind_address: String,

    /// 端口
    pub port: u16,

    /// 最大房间数
    pub max_rooms: usize,

    /// 每个房间最大玩家数
    pub max_players_per_room: usize,

    /// 服务器 Tick 率
    pub tick_rate: u64,

    /// 数据库配置
    pub database: DatabaseConfig,

    /// Redis 配置
    pub redis: Option<RedisConfig>,

    /// 认证配置
    pub auth: AuthConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// 指标配置
    pub metrics: Option<MetricsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL 连接 URL
    pub url: String,

    /// 最大连接数
    pub max_connections: u32,

    /// 最小连接数
    pub min_connections: u32,

    /// 连接超时 (秒)
    pub connect_timeout: u64,

    /// 空闲超时 (秒)
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis 连接 URL
    pub url: String,

    /// 最大连接数
    pub max_connections: u32,

    /// 键前缀
    pub key_prefix: String,

    /// 缓存 TTL (秒)
    pub cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT 密钥
    pub jwt_secret: String,

    /// Token 过期时间 (秒)
    pub token_expiry: u64,

    /// 是否启用验证
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,

    /// 是否输出到文件
    pub file: bool,

    /// 日志文件路径
    pub file_path: Option<String>,

    /// 是否输出到控制台
    pub console: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Prometheus 端口
    pub port: u16,

    /// 指标路径
    pub path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 5000,
            max_rooms: 64,
            max_players_per_room: 16,
            tick_rate: 60,
            database: DatabaseConfig::default(),
            redis: None,
            auth: AuthConfig::default(),
            logging: LoggingConfig::default(),
            metrics: None,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/trueworld".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout: 30,
            idle_timeout: 600,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "change-me-in-production".to_string(),
            token_expiry: 86400, // 24 hours
            enabled: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: true,
            file_path: Some("logs/trueworld.log".to_string()),
            console: true,
        }
    }
}

impl ServerConfig {
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 加载配置 (从文件或环境变量)
    pub fn load() -> anyhow::Result<Self> {
        // 尝试从配置文件加载
        if let Ok(config) = Self::load_from_file("config/server.toml") {
            return Ok(config);
        }

        // 从环境变量加载
        let config = ServerConfig {
            bind_address: std::env::var("SERVER_BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5000),
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://localhost/trueworld".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        Ok(config)
    }
}
