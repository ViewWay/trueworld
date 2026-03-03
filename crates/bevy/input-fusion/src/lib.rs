// crates/bevy/input-fusion/src/lib.rs

use bevy::{
    app::{Plugin, Update},
    ecs::schedule::SystemSet,
    prelude::*,
};
use std::time::Instant;

pub mod fusion;
pub mod sources;

pub use fusion::*;

/// Input Fusion Plugin
///
/// Fuses multiple input sources (keyboard/mouse, camera, voice) into
/// unified game actions.
pub struct InputFusionPlugin;

impl Plugin for InputFusionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Events
            .add_event::<AiInputEvent>()
            .add_event::<FusedInputEvent>()
            .add_event::<ActionTriggeredEvent>()

            // Resources
            .init_resource::<InputBuffer>()
            .init_resource::<SkillTriggerConfig>()
            .init_resource::<FusionStats>()

            // Systems
            .configure_sets(
                Update,
                InputFusionSet
                    .chain()
                    .in_set(InputSystemSet),
            )
            .add_systems(
                Update,
                (
                    receive_ai_inputs,
                    process_input_buffer,
                    fuse_inputs,
                    emit_fused_events,
                    update_fusion_stats,
                )
                    .chain()
                    .in_set(InputFusionSet),
            );
    }
}

/// System set for input fusion
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputFusionSet;

/// System set for all input systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSystemSet;

/// AI 输入事件
#[derive(Event, Clone, Debug)]
pub enum AiInputEvent {
    /// 摄像头/动作识别输入
    Action {
        action: ActionData,
        confidence: f32,
        timestamp: Instant,
    },

    /// 语音输入
    Voice {
        text: String,
        intent: Intent,
        confidence: f32,
        timestamp: Instant,
    },

    /// 手势输入
    Gesture {
        gesture: Gesture,
        confidence: f32,
        position: (f32, f32),
        timestamp: Instant,
    },
}

/// 融合后的输入事件
#[derive(Event, Clone, Debug)]
pub struct FusedInputEvent {
    /// 触发的动作
    pub action: PlayerAction,

    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,

    /// 输入来源
    pub sources: Vec<InputSource>,

    /// 时间戳
    pub timestamp: Instant,
}

/// 动作触发事件
#[derive(Event, Clone, Debug)]
pub struct ActionTriggeredEvent {
    /// 技能 ID
    pub skill_id: String,

    /// 动作参数
    pub parameters: ActionParameters,

    /// 触发来源
    pub sources: Vec<InputSource>,
}

/// 玩家动作
#[derive(Debug, Clone)]
pub enum PlayerAction {
    /// 释放技能
    ActivateSkill {
        skill_id: String,
        parameters: ActionParameters,
    },

    /// 移动
    Move {
        direction: Vec3,
        speed: f32,
    },

    /// 社交交互
    Social {
        action: SocialAction,
        target: Option<String>,
    },

    /// 系统命令
    System {
        command: SystemCommand,
    },
}

/// 动作参数
#[derive(Debug, Clone)]
pub struct ActionParameters {
    /// 方向
    pub direction: Vec3,

    /// 力度/强度
    pub power: f32,

    /// 目标 (如果有)
    pub target: Option<Entity>,
}

/// 社交动作
#[derive(Debug, Clone)]
pub enum SocialAction {
    Invite,
    Accept,
    Reject,
    Trade,
    Follow,
    Thanks,
    Sorry,
}

/// 系统命令
#[derive(Debug, Clone)]
pub enum SystemCommand {
    SwitchCamera,
    OpenMenu,
    Pause,
    Quit,
}

/// 输入来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    Keyboard,
    Mouse,
    Gamepad,
    CameraGesture,
    Voice,
    Action,
}

/// 动作数据
#[derive(Debug, Clone)]
pub struct ActionData {
    /// 动作类型
    pub action_type: ActionType,

    /// 位置
    pub position: (f32, f32),

    /// 速度
    pub velocity: (f32, f32),

    /// 轨迹
    pub trajectory: Vec<(f32, f32)>,
}

/// 动作类型
#[derive(Debug, Clone)]
pub enum ActionType {
    /// 挥动 (水平/垂直/斜向)
    Swing {
        direction: SwingDirection,
        speed: f32,
    },

    /// 刺击
    Thrust {
        speed: f32,
    },

    /// 格挡
    Parry,

    /// 闪避
    Dodge {
        direction: Vec3,
    },

    /// 未知
    Unknown,
}

/// 挥动方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwingDirection {
    Horizontal,
    Vertical,
    DiagonalDown,
    DiagonalUp,
}

/// 手势
#[derive(Debug, Clone)]
pub enum Gesture {
    Open,
    Closed,
    Pointing,
    Victory,
    ThumbUp,
}

/// 意图 (来自 NLP)
#[derive(Debug, Clone)]
pub struct Intent {
    /// 意图类型
    pub intent_type: IntentType,

    /// 实体
    pub entities: Vec<String>,

    /// 置信度
    pub confidence: f32,
}

/// 意图类型
#[derive(Debug, Clone)]
pub enum IntentType {
    /// 技能
    Skill { skill_name: String },

    /// 移动
    Move { direction: String },

    /// 社交
    Social { action: String },

    /// 系统
    System { command: String },

    /// 聊天
    Chat,
}

/// 融合统计
#[derive(Resource, Default)]
pub struct FusionStats {
    pub total_inputs: u64,
    pub fused_inputs: u64,
    pub confidence_sum: f32,
    pub source_counts: std::collections::HashMap<InputSource, u64>,
}

impl FusionStats {
    pub fn average_confidence(&self) -> f32 {
        if self.fused_inputs == 0 {
            return 0.0;
        }
        self.confidence_sum / self.fused_inputs as f32
    }
}
