// crates/client/src/state.rs

use bevy::prelude::*;
use trueworld_core::*;

/// 游戏状态 (Bevy 0.15 使用 States trait)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum GameState {
    /// 初始状态
    #[default]
    Init,

    /// 连接服务器
    Connecting,

    /// 登录
    Login,

    /// 角色选择
    CharacterSelect,

    /// 加载世界
    Loading,

    /// 游戏中
    Playing,

    /// 暂停
    Paused,

    /// 对话中
    Dialog,

    /// 菜单
    Menu,

    /// 断开连接
    Disconnect,

    /// 错误
    Error,
}

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// 游戏资源
#[derive(Resource)]
pub struct GameResource {
    pub player_id: Option<PlayerId>,
    pub server_address: String,
    pub server_port: u16,
    pub current_map: String,
}

impl Default for GameResource {
    fn default() -> Self {
        Self {
            player_id: None,
            server_address: "127.0.0.1".to_string(),
            server_port: 5000,
            current_map: "spawn".to_string(),
        }
    }
}

/// 网络统计
#[derive(Resource, Default)]
pub struct NetworkStats {
    pub ping_ms: f32,
    pub packet_loss: f32,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_update: f64,
}

impl NetworkStats {
    pub fn update_ping(&mut self, ping: f32) {
        // 指数移动平均
        self.ping_ms = self.ping_ms * 0.9 + ping * 0.1;
    }

    pub fn packet_loss_rate(&self) -> f32 {
        let total = self.packets_sent + self.packets_received;
        if total == 0 {
            return 0.0;
        }
        self.packets_lost() as f32 / total as f32
    }

    pub fn packets_lost(&self) -> u64 {
        self.packets_sent.saturating_sub(self.packets_received)
    }
}

/// 客户端配置
#[derive(Resource, Clone)]
pub struct ClientConfig {
    /// 网络配置
    pub network: NetworkClientConfig,

    /// 图形配置
    pub graphics: GraphicsConfig,

    /// 音频配置
    pub audio: AudioConfig,

    /// 输入配置
    pub input: InputConfig,

    /// AI 配置
    pub ai: AiClientConfig,
}

#[derive(Clone)]
pub struct NetworkClientConfig {
    pub server_address: String,
    pub server_port: u16,
    pub tick_rate: u64,
    pub client_update_rate: f32,
    pub timeout_secs: u64,
}

impl Default for NetworkClientConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1".to_string(),
            server_port: 5000,
            tick_rate: 60,
            client_update_rate: 30.0,
            timeout_secs: 10,
        }
    }
}

#[derive(Clone)]
pub struct GraphicsConfig {
    pub render_distance: f32,
    pub shadow_quality: ShadowQuality,
    pub msaa: Msaa,
    pub vsync: bool,
    pub target_fps: u32,
    pub bloom: bool,
    pub fog: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            render_distance: 100.0,
            shadow_quality: ShadowQuality::Medium,
            msaa: Msaa::Off,
            vsync: true,
            target_fps: 60,
            bloom: true,
            fog: true,
        }
    }
}

#[derive(Clone)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub enable_voice_chat: bool,
    pub enable_voice_input: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            voice_volume: 0.9,
            enable_voice_chat: true,
            enable_voice_input: true,
        }
    }
}

#[derive(Clone)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub gamepad_sensitivity: f32,
    pub invert_y: bool,
    pub enable_vibration: bool,
    pub keybinds: Keybinds,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.5,
            gamepad_sensitivity: 0.5,
            invert_y: false,
            enable_vibration: true,
            keybinds: Keybinds::default(),
        }
    }
}

#[derive(Clone)]
pub struct Keybinds {
    pub move_forward: Vec<KeyCode>,
    pub move_backward: Vec<KeyCode>,
    pub move_left: Vec<KeyCode>,
    pub move_right: Vec<KeyCode>,
    pub jump: Vec<KeyCode>,
    pub crouch: Vec<KeyCode>,
    pub sprint: Vec<KeyCode>,
    pub attack_key: Vec<KeyCode>,
    pub block_key: Vec<KeyCode>,
    pub attack_mouse: Vec<MouseButton>,
    pub block_mouse: Vec<MouseButton>,
    pub dodge: Vec<KeyCode>,
    pub interact: Vec<KeyCode>,
    pub inventory: Vec<KeyCode>,
    pub skills: Vec<Vec<KeyCode>>,
    pub items: Vec<Vec<KeyCode>>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            move_forward: vec![KeyCode::KeyW, KeyCode::ArrowUp],
            move_backward: vec![KeyCode::KeyS, KeyCode::ArrowDown],
            move_left: vec![KeyCode::KeyA, KeyCode::ArrowLeft],
            move_right: vec![KeyCode::KeyD, KeyCode::ArrowRight],
            jump: vec![KeyCode::Space],
            crouch: vec![KeyCode::ControlLeft, KeyCode::KeyC],
            sprint: vec![KeyCode::ShiftLeft],
            attack_key: vec![],
            block_key: vec![],
            attack_mouse: vec![MouseButton::Left],
            block_mouse: vec![MouseButton::Right],
            dodge: vec![KeyCode::AltLeft],
            interact: vec![KeyCode::KeyE],
            inventory: vec![KeyCode::Tab, KeyCode::KeyI],
            skills: vec![
                vec![KeyCode::Digit1],
                vec![KeyCode::Digit2],
                vec![KeyCode::Digit3],
                vec![KeyCode::Digit4],
                vec![KeyCode::Digit5],
                vec![KeyCode::Digit6],
                vec![KeyCode::Digit7],
                vec![KeyCode::Digit8],
                vec![KeyCode::Digit9],
                vec![KeyCode::Digit0],
            ],
            items: vec![
                vec![KeyCode::KeyQ],
                vec![KeyCode::KeyR],
            ],
        }
    }
}

#[derive(Clone)]
pub struct AiClientConfig {
    /// 是否启用摄像头输入
    pub enable_camera: bool,

    /// 摄像头索引
    pub camera_index: usize,

    /// 是否启用语音输入
    pub enable_voice: bool,

    /// 语音输入语言
    pub voice_language: String,

    /// 是否启用动作识别
    pub enable_action_recognition: bool,

    /// 感知配置
    pub perception: PerceptionClientConfig,
}

#[derive(Clone)]
pub struct PerceptionClientConfig {
    pub input_resolution: (u32, u32),
    pub inference_resolution: (u32, u32),
    pub target_fps: u32,
}

impl Default for AiClientConfig {
    fn default() -> Self {
        Self {
            enable_camera: true,
            camera_index: 0,
            enable_voice: true,
            voice_language: "zh".to_string(),
            enable_action_recognition: true,
            perception: PerceptionClientConfig {
                input_resolution: (640, 480),
                inference_resolution: (320, 240),
                target_fps: 30,
            },
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            network: NetworkClientConfig::default(),
            graphics: GraphicsConfig::default(),
            audio: AudioConfig::default(),
            input: InputConfig::default(),
            ai: AiClientConfig::default(),
        }
    }
}
