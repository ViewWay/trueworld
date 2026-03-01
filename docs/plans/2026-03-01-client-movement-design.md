# 客户端输入与移动系统设计

**日期**: 2026-03-01
**作者**: Claude & User
**状态**: 设计阶段

## 概述

实现客户端输入与移动系统，采用混合验证模式，平衡性能与安全性。

- 普通移动：客户端预测，服务器验证
- 关键动作：服务器权威验证

## 架构

### 数据流图

```
┌─────────────────────────────────────────────────────────────────┐
│                       客户端                                       │
├─────────────────────────────────────────────────────────────────┤
│  输入收集 (InputState)  →  本地预测 (PlayerPosition)           │
│         ↓                                                      │
│  发送输入 [60Hz] ────────────────────────────────────────┐      │
│         ↓                                                │      │
│  [普通移动] 直接本地更新位置，立即显示                     │      │
│  [关键动作] 发送给服务器，等待确认                          │      │
└───────────────────────────────────────────────────────────┼─────┘
                                                           │
                                                           ↓
┌─────────────────────────────────────────────────────────────────┐
│                       服务器                                       │
├─────────────────────────────────────────────────────────────────┤
│  接收客户端输入 → 验证合理性 (速度、防穿墙)                       │
│         ↓                                                      │
│  更新权威位置 → 广播给所有玩家                                   │
│         ↓                                                      │
│  处理关键动作 (攻击、拾取、交互) → 发送结果                      │
└─────────────────────────────────────────────────────────────────┘
                           ↑
                           │
                    位置更新/确认
                           │
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│                       客户端 (修正)                               │
├─────────────────────────────────────────────────────────────────┤
│  接收服务器位置 → 与本地预测对比                                  │
│         ↓                                                      │
│  偏差 > 阈值 → 修正位置 (平滑插值)                               │
│  偏差 ≤ 阈值 → 保持本地预测 (减少抖动)                           │
└─────────────────────────────────────────────────────────────────┘
```

## 组件职责

| 组件 | 位置 | 职责 |
|------|------|------|
| `InputState` | 客户端 | 已有，收集键盘/鼠标/手柄输入 |
| `PlayerInput` | 共享 | 已有，输入数据结构 |
| `ClientMovementPlugin` | 客户端 (新增) | 本地预测，发送输入 |
| `MovementValidation` | 服务器 (新增) | 验证移动合理性 |
| `PositionCorrection` | 客户端 (新增) | 服务器位置修正 |

## 客户端实现

### 文件结构

```
crates/client/src/
├── movement/           # 新增模块
│   ├── mod.rs          # 模块导出
│   ├── plugin.rs       # ClientMovementPlugin
│   ├── prediction.rs   # 本地预测系统
│   └── correction.rs   # 服务器位置修正
├── input.rs            # 已有，输入收集
└── network.rs          # 已有，发送输入
```

### 核心数据结构

```rust
/// 本地预测的玩家状态
#[derive(Component)]
pub struct PredictedState {
    /// 预测的位置
    pub position: Vec3,
    /// 上次确认的序列号
    pub last_ack_sequence: u32,
    /// 未确认的输入历史 (用于回滚)
    pub input_history: VecDeque<InputSnapshot>,
}

/// 输入快照，用于回滚修正
pub struct InputSnapshot {
    pub sequence: u32,
    pub input: PlayerInput,
    pub position: Vec3,
}

/// 移动配置
#[derive(Resource)]
pub struct MovementConfig {
    /// 移动速度 (单位/秒)
    pub walk_speed: f32,
    pub run_speed: f32,
    /// 跳跃速度
    pub jump_velocity: f32,
    /// 位置修正阈值 (超过此值才修正)
    pub correction_threshold: f32,
    /// 修正平滑速度 (0-1)
    pub correction_lerp: f32,
}
```

### 系统流程

```
每帧 Update:
┌─────────────────────────────────────────────────────────────┐
│ 1. collect_input                    # 已有系统              │
│    → InputState 获取当前输入                                 │
├─────────────────────────────────────────────────────────────┤
│ 2. predict_movement (新增)                                   │
│    → 根据 InputState 计算新位置                              │
│    → 更新 PredictedState.position                            │
│    → 保存 InputSnapshot 到历史                               │
├─────────────────────────────────────────────────────────────┤
│ 3. send_input_to_server (新增)                               │
│    → 每帧发送 PlayerInput 到服务器                           │
│    → 使用 unreliable channel (channel_id: 2)                │
├─────────────────────────────────────────────────────────────┤
│ 4. correct_position (新增)                                   │
│    → 检查服务器发来的位置                                    │
│    → 偏差 > threshold → 平滑修正                            │
│    → 偏差 ≤ threshold → 忽略                                 │
└─────────────────────────────────────────────────────────────┘
```

## 服务器端实现

### 文件结构

```
crates/server/src/
├── movement/           # 新增模块
│   ├── mod.rs          # 模块导出
│   ├── validation.rs   # 移动验证系统
│   └── update.rs       # 位置更新处理
├── player.rs           # 已有，需要扩展
└── entity.rs           # 已有，实体系统
```

### 核心数据结构

```rust
/// 移动验证结果
pub enum ValidationResult {
    /// 移动有效
    Valid,
    /// 移动过快
    TooFast { max_speed: f32, actual: f32 },
    /// 穿墙检测失败
    Collision { at: Vec3 },
    /// 位置跳变过大 (疑似传送作弊)
    TeleportDetected { from: Vec3, to: Vec3 },
}

/// 服务器上的玩家移动状态
#[derive(Component)]
pub struct ServerPlayerMovement {
    /// 当前权威位置
    pub position: Vec3,
    /// 当前速度
    pub velocity: Vec3,
    /// 上次更新时间
    pub last_update: GameTime,
    /// 上次确认的客户端输入序列
    pub last_client_sequence: u32,
}

/// 移动配置
#[derive(Clone, Resource)]
pub struct ServerMovementConfig {
    /// 最大移动速度 (单位/秒)
    pub max_speed: f32,
    /// 最大单帧位移 (防止穿墙)
    pub max_delta_per_frame: f32,
    /// 位置更新广播频率
    pub broadcast_rate: f32,
}
```

### 验证算法

服务器不直接信任客户端位置，而是验证"输入→位移"的合理性：

1. **速度验证**: 检查位移是否超过最大速度
2. **碰撞检测**: 检查新位置是否在墙壁内
3. **反传送**: 检查单帧位移是否过大

### 广播策略

| 方案 | 频率 | 优点 | 缺点 |
|------|------|------|------|
| 每帧广播 | 60Hz | 最实时 | 带宽高 |
| 20Hz 广播 | 20Hz | 平衡 | 需要插值 |
| 按需广播 | 变化时 | 最省带宽 | 静止显示可能不准确 |

**推荐**: 20Hz 广播，配合客户端插值。

## 协议定义

### 客户端 → 服务器

```rust
/// 客户端输入数据包 (unreliable channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInputPacket {
    /// 输入序列号
    pub sequence: u32,
    /// 移动方向 [x, y, z]
    pub movement: [f32; 3],
    /// 动作列表
    pub actions: Vec<InputAction>,
    /// 时间戳
    pub timestamp: u64,
}
```

### 服务器 → 客户端

```rust
/// 服务器位置确认 (unreliable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPositionAck {
    pub player_id: PlayerId,
    pub ack_sequence: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub server_time: u64,
}

/// 位置修正 (reliable - 重要)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPositionCorrection {
    pub player_id: PlayerId,
    pub correct_position: [f32; 3],
    pub reason: CorrectionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionReason {
    SpeedLimitExceeded,
    Collision,
    ServerRollback,
}
```

### 通道使用

| 通道 | 类型 | 用途 |
|------|------|------|
| 0 | ReliableOrdered | 错误消息、关键数据 |
| 1 | ReliableUnordered | 聊天、状态更新 |
| 2 | Unreliable | 位置更新、输入 |

## 错误处理

### 客户端

| 场景 | 处理方式 |
|------|----------|
| 网络中断 | 保存输入历史，重连后重发 |
| 位置差异过大 | 强制修正到服务器位置 |
| 服务器拒绝 | 停止预测，等待新确认 |

### 服务器

| 场景 | 处理方式 |
|------|----------|
| 输入延迟 | 忽略 500ms 之前的输入 |
| 速度超标 | 记录日志，修正位置 |
| 持续异常 | 踢出玩家 |

## 实现步骤

1. **协议扩展** - 添加新的数据包类型
2. **客户端移动模块** - 实现本地预测
3. **服务器验证模块** - 实现移动验证
4. **位置修正** - 实现服务器位置同步
5. **测试** - 单元测试和集成测试

## 技术要点

- **60Hz 输入发送**: 客户端以 60fps 发送输入
- **输入历史回滚**: 保存 60 帧快照用于修正
- **平滑修正**: 使用 lerp 插值避免抖动
- **序列号机制**: 追踪已确认的输入
- **混合通道**: unreliable 用于位置，reliable 用于修正
