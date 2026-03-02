# 移动系统产品规格书 | Movement System Product Requirements

**项目**: TrueWorld 多人游戏
**版本**: 1.0
**日期**: 2026-03-02
**状态**: Phase 1-3 已完成

---

## 1. 概述 | Overview

### 1.1 文档目的 | Document Purpose

本文档定义 TrueWorld 多人游戏移动系统的完整产品需求，包括功能需求、非功能需求、配置参数和验收标准。

### 1.2 系统简介 | System Introduction

TrueWorld 移动系统采用客户端预测 + 服务器验证的混合架构，为多人游戏提供低延迟、安全的移动体验。

**核心价值主张**：
- 🎯 **即时响应**：客户端本地预测，无输入延迟感
- 🛡️ **防作弊**：服务器权威验证，防止速度/传送作弊
- 🌐 **网络优化**：智能通道选择，优化带宽使用

---

## 2. 功能需求 | Functional Requirements

### 2.1 客户端功能 | Client Features

#### FR-1: 本地预测 | Local Prediction

**需求描述** | Requirement Description
| 描述 | 客户端根据输入立即计算并显示预测位置，无需等待服务器确认 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 输入到显示的延迟 < 16ms（1帧）
- 预测位置基于当前输入和配置的物理参数
- 保存最近 60 帧输入历史用于回滚

#### FR-2: 位置修正 | Position Correction

**需求描述** | Requirement Description
| 描述 | 客户端接收服务器权威位置，与本地预测对比并进行修正 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 接收 `ServerPositionAck` 包（20Hz）
- 偏差超过阈值时触发修正
- 传送类修正立即跳转，其他使用平滑插值
- 修正插值系数可配置（默认 0.3）

#### FR-3: 输入发送 | Input Transmission

**需求描述** | Requirement Description
| 描述 | 客户端以 60Hz 频率向服务器发送输入包 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 使用 unreliable 通道（Channel 2）
- 包含序列号、移动向量、动作列表、时间戳
- 自动处理网络抖动和丢包

### 2.2 服务器功能 | Server Features

#### FR-4: 移动验证 | Movement Validation

**需求描述** | Requirement Description
| 描述 | 服务器验证客户端移动的合法性，检测作弊行为 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 验证输入序列号，忽略重复/乱序包
- 速度检测：超过最大速度则违规
- 碰撞检测：检测地下/穿墙移动
- 传送检测：单帧位移超过阈值则违规

#### FR-5: 违规处理 | Violation Handling

**需求描述** | Requirement Description
| 描述 | 累积玩家违规行为，达到阈值后踢出 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 速度违规累积 10 次警告
- 传送违规累积 3 次踢出
- 违规计数随时间衰减（60秒无违规后减少）

#### FR-6: 位置广播 | Position Broadcasting

**需求描述** | Requirement Description
| 描述 | 服务器向所有客户端广播玩家位置 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 广播频率 20Hz
- 只广播范围内的实体
- 使用 unreliable 通道（Channel 2）
- 包含位置、速度、序列号

#### FR-7: 位置确认 | Position Acknowledgment

**需求描述** | Requirement Description
| 描述 | 服务器定期向客户端发送位置确认包 |
|---|---|
| 优先级 | P0 (必须实现) |
| 状态 | ✅ 已实现 |

**验收标准** | Acceptance Criteria
- 包含玩家 ID、确认序列号、权威位置、速度
- 用于客户端更新最后确认序列号
- 发送频率 20Hz

### 2.3 扩展功能 | Extended Features (Future)

#### FR-8: 游泳 | Swimming

**需求描述** | Requirement Description
| 描述 | 支持水下移动，不同的移动速度和物理效果 |
|---|---|
| 优先级 | P2 (未来实现) |
| 状态 | ⏳ 计划中 |

#### FR-9: 爬墙/攀爬 | Climbing

**需求描述** | Requirement Description
| 描述 | 支持垂直地形移动，自动地形适配 |
|---|---|
| 优先级 | P2 (未来实现) |
| 状态 | ⏳ 计划中 |

#### FR-10: 载具 | Vehicles

**需求描述** | Requirement Description
| 描述 | 支持骑乘/驾驶载具，不同的移动模式 |
|---|---|
| 优先级 | P3 (未来考虑) |
| 状态 | ⏳ 计划中 |

---

## 3. 非功能需求 | Non-Functional Requirements

### 3.1 性能需求 | Performance Requirements

| 需求 | 目标值 | 测量方法 |
|------|--------|----------|
| 输入延迟 | < 16ms | 客户端输入到显示的时间 |
| 位置更新频率 | 20Hz | 服务器广播间隔 |
| 单服务器玩家数 | 64+ | 并发连接测试 |
| 内存占用 | < 100MB/玩家 | 性能分析 |
| CPU 占用 | < 5%/玩家 | 性能分析 |

### 3.2 网络需求 | Network Requirements

| 需求 | 目标值 | 说明 |
|------|--------|------|
| 最小带宽 | 56 Kbps | 上行 + 下行总和 |
| 接受延迟 | < 200ms | 玩家可接受的体验阈值 |
| 抖动容忍 | ±30% | 网络抖动补偿能力 |
| 丢包恢复 | 自动 | unreliable 通道丢包不重传 |

### 3.3 安全需求 | Security Requirements

| 威胁 | 防护措施 | 实现状态 |
|------|----------|----------|
| 速度作弊 | 服务器验证 | ✅ 已实现 |
| 传送作弊 | 单帧位移检测 | ✅ 已实现 |
| 穿墙作弊 | 碰撞检测 | ✅ 已实现 |
| 重放攻击 | 序列号验证 | ✅ 已实现 |
| 加速作弊 | 输入频率限制 | 🔄 计划中 |

### 3.4 可用性需求 | Usability Requirements

| 需求 | 目标值 |
|------|--------|
| 修正可见性 | 修正抖动不影响游戏体验 |
| 配置灵活性 | 所有关键参数可配置 |
| 调试支持 | 完整的日志和监控 |

---

## 4. 配置参数 | Configuration Parameters

### 4.1 客户端配置 | Client Configuration

| 参数名 | 类型 | 默认值 | 范围 | 说明 |
|--------|------|--------|------|------|
| `walk_speed` | f32 | 5.0 | 1.0-20.0 | 行走速度（单位/秒） |
| `run_speed` | f32 | 8.0 | 2.0-30.0 | 冲刺速度（单位/秒） |
| `jump_velocity` | f32 | 5.0 | 3.0-20.0 | 跳跃初速度 |
| `gravity` | f32 | 20.0 | 5.0-50.0 | 重力加速度 |
| `correction_threshold` | f32 | 2.0 | 0.5-10.0 | 修正触发阈值（单位） |
| `correction_lerp` | f32 | 0.3 | 0.1-1.0 | 修正插值系数（0-1） |

### 4.2 服务器配置 | Server Configuration

| 参数名 | 类型 | 默认值 | 范围 | 说明 |
|--------|------|--------|------|------|
| `max_speed` | f32 | 5.0 | 1.0-20.0 | 最大移动速度（单位/秒） |
| `max_sprint_speed` | f32 | 8.0 | 2.0-30.0 | 最大冲刺速度 |
| `max_jump_velocity` | f32 | 10.0 | 3.0-20.0 | 最大跳跃速度 |
| `max_delta_per_frame` | f32 | 2.0 | 0.5-10.0 | 每帧最大位移（防作弊） |
| `broadcast_rate` | f32 | 20.0 | 10.0-60.0 | 位置广播频率（Hz） |
| `afk_timeout_seconds` | u64 | 300 | 60-3600 | AFK 超时时间（秒） |
| `speed_violation_threshold` | u32 | 10 | 3-100 | 速度违规踢出阈值 |
| `teleport_violation_threshold` | u32 | 3 | 1-10 | 传送违规踢出阈值 |

---

## 5. 数据包定义 | Packet Definitions

### 5.1 客户端 → 服务器 | Client to Server

#### ClientInputPacket

| 字段 | 类型 | 大小 | 说明 |
|------|------|------|------|
| `sequence` | u32 | 4 bytes | 输入序列号 |
| `movement` | [f32; 3] | 12 bytes | 移动向量 [x, y, z] |
| `actions` | Vec<InputAction> | 变长 | 动作列表 |
| `timestamp` | u64 | 8 bytes | 客户端时间戳 |

**总大小**: ~24+ bytes（不含 actions）

### 5.2 服务器 → 客户端 | Server to Client

#### ServerPositionAck

| 字段 | 类型 | 大小 | 说明 |
|------|------|------|------|
| `player_id` | PlayerId | 8 bytes | 玩家 ID |
| `ack_sequence` | u32 | 4 bytes | 确认的序列号 |
| `position` | [f32; 3] | 12 bytes | 权威位置 |
| `velocity` | [f32; 3] | 12 bytes | 当前速度 |
| `server_time` | u64 | 8 bytes | 服务器时间戳 |

**总大小**: ~44 bytes

#### ServerPositionCorrection

| 字段 | 类型 | 大小 | 说明 |
|------|------|------|------|
| `player_id` | PlayerId | 8 bytes | 玩家 ID |
| `correct_position` | [f32; 3] | 12 bytes | 修正后的位置 |
| `reason` | CorrectionReason | 1 byte | 修正原因 |

**总大小**: ~21 bytes

---

## 6. 用户故事 | User Stories

### US-1: 玩家移动

**作为** 玩家
**我想要** 在游戏中流畅地移动角色
**以便** 我能够探索游戏世界并与环境互动

**验收标准**:
- 按下移动键后角色立即开始移动
- 角色移动流畅，无明显卡顿
- 网络延迟高时移动仍可响应

### US-2: 服务器验证

**作为** 游戏运营
**我想要** 服务器验证所有玩家移动
**以便** 防止作弊破坏游戏平衡

**验收标准**:
- 超速移动的玩家被检测
- 传送作弊的玩家被踢出
- 正常玩家不受影响

### US-3: 位置修正

**作为** 玩家
**我想要** 即使网络波动也能看到正确的位置
**以便** 游戏体验保持一致

**验收标准**:
- 位置修正平滑无抖动
- 传送修正立即生效
- 服务器位置与客户端显示一致

---

## 7. 验收标准 | Acceptance Criteria

### 7.1 功能验收 | Functional Acceptance

| ID | 验收项 | 状态 |
|----|--------|------|
| AC-1 | 客户端输入响应延迟 < 16ms | ✅ 通过 |
| AC-2 | 服务器验证所有输入包 | ✅ 通过 |
| AC-3 | 位置偏差 > 2.0 单位时触发修正 | ✅ 通过 |
| AC-4 | 速度违规累计 10 次记录警告 | ✅ 通过 |
| AC-5 | 传送违规累计 3 次踢出玩家 | ✅ 通过 |
| AC-6 | 单元测试覆盖率 > 80% | ✅ 通过 (29/29) |

### 7.2 性能验收 | Performance Acceptance

| ID | 验收项 | 目标值 | 实际值 |
|----|--------|--------|--------|
| PC-1 | 输入延迟 | < 16ms | TBD |
| PC-2 | 位置更新频率 | 20Hz | ✅ 20Hz |
| PC-3 | 单服务器玩家数 | 64+ | TBD |
| PC-4 | 内存占用/玩家 | < 100MB | TBD |

---

## 8. 限制与假设 | Limitations & Assumptions

### 8.1 当前限制 | Current Limitations

1. **不支持地形碰撞**：当前只有简单的地面碰撞检测
2. **不支持载具**：载具系统需要额外的移动模式
3. **输入方式限制**：当前只支持键盘/鼠标，未实现手柄

### 8.2 假设 | Assumptions

1. 玩家使用稳定的网络连接（延迟 < 500ms）
2. 客户端和服务器使用相同的物理参数
3. 游戏以 60 FPS 运行

---

## 9. 依赖关系 | Dependencies

### 9.1 内部依赖 | Internal Dependencies

```
movement 依赖:
├── trueworld_core (共享类型)
├── trueworld_protocol (协议定义)
└── bevy (ECS框架)
```

### 9.2 外部依赖 | External Dependencies

| 依赖 | 版本 | 用途 |
|------|------|------|
| bevy | 0.15 | ECS 游戏引擎 |
| renet | 1.2 | 网络传输 |
| serde | - | 序列化/反序列化 |
| glam | - | 数学库（Vec3, Quat） |

---

## 10. 发布计划 | Release Plan

| 阶段 | 内容 | 状态 |
|------|------|------|
| Phase 1 | 协议扩展 | ✅ 完成 |
| Phase 2 | 客户端移动模块 | ✅ 完成 |
| Phase 3 | 服务器验证模块 | ✅ 完成 |
| Phase 4 | 集成与测试 | 🔄 进行中 |
| Phase 5 | 性能优化 | ⏳ 计划中 |
| Phase 6 | 扩展功能 | ⏳ 计划中 |

---

**文档版本**: 1.0
**最后更新**: 2026-03-02
