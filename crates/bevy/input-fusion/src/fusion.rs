// crates/bevy/input-fusion/src/fusion.rs

use bevy::prelude::*;
use std::{collections::{HashMap, VecDeque}, time::Instant};
use crate::*;
use crate::{AiInputEvent, FusedInputEvent, ActionTriggeredEvent, PlayerAction, InputSource};

/// 输入缓冲区
#[derive(Resource, Default)]
pub struct InputBuffer {
    /// 动作输入
    pub actions: VecDeque<(ActionData, Instant)>,

    /// 语音输入
    pub voice: VecDeque<(VoiceInput, Instant)>,

    /// 手势输入
    pub gestures: VecDeque<(Gesture, f32, Instant)>,

    /// 缓冲时长
    pub buffer_duration: std::time::Duration,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            buffer_duration: std::time::Duration::from_millis(500),
            ..Default::default()
        }
    }

    /// 清理过期数据
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let cutoff = now - self.buffer_duration;

        self.actions.retain(|(_, t)| *t > cutoff);
        self.voice.retain(|(_, t)| *t > cutoff);
        self.gestures.retain(|(_, _, t)| *t > cutoff);
    }
}

#[derive(Debug, Clone)]
pub struct VoiceInput {
    pub text: String,
    pub intent: Intent,
    pub confidence: f32,
}

/// 技能触发配置
#[derive(Resource, Clone)]
pub struct SkillTriggerConfig {
    /// 技能触发配置
    pub skills: HashMap<String, SkillTrigger>,

    /// 灵活模式配置
    pub flexible_mode: bool,
}

#[derive(Clone)]
pub struct SkillTrigger {
    pub id: String,
    pub display_name: String,
    pub aliases: Vec<String>,

    /// 语音关键词
    pub voice_keywords: Vec<String>,

    /// 动作模式
    pub motion_pattern: Option<MotionPattern>,

    /// 键盘快捷键
    pub keybind: Option<KeyCode>,

    /// 所需的最少输入源数量
    pub min_sources: usize,

    /// 所需的最小置信度
    pub min_confidence: f32,
}

#[derive(Clone)]
pub struct MotionPattern {
    /// 动作类型
    pub action_type: ActionType,

    /// 速度范围
    pub speed_range: (f32, f32),

    /// 方向范围 (度)
    pub direction_range: (f32, f32),

    /// 容差
    pub tolerance: f32,
}

impl Default for SkillTriggerConfig {
    fn default() -> Self {
        let mut skills = HashMap::new();

        // 水平斩
        skills.insert("horizontal_slash".to_string(), SkillTrigger {
            id: "horizontal_slash".to_string(),
            display_name: "水平斩".to_string(),
            aliases: vec!["横扫".to_string(), "横斩".to_string()],
            voice_keywords: vec!["水平斩".to_string(), "横扫".to_string()],
            motion_pattern: Some(MotionPattern {
                action_type: ActionType::Swing {
                    direction: SwingDirection::Horizontal,
                    speed: 1.0,
                },
                speed_range: (0.5, 2.0),
                direction_range: (-30.0, 30.0),
                tolerance: 45.0,
            }),
            keybind: Some(KeyCode::Digit1),
            min_sources: 1,
            min_confidence: 0.6,
        });

        // 垂直斩
        skills.insert("vertical_slash".to_string(), SkillTrigger {
            id: "vertical_slash".to_string(),
            display_name: "垂直斩".to_string(),
            aliases: vec!["竖斩".to_string(), "劈砍".to_string()],
            voice_keywords: vec!["垂直斩".to_string(), "竖斩".to_string()],
            motion_pattern: Some(MotionPattern {
                action_type: ActionType::Swing {
                    direction: SwingDirection::Vertical,
                    speed: 1.0,
                },
                speed_range: (0.5, 2.0),
                direction_range: (60.0, 120.0),
                tolerance: 30.0,
            }),
            keybind: Some(KeyCode::Digit2),
            min_sources: 1,
            min_confidence: 0.6,
        });

        // 音速冲击 (刺击)
        skills.insert("sonic_impact".to_string(), SkillTrigger {
            id: "sonic_impact".to_string(),
            display_name: "音速冲击".to_string(),
            aliases: vec!["突刺".to_string()],
            voice_keywords: vec!["音速冲击".to_string(), "突刺".to_string()],
            motion_pattern: Some(MotionPattern {
                action_type: ActionType::Thrust { speed: 2.0 },
                speed_range: (1.5, 3.0),
                direction_range: (-15.0, 15.0),
                tolerance: 20.0,
            }),
            keybind: Some(KeyCode::Digit3),
            min_sources: 1,
            min_confidence: 0.5,
        });

        Self {
            skills,
            flexible_mode: true,
        }
    }
}

/// 接收 AI 输入事件
pub fn receive_ai_inputs(
    mut events: EventReader<AiInputEvent>,
    mut buffer: ResMut<InputBuffer>,
) {
    for event in events.read() {
        match event {
            AiInputEvent::Action { action, confidence: _, timestamp } => {
                buffer.actions.push_back((action.clone(), *timestamp));
            }
            AiInputEvent::Voice { text, intent, confidence, timestamp } => {
                buffer.voice.push_back((
                    VoiceInput {
                        text: text.clone(),
                        intent: intent.clone(),
                        confidence: *confidence,
                    },
                    *timestamp,
                ));
            }
            AiInputEvent::Gesture { gesture, confidence, position: _, timestamp } => {
                buffer.gestures.push_back((
                    gesture.clone(),
                    *confidence,
                    *timestamp,
                ));
            }
        }
    }
}

/// 处理输入缓冲
pub fn process_input_buffer(
    mut buffer: ResMut<InputBuffer>,
) {
    buffer.cleanup();
}

/// 融合多源输入
pub fn fuse_inputs(
    _commands: Commands,
    mut buffer: ResMut<InputBuffer>,
    config: ResMut<SkillTriggerConfig>,
    mut fused_events: EventWriter<FusedInputEvent>,
    mut action_events: EventWriter<ActionTriggeredEvent>,
    mut stats: ResMut<FusionStats>,
) {
    stats.total_inputs += 1;

    // 遍历所有技能配置
    for (skill_id, trigger) in &config.skills {
        let mut matched_sources = Vec::new();
        let mut confidence = 0.0;

        // 检查语音匹配
        for (voice, _) in buffer.voice.iter() {
            if let Some(match_conf) = check_voice_match(&trigger.voice_keywords, &voice.text, &voice.intent) {
                matched_sources.push(InputSource::Voice);
                confidence += match_conf * 0.4;
            }
        }

        // 检查动作匹配
        if let Some(pattern) = &trigger.motion_pattern {
            for (action, _) in buffer.actions.iter() {
                if let Some(match_conf) = check_action_match(pattern, &action.action_type) {
                    if match_conf > 0.5 {
                        matched_sources.push(InputSource::Action);
                        confidence += match_conf * 0.4;
                    }
                }
            }
        }

        // 检查手势匹配
        for (gesture, conf, _) in buffer.gestures.iter() {
            if check_gesture_match(&trigger.id, gesture) {
                matched_sources.push(InputSource::CameraGesture);
                confidence += conf * 0.2;
            }
        }

        // 验证是否满足触发条件
        if matched_sources.len() >= trigger.min_sources && confidence >= trigger.min_confidence {
            // 多源加成
            if matched_sources.len() > 1 {
                confidence *= 1.2;
            }
            confidence = confidence.min(1.0);

            // 计算动作参数
            let parameters = compute_parameters(&buffer, &trigger);

            // 发送融合事件
            fused_events.send(FusedInputEvent {
                action: PlayerAction::ActivateSkill {
                    skill_id: skill_id.clone(),
                    parameters: parameters.clone(),
                },
                confidence,
                sources: matched_sources.clone(),
                timestamp: Instant::now(),
            });

            // 发送动作触发事件
            action_events.send(ActionTriggeredEvent {
                skill_id: skill_id.clone(),
                parameters,
                sources: matched_sources.clone(),
            });

            // 更新统计
            stats.fused_inputs += 1;
            stats.confidence_sum += confidence;
            for source in &matched_sources {
                *stats.source_counts.entry(*source).or_insert(0) += 1;
            }

            // 清空缓冲 (避免重复触发)
            buffer.actions.clear();
            buffer.voice.clear();
        }
    }
}

/// 检查语音匹配
fn check_voice_match(keywords: &[String], text: &str, intent: &Intent) -> Option<f32> {
    // 精确匹配
    for keyword in keywords {
        if text.contains(keyword) {
            return Some(1.0);
        }
    }

    // 意图匹配
    if let IntentType::Skill { skill_name } = &intent.intent_type {
        for keyword in keywords {
            if skill_name == keyword || skill_name.contains(keyword) || keyword.contains(skill_name) {
                return Some(intent.confidence);
            }
        }
    }

    None
}

/// 检查动作匹配
fn check_action_match(pattern: &MotionPattern, action: &ActionType) -> Option<f32> {
    match (&pattern.action_type, action) {
        (ActionType::Swing { direction: d1, .. }, ActionType::Swing { direction: d2, speed }) => {
            if d1 == d2 {
                // 检查速度范围
                if *speed >= pattern.speed_range.0 && *speed <= pattern.speed_range.1 {
                    return Some(0.8);
                }
            }
        }
        (ActionType::Thrust { .. }, ActionType::Thrust { .. }) => {
            return Some(0.7);
        }
        _ => {}
    }

    None
}

/// 检查手势匹配
fn check_gesture_match(skill_id: &str, gesture: &Gesture) -> bool {
    match skill_id {
        "horizontal_slash" | "vertical_slash" => {
            matches!(gesture, Gesture::Closed)
        }
        "sonic_impact" => {
            matches!(gesture, Gesture::Pointing)
        }
        _ => false,
    }
}

/// 计算动作参数
fn compute_parameters(buffer: &InputBuffer, _trigger: &SkillTrigger) -> ActionParameters {
    let mut direction = Vec3::ZERO;
    let mut power = 1.0;

    // 从动作数据获取方向
    if let Some((action, _)) = buffer.actions.front() {
        match &action.action_type {
            ActionType::Swing { direction: swing_dir, .. } => {
                match swing_dir {
                    SwingDirection::Horizontal => direction = Vec3::X,
                    SwingDirection::Vertical => direction = Vec3::Y,
                    SwingDirection::DiagonalDown => direction = Vec3::new(1.0, -1.0, 0.0).normalize(),
                    SwingDirection::DiagonalUp => direction = Vec3::new(1.0, 1.0, 0.0).normalize(),
                }
            }
            ActionType::Thrust { speed } => {
                direction = Vec3::Z;
                power = *speed;
            }
            _ => {}
        }

        // 使用动作速度
        if let ActionType::Swing { speed, .. } = action.action_type {
            power = speed;
        }
    }

    ActionParameters {
        direction,
        power,
        target: None,
    }
}

/// 发送融合事件
pub fn emit_fused_events(
    mut fused_events: EventReader<FusedInputEvent>,
    _commands: Commands,
) {
    for event in fused_events.read() {
        match &event.action {
            PlayerAction::ActivateSkill { skill_id, parameters } => {
                info!("Skill activated: {} with power {}", skill_id, parameters.power);
            }
            PlayerAction::Move { direction, speed } => {
                info!("Move: {:?} at speed {}", direction, speed);
            }
            PlayerAction::Social { action, target } => {
                info!("Social: {:?} targeting {:?}", action, target);
            }
            PlayerAction::System { command } => {
                info!("System command: {:?}", command);
            }
        }
    }
}

/// 更新融合统计
pub fn update_fusion_stats(
    stats: Res<FusionStats>,
) {
    if stats.total_inputs % 100 == 0 {
        info!(
            "Fusion stats: {}/{} inputs fused, avg confidence: {:.2}",
            stats.fused_inputs,
            stats.total_inputs,
            stats.average_confidence()
        );
    }
}
