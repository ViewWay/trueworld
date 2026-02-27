// crates/client/src/network.rs

use std::{
    net::UdpSocket,
    time::{Duration, Instant},
};

use bevy::{
    prelude::*,
    time::Time,
};
use renet::{
    ConnectionConfig, DefaultChannel, RenetClient,
};
use renet_netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
    ClientAuthentication as NetcodeClientAuth, ServerConfig as NetcodeServerConfig,
};

use trueworld_core::*;
use trueworld_core::net::*;

use crate::state::{ConnectionState, NetworkStats};

/// 网络插件
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PreUpdate,
                (
                    handle_connection,
                    receive_packets,
                    handle_packets,
                )
                    .chain()
                    .run_if(|state: Res<State<ConnectionState>>| {
                        **state == ConnectionState::Connected || **state == ConnectionState::Connecting
                    }),
            )
            .add_systems(
                PostUpdate,
                (
                    send_packets,
                    update_network_stats,
                )
                    .chain()
                    .run_if(|state: Res<State<ConnectionState>>| **state == ConnectionState::Connected),
            )
            .init_resource::<NetworkQueue>()
            .init_resource::<NetworkStats>();
    }
}

/// 网络资源
#[derive(Resource)]
pub struct NetworkResource {
    pub client: Option<RenetClient>,
    pub transport: Option<NetcodeClientTransport>,
    pub server_addr: String,
    pub server_port: u16,
    pub current_connection_id: Option<u64>,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            client: None,
            transport: None,
            server_addr: "127.0.0.1".to_string(),
            server_port: 5000,
            current_connection_id: None,
        }
    }
}

/// 网络队列
#[derive(Resource, Default)]
pub struct NetworkQueue {
    pub outgoing: Vec<ServerMessage>,
    pub incoming: Vec<ClientMessage>,
}

/// 连接请求事件
#[derive(Event)]
pub struct ConnectionRequest {
    pub address: String,
    pub port: u16,
}

/// 连接结果事件
#[derive(Event)]
pub struct ConnectionResult {
    pub success: bool,
    pub player_id: Option<PlayerId>,
    pub reason: Option<String>,
}

/// 断开连接事件
#[derive(Event)]
pub struct DisconnectEvent {
    pub reason: String,
}

/// 玩家加入事件
#[derive(Event)]
pub struct PlayerJoinedEvent {
    pub player_id: PlayerId,
    pub name: String,
}

/// 玩家离开事件
#[derive(Event)]
pub struct PlayerLeftEvent {
    pub player_id: PlayerId,
}

/// 连接到服务器
pub fn connect_to_server(
    world: &mut World,
    address: String,
    port: u16,
) -> anyhow::Result<()> {
    let mut network = world.get_resource_mut::<NetworkResource>()
        .ok_or_else(|| anyhow::anyhow!("NetworkResource not found"))?;

    // 创建 Renet 客户端
    let connection_config = create_connection_config();
    let client = RenetClient::new(connection_config);

    // 创建传输层
    let server_addr = format!("{}:{}", address, port);
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    let current_time = Instant::now();
    let client_id = 0; // 将由服务器分配
    let authentication = ClientAuthentication::Unsecure;

    let transport = NetcodeClientTransport::new(
        current_time,
        socket,
        client_id,
        authentication,
        &server_addr,
    )?;

    network.client = Some(client);
    network.transport = Some(transport);
    network.server_addr = address;
    network.server_port = port;

    world.insert_resource(ConnectionState::Connecting);

    Ok(())
}

/// 创建连接配置
fn create_connection_config() -> ConnectionConfig {
    ConnectionConfig {
        server_channels_config: DefaultChannel::config(),
        client_channels_config: DefaultChannel::config(),
        ..Default::default()
    }
}

/// 处理连接状态
fn handle_connection(
    mut commands: Commands,
    mut network_res: ResMut<NetworkResource>,
    mut next_state: ResMut<NextState<ConnectionState>>,
    mut connection_events: EventWriter<ConnectionResult>,
    time: Res<Time<Real>>,
) {
    let Some(ref transport) = network_res.transport else {
        return;
    };

    let Some(ref mut client) = network_res.client else {
        return;
    };

    match transport.update(time.delta(), client) {
        Ok(disconnected) => {
            if disconnected {
                warn!("Disconnected from server");
                next_state.set(ConnectionState::Disconnected);
                connection_events.send(ConnectionResult {
                    success: false,
                    player_id: None,
                    reason: Some("Disconnected".to_string()),
                });
            }
        }
        Err(NetcodeTransportError::Disconnect) => {
            warn!("Server disconnected");
            next_state.set(ConnectionState::Disconnected);
            connection_events.send(ConnectionResult {
                success: false,
                player_id: None,
                reason: Some("Server disconnected".to_string()),
            });
        }
        Err(err) => {
            error!("Network error: {}", err);
        }
    }

    // 检查是否已连接
    if transport.is_connected() {
        if **next_state == ConnectionState::Connecting {
            next_state.set(ConnectionState::Connected);
            info!("Connected to server");
            connection_events.send(ConnectionResult {
                success: true,
                player_id: None, // 将从登录响应获取
                reason: None,
            });
        }
    }
}

/// 接收数据包
fn receive_packets(
    mut network_res: ResMut<NetworkResource>,
    mut queue: ResMut<NetworkQueue>,
) {
    let Some(ref mut client) = network_res.client else {
        return;
    };

    // 可靠有序通道 (channel_id = 0)
    while let Some(message) = client.receive_message(0) {
        match bincode::deserialize::<ServerMessage>(&message) {
            Ok(msg) => {
                handle_server_message(msg, &mut queue);
            }
            Err(e) => {
                error!("Failed to deserialize packet: {}", e);
            }
        }
    }

    // 不可靠通道 (channel_id = 2)
    while let Some(message) = client.receive_message(2) {
        match bincode::deserialize::<ServerMessage>(&message) {
            Ok(msg) => {
                handle_server_message(msg, &mut queue);
            }
            Err(e) => {
                error!("Failed to deserialize packet: {}", e);
            }
        }
    }
}

/// 处理服务器消息
fn handle_server_message(msg: ServerMessage, queue: &mut NetworkQueue) {
    match msg {
        ServerMessage::ConnectResult(result) => {
            info!("Connect result: success={}", result.success);
        }
        ServerMessage::WorldUpdate(update) => {
            info!("World update: {} entities", update.entities.len());
        }
        ServerMessage::Pong(pong) => {
            info!("Received pong, RTT: {}ms", pong.rtt());
        }
    }
}

/// 处理接收到的数据包
fn handle_packets(
    mut queue: ResMut<NetworkQueue>,
    mut commands: Commands,
    mut player_joined: EventWriter<PlayerJoinedEvent>,
    mut player_left: EventWriter<PlayerLeftEvent>,
    mut connection_events: EventWriter<ConnectionResult>,
    mut disconnect_events: EventWriter<DisconnectEvent>,
) {
    // Server messages are now handled in receive_packets
    // This function can be used for higher-level game logic
}

/// 发送数据包
fn send_packets(
    mut network_res: ResMut<NetworkResource>,
    mut queue: ResMut<NetworkQueue>,
) {
    let Some(ref mut client) = network_res.client else {
        return;
    };

    for msg in queue.outgoing.drain(..) {
        let serialized = match bincode::serialize(&msg) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialize packet: {}", e);
                continue;
            }
        };

        // 根据消息类型选择通道
        let channel_id = match msg {
            ServerMessage::WorldUpdate(_) => 2, // 不可靠通道
            _ => 0, // 可靠通道
        };

        client.send_message(channel_id, serialized);
    }
}

/// 更新网络统计
fn update_network_stats(
    mut stats: ResMut<NetworkStats>,
    network_res: Res<NetworkResource>,
    time: Res<Time<Real>>,
) {
    let Some(ref transport) = network_res.transport else {
        return;
    };

    // 注意：renet_netcode 1.2 的 API 可能不同
    // 这里使用占位符实现
    stats.last_update = time.elapsed_seconds();
}
