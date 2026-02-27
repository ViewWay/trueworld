// crates/client/src/input.rs

use std::time::Instant;

use bevy::{
    input::{
        gamepad::{Gamepad, GamepadButton, GamepadButtonChangedEvent},
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion},
        ButtonInput,
    },
    prelude::*,
};

use trueworld_core::*;

use crate::state::{ClientConfig, InputConfig};

/// 输入插件
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputState>()
            .add_systems(
                Update,
                (
                    collect_keyboard_input,
                    collect_mouse_input,
                    collect_gamepad_input,
                    update_input_state,
                )
                    .chain(),
            );
    }
}

/// 输入状态
#[derive(Resource, Default)]
pub struct InputState {
    /// 当前帧的输入
    pub current: PlayerInput,

    /// 上一帧的输入
    pub previous: PlayerInput,

    /// 输入序列号
    pub sequence: u32,

    /// 按键状态
    pub keys: ButtonInput<KeyCode>,

    /// 鼠标按钮状态
    pub mouse_buttons: ButtonInput<MouseButton>,

    /// 游戏手柄按钮状态
    pub gamepad_buttons: Vec<(Gamepad, ButtonInput<GamepadButton>)>,

    /// 鼠标位置
    pub mouse_position: Vec2,

    /// 鼠标移动增量
    pub mouse_delta: Vec2,

    /// 鼠标滚轮
    pub mouse_wheel: f32,

    /// 时间戳
    pub timestamp: Instant,
}

impl InputState {
    /// 是否按下某个键
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.keys.pressed(key)
    }

    /// 是否刚按下某个键
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.keys.just_pressed(key)
    }

    /// 是否刚释放某个键
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.keys.just_released(key)
    }

    /// 是否按下某个鼠标按钮
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.pressed(button)
    }

    /// 是否刚按下某个鼠标按钮
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.just_pressed(button)
    }

    /// 是否刚释放某个鼠标按钮
    pub fn mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons.just_released(button)
    }

    /// 获取移动方向
    pub fn move_direction(&self) -> Vec2 {
        let mut dir = Vec2::ZERO;

        if self.pressed(KeyCode::KeyW) || self.pressed(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if self.pressed(KeyCode::KeyS) || self.pressed(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if self.pressed(KeyCode::KeyA) || self.pressed(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if self.pressed(KeyCode::KeyD) || self.pressed(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        if dir != Vec2::ZERO {
            dir = dir.normalize();
        }

        dir
    }

    /// 是否在奔跑
    pub fn is_sprinting(&self) -> bool {
        self.pressed(KeyCode::ShiftLeft) || self.pressed(KeyCode::ShiftRight)
    }

    /// 是否在蹲伏
    pub fn is_crouching(&self) -> bool {
        self.pressed(KeyCode::ControlLeft) || self.pressed(KeyCode::KeyC)
    }

    /// 是否在瞄准 (右键)
    pub fn is_aiming(&self) -> bool {
        self.mouse_pressed(MouseButton::Right)
    }

    /// 是否在攻击 (左键)
    pub fn is_attacking(&self) -> bool {
        self.mouse_pressed(MouseButton::Left)
    }

    /// 获取技能槽索引 (1-0)
    pub fn get_skill_slot(&self) -> Option<usize> {
        let skill_keys = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
            KeyCode::Digit0,
        ];

        for (i, &key) in skill_keys.iter().enumerate() {
            if self.just_pressed(key) {
                return Some(i);
            }
        }

        None
    }

    /// 获取物品槽索引
    pub fn get_item_slot(&self) -> Option<usize> {
        if self.just_pressed(KeyCode::KeyQ) {
            return Some(0);
        }
        if self.just_pressed(KeyCode::KeyR) {
            return Some(1);
        }

        None
    }

    /// 收集当前输入为 PlayerInput
    pub fn collect_input(&mut self, config: &InputConfig) -> PlayerInput {
        let move_dir = self.move_direction();

        let mut actions = Vec::new();

        // 移动
        if move_dir != Vec2::ZERO {
            actions.push(InputAction::Move {
                direction: move_dir,
            });
        }

        // 奔跑
        if self.is_sprinting() {
            actions.push(InputAction::Sprint(true));
        }

        // 蹲伏
        if self.is_crouching() {
            actions.push(InputAction::Crouch);
        }

        // 跳跃
        if self.just_pressed(KeyCode::Space) {
            actions.push(InputAction::Jump);
        }

        // 攻击
        if self.is_attacking() {
            actions.push(InputAction::Attack);
        }

        // 格挡
        if self.is_aiming() {
            actions.push(InputAction::Block);
        }

        // 闪避
        if self.just_pressed(KeyCode::AltLeft) {
            let dodge_dir = if move_dir != Vec2::ZERO {
                Vec3::new(move_dir.x, 0.0, move_dir.y)
            } else {
                Vec3::NEG_Z // 向后闪避
            };
            actions.push(InputAction::Dodge {
                direction: dodge_dir,
            });
        }

        // 技能
        if let Some(slot) = self.get_skill_slot() {
            if let Some(skill_id) = &config.keybinds.skills.get(slot).and_then(|_| None) {
                // TODO: 从配置获取技能ID
                actions.push(InputAction::Skill {
                    skill_id: format!("skill_{}", slot),
                    target: None,
                });
            }
        }

        // 交互
        if self.just_pressed(KeyCode::KeyE) {
            actions.push(InputAction::Interact {
                target: EntityId::MAX, // TODO: 获取当前目标
            });
        }

        // 菜单
        if self.just_pressed(KeyCode::Tab) || self.just_pressed(KeyCode::KeyI) {
            // 打开背包
        }

        PlayerInput {
            sequence: self.sequence,
            tick: 0, // 由服务器填充
            actions,
            position: Vec3::ZERO,  // 由玩家状态填充
            rotation: Quat::IDENTITY, // 由玩家状态填充
        }
    }

    /// 更新到下一帧
    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = Instant::now();

        // 重置一次性状态
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel = 0.0;
    }
}

/// 收集键盘输入
fn collect_keyboard_input(
    mut input_state: ResMut<InputState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    input_state.keys = keys.clone();
}

/// 收集鼠标输入
fn collect_mouse_input(
    mut input_state: ResMut<InputState>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion_reader: EventReader<MouseMotion>,
    mut scroll_reader: EventReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
) {
    input_state.mouse_buttons = buttons.clone();

    // 鼠标移动
    for event in motion_reader.read() {
        input_state.mouse_delta += event.delta;
    }

    // 鼠标滚轮
    for event in scroll_reader.read() {
        input_state.mouse_wheel += event.y;
    }

    // 鼠标位置
    if let Ok(window) = windows.get_single() {
        if let Some(position) = window.cursor_position() {
            input_state.mouse_position = position;
        }
    }
}

/// 收集游戏手柄输入
fn collect_gamepad_input(
    mut input_state: ResMut<InputState>,
    gamepads: Query<(Entity, &Gamepad)>,
    buttons: Res<ButtonInput<GamepadButton>>,
    mut axis_events: EventReader<bevy::input::gamepad::GamepadAxisChangedEvent>,
) {
    input_state.gamepad_buttons.clear();

    for (entity, gamepad) in gamepads.iter() {
        let button_input = ButtonInput::<GamepadButton>::default(); // 需要从系统获取

        input_state.gamepad_buttons.push((*gamepad, button_input));
    }
}

/// 更新输入状态
fn update_input_state(
    mut input_state: ResMut<InputState>,
    config: Res<ClientConfig>,
) {
    input_state.current = input_state.collect_input(&config.input);
    input_state.advance();
}
