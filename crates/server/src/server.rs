// crates/server/src/server.rs

use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::Arc,
    time::Duration,
};

use tracing::{info, error, warn};

use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{ServerAuthentication, NetcodeServerTransport};
use renet_netcode::ServerConfig as NetcodeServerConfig;
use tokio::{
    sync::{mpsc, RwLock, oneshot},
    task::JoinHandle,
    time::interval,
};
use trueworld_core::*;
use trueworld_protocol::*;

use crate::{
    config::ServerConfig,
    database::{DatabaseManager, PlayerData},
    game::GameWorld,
    network::{NetworkPacket, ServerNetwork},
    player::{Player, PlayerSession},
    room::{Room, RoomManager},
    shutdown::ShutdownManager,
};

/// TrueWorld 服务器
pub struct TrueWorldServer {
    config: ServerConfig,

    /// Renet 服务器
    renet_server: RenetServer,

    /// 传输层
    transport: NetcodeServerTransport,

    /// 房间管理器
    room_manager: Arc<RwLock<RoomManager>>,

    /// 数据库管理器
    database: Arc<DatabaseManager>,

    /// 在线玩家
    players: Arc<RwLock<HashMap<PlayerId, Player>>>,

    /// 网络任务句柄
    network_handle: Option<JoinHandle<()>>,

    /// 游戏循环句柄
    game_handle: Option<JoinHandle<()>>,

    /// 关闭信号
    shutdown: ShutdownManager,
}

impl TrueWorldServer {
    /// 创建新服务器
    pub async fn new(config: ServerConfig) -> anyhow::Result<Self> {
        info!("Initializing TrueWorld Server...");

        // 初始化数据库
        let database = Arc::new(DatabaseManager::new(&config.database).await?);

        // 创建 Renet 服务器
        let connection_config = Self::create_connection_config();
        let renet_server = RenetServer::new(connection_config);

        // 创建传输层
        let socket = UdpSocket::bind(format!("{}:{}", config.bind_address, config.port))?;
        let server_addr = socket.local_addr()?;

        let netcode_config = NetcodeServerConfig {
            current_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap(),
            max_clients: config.max_players_per_room * config.max_rooms,
            protocol_id: 0,
            public_addresses: vec![server_addr],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = NetcodeServerTransport::new(netcode_config, socket)
            .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;

        // 初始化房间管理器
        let room_manager = Arc::new(RwLock::new(RoomManager::new(config.max_rooms, config.max_players_per_room)));

        // 初始化玩家映射
        let players = Arc::new(RwLock::new(HashMap::new()));

        // 初始化关闭管理器
        let shutdown = ShutdownManager::new();

        info!("Server initialized on {}:{}", config.bind_address, config.port);

        Ok(Self {
            config,
            renet_server,
            transport,
            room_manager,
            database,
            players,
            network_handle: None,
            game_handle: None,
            shutdown,
        })
    }

    /// 运行服务器
    pub async fn run(mut self) -> anyhow::Result<()> {
        info!("Starting TrueWorld Server...");

        // 创建网络通道
        let (network_tx, mut network_rx) = mpsc::unbounded_channel();
        let (game_tx, game_rx) = mpsc::unbounded_channel();

        // 启动网络任务
        let network_handle = Self::spawn_network_task(
            self.renet_server,
            self.transport,
            network_tx.clone(),
            game_rx,
            self.shutdown.subscribe(),
        );

        // 启动游戏循环任务
        let game_handle = Self::spawn_game_task(
            self.room_manager.clone(),
            self.players.clone(),
            self.database.clone(),
            game_tx.clone(),
            network_rx,
            self.config.tick_rate,
            self.shutdown.subscribe(),
        );

        self.network_handle = Some(network_handle);
        self.game_handle = Some(game_handle);

        // 等待关闭信号
        self.shutdown.wait().await;

        info!("Shutting down TrueWorld Server...");

        // 等待任务完成
        if let Some(handle) = self.network_handle.take() {
            handle.await.ok();
        }
        if let Some(handle) = self.game_handle.take() {
            handle.await.ok();
        }

        info!("TrueWorld Server stopped");

        Ok(())
    }

    /// 启动网络处理任务
    fn spawn_network_task(
        mut renet_server: RenetServer,
        mut transport: NetcodeServerTransport,
        network_tx: mpsc::UnboundedSender<NetworkPacket>,
        mut game_rx: mpsc::UnboundedReceiver<ServerMessage>,
        shutdown: oneshot::Receiver<()>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut shutdown = shutdown;

            loop {
                tokio::select! {
                    _ = &mut shutdown => {
                        info!("Network task shutting down");
                        break;
                    }

                    result = tokio::time::timeout(Duration::from_millis(16), async {
                        // 处理网络事件
                        while let Some(event) = renet_server.get_event() {
                            match event {
                                ServerEvent::ClientConnected { client_id } => {
                                    info!("Client connected: {}", client_id);
                                    network_tx.send(NetworkPacket::ClientConnected { client_id })
                                        .ok();
                                }
                                ServerEvent::ClientDisconnected { client_id, reason } => {
                                    info!("Client disconnected: {} - {:?}", client_id, reason);
                                    network_tx.send(NetworkPacket::ClientDisconnected { client_id })
                                        .ok();
                                }
                            }
                        }

                        // 接收客户端消息
                        for client_id in renet_server.clients_id() {
                            while let Some(message) = renet_server.receive_message(client_id, 0) {
                                if let Ok(packet) = bincode::deserialize::<ClientMessage>(&message) {
                                    network_tx.send(NetworkPacket::Message { client_id, message: packet })
                                        .ok();
                                }
                            }

                            while let Some(message) = renet_server.receive_message(client_id, 2) {
                                if let Ok(packet) = bincode::deserialize::<ClientMessage>(&message) {
                                    network_tx.send(NetworkPacket::Message { client_id, message: packet })
                                        .ok();
                                }
                            }
                        }

                        // 发送游戏消息
                        while let Some(msg) = game_rx.recv().await {
                            let serialized = match bincode::serialize(&msg) {
                                Ok(data) => data,
                                Err(e) => {
                                    error!("Failed to serialize message: {}", e);
                                    continue;
                                }
                            };

                            match msg {
                                ServerMessage::WorldUpdate(_) => {
                                    // 世界更新用不可靠通道
                                    renet_server.broadcast_message(2, serialized);
                                }
                                _ => {
                                    // 其他消息用可靠通道
                                    renet_server.broadcast_message(0, serialized);
                                }
                            }
                        }

                        // 更新传输层
                        let duration = Duration::from_secs_f64(1.0 / 60.0);
                        transport.update(duration, &mut renet_server).ok();

                        Ok::<_, anyhow::Error>(())
                    }) => {
                        if let Err(_) = result {
                            // Timeout is expected
                        }
                    }
                }
            }
        })
    }

    /// 启动游戏循环任务
    fn spawn_game_task(
        room_manager: Arc<RwLock<RoomManager>>,
        players: Arc<RwLock<HashMap<PlayerId, Player>>>,
        database: Arc<DatabaseManager>,
        game_tx: mpsc::UnboundedSender<ServerMessage>,
        mut network_rx: mpsc::UnboundedReceiver<NetworkPacket>,
        tick_rate: u64,
        shutdown: oneshot::Receiver<()>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut shutdown = shutdown;
            let mut tick = 0u64;
            let tick_duration = Duration::from_secs_f64(1.0 / tick_rate as f64);

            // 创建游戏世界
            let mut game_world = GameWorld::new();

            loop {
                tokio::select! {
                    _ = &mut shutdown => {
                        info!("Game loop task shutting down");
                        break;
                    }

                    _ = tokio::time::sleep(tick_duration) => {
                        // 处理网络消息
                        while let Some(packet) = network_rx.recv().await {
                            match packet {
                                NetworkPacket::ClientConnected { client_id } => {
                                    // 处理新连接
                                    Self::handle_client_connected(
                                        client_id,
                                        &room_manager,
                                        &players,
                                        &database,
                                        &game_tx,
                                    ).await;
                                }
                                NetworkPacket::ClientDisconnected { client_id } => {
                                    // 处理断开连接
                                    Self::handle_client_disconnected(
                                        client_id,
                                        &room_manager,
                                        &players,
                                        &game_tx,
                                    ).await;
                                }
                                NetworkPacket::Message { client_id, message } => {
                                    // 处理客户端消息
                                    Self::handle_client_message(
                                        client_id,
                                        message,
                                        &room_manager,
                                        &players,
                                        &mut game_world,
                                        &database,
                                        &game_tx,
                                    ).await;
                                }
                            }
                        }

                        // 更新游戏世界
                        game_world.update(tick);

                        // 广播状态
                        let world_update = game_world.create_update_packet(tick);
                        game_tx.send(world_update).ok();

                        tick += 1;
                    }
                }
            }
        })
    }

    /// 处理客户端连接
    async fn handle_client_connected(
        client_id: u64,
        room_manager: &Arc<RwLock<RoomManager>>,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        database: &Arc<DatabaseManager>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
    ) {
        info!("Handling client connection: {}", client_id);

        // 发送连接确认
        let server_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let connect_result = ConnectResultMessage {
            success: true,
            player_id: None,
            entity_id: None,
            spawn_position: None,
            error: None,
            server_timestamp,
        };
        let response = ServerMessage::ConnectResult(connect_result);

        game_tx.send(response).ok();

        // 创建临时会话
        let session = PlayerSession {
            client_id,
            player_id: None,
            room_id: None,
            authenticated: false,
        };

        // TODO: 存储会话，等待登录
    }

    /// 处理客户端断开
    async fn handle_client_disconnected(
        client_id: u64,
        room_manager: &Arc<RwLock<RoomManager>>,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
    ) {
        info!("Handling client disconnect: {}", client_id);

        // 查找并移除玩家
        let player_id = {
            let mut players_guard = players.write().await;
            let mut found_id = None;

            for (id, player) in players_guard.iter() {
                if player.session.client_id == client_id {
                    found_id = Some(*id);
                    break;
                }
            }

            if let Some(id) = found_id {
                players_guard.remove(&id);
                Some(id)
            } else {
                None
            }
        };

        if let Some(player_id) = player_id {
            // 从房间移除
            let mut room_manager_guard = room_manager.write().await;
            room_manager_guard.remove_player(&player_id);

            // 广播离开消息 - 通过 WorldUpdate
            let mut update = WorldUpdateMessage::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                0,
            );
            update.remove_entity(EntityId::new(player_id.raw()));
            game_tx.send(ServerMessage::WorldUpdate(update)).ok();
        }
    }

    /// 处理客户端消息
    async fn handle_client_message(
        client_id: u64,
        message: ClientMessage,
        room_manager: &Arc<RwLock<RoomManager>>,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_world: &mut GameWorld,
        database: &Arc<DatabaseManager>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
    ) {
        match message {
            ClientMessage::Connect(connect_msg) => {
                // 处理连接请求
                Self::handle_connect_request_v2(client_id, connect_msg, players, database, game_tx).await;
            }

            ClientMessage::PlayerInput(input_msg) => {
                // 处理玩家输入
                Self::handle_player_input_v2(client_id, input_msg, players, game_world).await;
            }

            ClientMessage::Ping(ping_msg) => {
                // 响应 Pong
                let pong_msg = PongMessage {
                    ping_sequence: ping_msg.sequence,
                    ping_timestamp: ping_msg.timestamp,
                    pong_timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                game_tx.send(ServerMessage::Pong(pong_msg)).ok();
            }

            _ => {
                warn!("Unhandled message from client {}: {:?}", client_id, message);
            }
        }
    }

    /// 处理连接请求 (新版本)
    async fn handle_connect_request_v2(
        client_id: u64,
        connect_msg: ConnectMessage,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        database: &Arc<DatabaseManager>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
    ) {
        info!("Connect request from {}: {}", client_id, connect_msg.player_name);

        // TODO: 验证版本
        // TODO: 验证 Token (如果提供)
        // TODO: 创建玩家会话

        // 临时响应
        let result = ConnectResultMessage::success(
            PlayerId::new(client_id as u64),
            EntityId::new(client_id),
            [0.0, 0.0, 0.0],
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        game_tx.send(ServerMessage::ConnectResult(result)).ok();
    }

    /// 处理玩家输入 (新版本)
    async fn handle_player_input_v2(
        client_id: u64,
        input_msg: PlayerInputMessage,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_world: &mut GameWorld,
    ) {
        // 查找玩家
        let player_id = {
            let players_guard = players.read().await;
            let mut found_id = None;

            for (id, player) in players_guard.iter() {
                if player.session.client_id == client_id {
                    found_id = Some(*id);
                    break;
                }
            }

            found_id
        };

        if let Some(player_id) = player_id {
            // 更新输入
            game_world.set_player_input(player_id, input_msg.input);
        }
    }

    /// 创建连接配置
    fn create_connection_config() -> ConnectionConfig {
        ConnectionConfig {
            server_channels_config: DefaultChannel::config(),
            client_channels_config: DefaultChannel::config(),
            ..Default::default()
        }
    }
}
