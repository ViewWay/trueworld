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
};
use trueworld_core::*;

use crate::{
    config::ServerConfig,
    database::DatabaseManager,
    game::GameWorld,
    movement::MovementUpdateProcessor,
    network::NetworkEvent,
    player::Player,
    room::RoomManager,
    shutdown::ShutdownManager,
};

use trueworld_core::net::ServerPositionAck;
use crate::movement::ProcessInputResult;

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
        let (network_tx, network_rx) = mpsc::unbounded_channel();
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
        let _ = self.shutdown.wait().await;

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
        network_tx: mpsc::UnboundedSender<NetworkEvent>,
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
                                    network_tx.send(NetworkEvent::ClientConnected { client_id })
                                        .ok();
                                }
                                ServerEvent::ClientDisconnected { client_id, reason } => {
                                    info!("Client disconnected: {} - {:?}", client_id, reason);
                                    network_tx.send(NetworkEvent::ClientDisconnected { client_id, player_id: None })
                                        .ok();
                                }
                            }
                        }

                        // 接收客户端消息
                        for client_id in renet_server.clients_id() {
                            while let Some(message) = renet_server.receive_message(client_id, 0) {
                                if let Ok(packet) = bincode::deserialize::<ClientMessage>(&message) {
                                    network_tx.send(NetworkEvent::Message { client_id, message: packet })
                                        .ok();
                                }
                            }

                            while let Some(message) = renet_server.receive_message(client_id, 2) {
                                if let Ok(packet) = bincode::deserialize::<ClientMessage>(&message) {
                                    network_tx.send(NetworkEvent::Message { client_id, message: packet })
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
        mut network_rx: mpsc::UnboundedReceiver<NetworkEvent>,
        tick_rate: u64,
        shutdown: oneshot::Receiver<()>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut shutdown = shutdown;
            let _tick = 0u64;
            let tick_duration = Duration::from_secs_f64(1.0 / tick_rate as f64);

            // 创建游戏世界
            let mut game_world = GameWorld::new();

            // 创建移动验证处理器 (Phase 4)
            let mut movement_processor = MovementUpdateProcessor::with_defaults();

            // Ack发送计数器 (每20个tick发送一次位置确认，约20Hz)
            let mut ack_send_counter = 0u32;
            const ACK_SEND_INTERVAL: u32 = 3; // 60Hz / 3 = 20Hz

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
                                NetworkEvent::ClientConnected { client_id } => {
                                    // 处理新连接
                                    Self::handle_client_connected(
                                        client_id,
                                        &room_manager,
                                        &players,
                                        &database,
                                        &game_tx,
                                        &mut movement_processor,
                                    ).await;
                                }
                                NetworkEvent::ClientDisconnected { client_id, player_id: _ } => {
                                    // 处理断开连接
                                    Self::handle_client_disconnected(
                                        client_id,
                                        &room_manager,
                                        &players,
                                        &game_tx,
                                        &mut movement_processor,
                                    ).await;
                                }
                                NetworkEvent::Message { client_id, message } => {
                                    // 处理客户端消息
                                    Self::handle_client_message(
                                        client_id,
                                        message,
                                        &room_manager,
                                        &players,
                                        &mut game_world,
                                        &database,
                                        &game_tx,
                                        &mut movement_processor,
                                        tick_duration,
                                    ).await;
                                }
                            }
                        }

                        // 更新游戏世界
                        game_world.update(tick_duration);

                        // 更新移动处理器
                        movement_processor.update(tick_duration);

                        // 定期发送位置确认 (约20Hz)
                        ack_send_counter = ack_send_counter.wrapping_add(1);
                        if ack_send_counter >= ACK_SEND_INTERVAL {
                            ack_send_counter = 0;

                            // 获取所有玩家的位置确认
                            for (player_id, _position, _velocity) in movement_processor.create_position_updates() {
                                let state = movement_processor.get_player_state(player_id);
                                if let Some(state) = state {
                                    let ack = ServerPositionAck {
                                        player_id,
                                        ack_sequence: state.last_client_sequence,
                                        position: state.position,
                                        velocity: state.velocity,
                                        server_time: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs(),
                                    };
                                    game_tx.send(ServerMessage::PositionAck(ack)).ok();
                                }
                            }
                        }

                        // 发送位置修正 (立即发送，可靠通道)
                        for correction in movement_processor.take_corrections() {
                            game_tx.send(ServerMessage::PositionCorrection(correction)).ok();
                        }

                        // 广播状态
                        let world_update = game_world.create_update_packet();
                        game_tx.send(world_update).ok();
                    }
                }
            }
        })
    }

    /// 处理客户端连接
    async fn handle_client_connected(
        client_id: u64,
        _room_manager: &Arc<RwLock<RoomManager>>,
        _players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        _database: &Arc<DatabaseManager>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
        _movement_processor: &mut MovementUpdateProcessor,
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

        // TODO: 在玩家完全连接后添加到 movement_processor
        // 当前在 handle_connect_request_v2 中处理
    }

    /// 处理客户端断开
    async fn handle_client_disconnected(
        client_id: u64,
        room_manager: &Arc<RwLock<RoomManager>>,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
        movement_processor: &mut MovementUpdateProcessor,
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
            // 从移动处理器中移除 (Phase 4)
            movement_processor.remove_player(player_id);

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
        _room_manager: &Arc<RwLock<RoomManager>>,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_world: &mut GameWorld,
        _database: &Arc<DatabaseManager>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
        movement_processor: &mut MovementUpdateProcessor,
        tick_duration: Duration,
    ) {
        match message {
            ClientMessage::Connect(connect_msg) => {
                // 处理连接请求
                Self::handle_connect_request_v2(client_id, connect_msg, players, game_tx, movement_processor).await;
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

            ClientMessage::ClientInputPacket(packet_bytes) => {
                // 处理客户端移动输入包 (Phase 4)
                match bincode::deserialize::<trueworld_protocol::ClientInputPacket>(&packet_bytes) {
                    Ok(packet) => {
                        Self::handle_movement_input(
                            client_id,
                            packet,
                            players,
                            game_world,
                            game_tx,
                            movement_processor,
                            tick_duration,
                        ).await;
                    }
                    Err(e) => {
                        warn!("Failed to deserialize ClientInputPacket: {}", e);
                    }
                }
            }
        }
    }

    /// 处理连接请求 (新版本)
    async fn handle_connect_request_v2(
        client_id: u64,
        connect_msg: ConnectMessage,
        _players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_tx: &mpsc::UnboundedSender<ServerMessage>,
        movement_processor: &mut MovementUpdateProcessor,
    ) {
        info!("Connect request from {}: {}", client_id, connect_msg.player_name);

        // TODO: 验证版本
        // TODO: 验证 Token (如果提供)

        // 创建玩家 ID
        let player_id = PlayerId::new(client_id as u64);
        let entity_id = EntityId::new(client_id);
        let spawn_position = [0.0, 0.0, 0.0];

        // 添加到移动处理器 (Phase 4)
        movement_processor.add_player(player_id, spawn_position);

        // 发送成功响应
        let result = ConnectResultMessage::success(
            player_id,
            entity_id,
            spawn_position,
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

    /// 处理客户端移动输入包 (Phase 4 移动系统集成)
    async fn handle_movement_input(
        client_id: u64,
        packet: trueworld_protocol::ClientInputPacket,
        players: &Arc<RwLock<HashMap<PlayerId, Player>>>,
        game_world: &mut GameWorld,
        _game_tx: &mpsc::UnboundedSender<ServerMessage>,
        movement_processor: &mut MovementUpdateProcessor,
        tick_duration: Duration,
    ) {
        // 查找玩家 ID
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
            // 使用 MovementUpdateProcessor 处理输入 (Phase 4)
            let result = movement_processor.process_client_input(
                player_id,
                &packet,
                tick_duration,
            );

            match result {
                ProcessInputResult::Success { new_position, velocity } => {
                    // 同时更新 GameWorld 以保持兼容性
                    let player_input = PlayerInput {
                        sequence: packet.sequence,
                        movement: packet.movement,
                        actions: packet.actions,
                        view_direction: [0.0, 0.0, 0.0],
                        timestamp: packet.timestamp,
                    };
                    game_world.set_player_input(player_id, player_input);

                    info!("Player {} moved to {:?}, velocity {:?}", player_id, new_position, velocity);
                }
                ProcessInputResult::IgnoredOldInput => {
                    info!("Ignored old input from player {}", player_id);
                }
                ProcessInputResult::Violation { reason, .. } => {
                    warn!("Movement violation for player {}: {:?}", player_id, reason);
                }
                ProcessInputResult::KickRequired { player_id, reason } => {
                    warn!("Kicking player {} for: {}", player_id, reason);
                    // TODO: 实现踢出逻辑
                }
                ProcessInputResult::PlayerNotFound => {
                    info!("Player {} not found in movement processor", player_id);
                }
            }
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
