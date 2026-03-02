# 移动系统架构设计 | Movement System Architecture Design

**项目**: TrueWorld 多人游戏
**版本**: 1.0
**日期**: 2026-03-02
**状态**: 已实现 Phase 1-3

---

## 1. 系统概述 | System Overview

### 1.1 设计目标 | Design Goals

移动系统采用 **客户端预测 + 服务器验证** 的混合架构，平衡性能与安全性：

```mermaid
graph LR
    A[Client Input] --> B[Local Prediction]
    B --> C[Immediate Display]
    A --> D[Server Validation]
    D --> E[Authoritative Update]
    E --> C
    E --> F[Position Ack]
    F --> G[Client Correction]
    G --> C
```

| 目标 | 实现方式 |
|------|----------|
| 低延迟输入 | 客户端本地预测，立即显示 |
| 作弊防护 | 服务器验证移动，权威位置 |
| 平滑体验 | 位置修正使用插值平滑 |
| 网络优化 | 分层通道，高频数据用 unreliable |

### 1.2 核心原则 | Core Principles

1. **服务器权威** (Server Authority)
   - 所有位置最终以服务器为准
   - 客户端预测仅用于本地显示

2. **输入驱动** (Input-Driven)
   - 服务器接收输入而非位置
   - 基于输入计算位移

3. **宽容验证** (Tolerant Validation)
   - 允许合理网络抖动
   - 违规累积阈值机制

---

## 2. 架构分层 | Architecture Layers

```mermaid
graph TB
    subgraph "Client Side"
        C1[Input Collection]
        C2[Movement Plugin]
        C3[Prediction System]
        C4[Correction System]
    end

    subgraph "Protocol Layer"
        P1[Client Packets]
        P2[Server Packets]
        P3[Channels]
    end

    subgraph "Server Side"
        S1[Movement Validator]
        S2[Update Processor]
        S3[Violation Manager]
        S4[Position Broadcaster]
    end

    C1 --> C3
    C3 --> C4
    C2 --> P1
    P1 --> P3
    P3 --> S2
    S2 --> S1
    S2 --> S3
    S4 --> P2
    P2 --> P3
    P3 --> C4
```

### 2.1 客户端分层 | Client Layers

| 层级 | 模块 | 文件 | 职责 |
|------|------|------|------|
| 输入层 | Input System | `client/src/input.rs` | 收集键盘/鼠标/手柄输入 |
| 逻辑层 | Movement Plugin | `client/src/movement/plugin.rs` | 管理移动配置和系统注册 |
| 预测层 | Prediction | `client/src/movement/prediction.rs` | 本地预测位置计算 |
| 修正层 | Correction | `client/src/movement/correction.rs` | 服务器位置同步 |

### 2.2 服务器分层 | Server Layers

| 层级 | 模块 | 文件 | 职责 |
|------|------|------|------|
| 配置层 | Config | `server/src/movement/config.rs` | 配置参数和违规追踪 |
| 验证层 | Validation | `server/src/movement/validation.rs` | 移动验证算法 |
| 处理层 | Update Processor | `server/src/movement/update.rs` | 输入处理和状态更新 |
| 广播层 | Network | `server/src/network.rs` | 消息广播 |

---

## 3. 数据流设计 | Data Flow Design

### 3.1 正常移动流程 | Normal Movement Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant P as Prediction
    participant N as Network
    participant S as Server
    participant V as Validator

    C->>P: 收集输入 (60Hz)
    P->>P: 计算预测位置
    P->>C: 立即显示
    C->>N: 发送 ClientInputPacket
    N->>S: 传输 (Channel 2, Unreliable)
    S->>V: 验证移动
    V->>S: 返回验证结果
    S->>S: 更新权威位置
    S->>N: 发送 ServerPositionAck (20Hz)
    N->>C: 传输
    C->>C: 更新最后确认序列号
```

### 3.2 位置修正流程 | Position Correction Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant P as Prediction
    participant N as Network
    participant S as Server
    participant V as Validator

    C->>N: 发送 ClientInputPacket
    N->>S: 传输
    S->>V: 验证移动
    V->>V: 检测违规 (TooFast/Collision/Teleport)
    V->>S: 返回违规结果
    S->>N: 发送 ServerPositionCorrection (Channel 0, Reliable)
    N->>C: 传输
    C->>P: 接收修正
    P->>P: 平滑插值到正确位置
    Note over P: correction_lerp = 0.3
```

### 3.3 违规处理流程 | Violation Handling Flow

```mermaid
stateDiagram-v2
    [*] --> ReceiveInput: 收到客户端输入
    ReceiveInput --> Validate: 验证序列号
    Validate --> OldInput: 序列号过期
    Validate --> CheckMovement: 序列号有效
    OldInput --> [*]

    CheckMovement --> Valid: 移动有效
    CheckMovement --> SpeedViolation: 速度超限
    CheckMovement --> Collision: 碰撞检测
    CheckMovement --> Teleport: 传送检测

    Valid --> UpdatePosition: 更新位置
    UpdatePosition --> SendAck: 发送确认包
    SendAck --> [*]

    SpeedViolation --> RecordViolation: 记录违规
    Collision --> RecordViolation: 记录违规
    Teleport --> RecordViolation: 记录违规

    RecordViolation --> CheckThreshold: 检查阈值
    CheckThreshold --> SendCorrection: 未达阈值
    CheckThreshold --> KickPlayer: 达到阈值

    SendCorrection --> [*]
    KickPlayer --> [*]: 踢出玩家
```

---

## 4. 组件关系图 | Component Relationships

```mermaid
erDiagram
    Client ||--o{ MovementConfig : uses
    Client ||--o{ PredictedState : manages
    Client ||--o{ InputSnapshot : stores

    Prediction ||--|| MovementConfig : references
    Prediction ||--o{ PredictedState : updates
    Prediction ||--o{ InputSnapshot : creates

    Correction ||--|| PositionCorrection : uses
    Correction ||--o{ PredictedState : corrects
    Correction ||--o{ MovementConfig : references

    Server ||--o{ MovementUpdateProcessor : contains
    Server ||--o{ ViolationManager : contains

    MovementUpdateProcessor ||--|| MovementValidator : uses
    MovementUpdateProcessor ||--o{ ServerPlayerMovement : manages
    MovementUpdateProcessor ||--o{ ViolationManager : tracks

    MovementValidator ||--|| ServerMovementConfig : references
    MovementValidator ||--|| ValidationResult : returns

    ViolationManager ||--o{ PlayerViolationTracker : manages
```

---

## 5. 关键数据结构 | Key Data Structures

### 5.1 客户端数据 | Client Data

```mermaid
classDiagram
    class PredictedState {
        +Vec3 position
        +Vec3 velocity
        +u32 last_ack_sequence
        +VecDeque~InputSnapshot~ input_history
    }

    class InputSnapshot {
        +u32 sequence
        +PlayerInput input
        +Vec3 position
    }

    class PositionCorrection {
        +Option~CorrectionTarget~ target
        +Option~Vec3~ last_server_position
    }

    class MovementConfig {
        +f32 walk_speed
        +f32 run_speed
        +f32 jump_velocity
        +f32 gravity
        +f32 correction_threshold
        +f32 correction_lerp
    }

    PredictedState "1" *-- "60" InputSnapshot : stores
    PositionCorrection .. MovementConfig : references
```

### 5.2 服务器数据 | Server Data

```mermaid
classDiagram
    class ServerPlayerMovement {
        +Position position
        +Velocity velocity
        +f32 rotation
        +bool on_ground
        +bool is_sprinting
        +u32 last_client_sequence
        +u64 last_update_tick
        +Position last_position
    }

    class PlayerViolationTracker {
        +u32 speed_violations
        +u32 teleport_violations
        +u32 collision_violations
        +Option~u64~ last_violation_time
        +bool mark_for_kick
    }

    class ServerMovementConfig {
        +f32 max_speed
        +f32 max_sprint_speed
        +f32 max_jump_velocity
        +f32 max_delta_per_frame
        +f32 broadcast_rate
        +u64 afk_timeout_seconds
        +u32 speed_violation_threshold
        +u32 teleport_violation_threshold
    }

    class MovementUpdateProcessor {
        +MovementValidator validator
        +HashMap~PlayerId, ServerPlayerMovement~ player_states
        +ViolationManager violations
        +Vec~ServerPositionCorrection~ pending_corrections
        +Vec~ServerPositionAck~ pending_acks
        +u64 current_tick
    }
```

---

## 6. 网络拓扑 | Network Topology

```mermaid
graph TB
    subgraph "Network Channels"
        direction TB
        CH0["Channel 0<br/>ReliableOrdered<br/>5MB buffer<br/>关键数据"]
        CH1["Channel 1<br/>ReliableUnordered<br/>5MB buffer<br/>状态更新"]
        CH2["Channel 2<br/>Unreliable<br/>10MB buffer<br/>位置更新"]
    end

    subgraph "Client → Server"
        CI1["ClientInputPacket<br/>60Hz"]
        CI2["ConnectMessage<br/>连接时"]
    end

    subgraph "Server → Client"
        SI1["ServerPositionAck<br/>20Hz"]
        SI2["ServerPositionCorrection<br/>按需"]
        SI3["ConnectResultMessage<br/>连接响应"]
    end

    CI1 --> CH2
    CI2 --> CH0
    SI1 --> CH2
    SI2 --> CH0
    SI3 --> CH0
```

---

## 7. 性能与扩展性 | Performance & Scalability

### 7.1 性能指标 | Performance Metrics

| 指标 | 目标值 | 当前实现 |
|------|--------|----------|
| 客户端输入频率 | 60Hz | ✅ 实现 |
| 服务器 tick rate | 60Hz | ✅ 实现 |
| 位置广播频率 | 20Hz | ✅ 实现 |
| 单服务器玩家数 | 64+ | 可扩展 |
| 输入历史容量 | 60帧 | ✅ 实现 |

### 7.2 网络带宽估算 | Bandwidth Estimation

```
每个玩家每秒带宽（上行）:
ClientInputPacket: ~32 bytes × 60 Hz = 1.92 KB/s

每个玩家每秒带宽（下行）:
ServerPositionAck: ~40 bytes × 20 Hz = 0.8 KB/s
ServerPositionCorrection: ~32 bytes × 按需 = 可变

64玩家服务器总下行:
≈ 64 × 0.8 KB/s = 51.2 KB/s (基础)
+ 其他实体更新 ≈ 100-200 KB/s
```

---

## 8. 安全机制 | Security Mechanisms

```mermaid
graph LR
    I[Client Input] --> SN[序列号验证]
    SN --> OV{旧输入?}
    OV -->|是| IG[忽略]
    OV -->|否| MV[移动验证]

    MV --> SD[速度检测]
    MV --> CD[碰撞检测]
    MV --> TD[传送检测]

    SD --> VT{违规?}
    CD --> VT
    TD --> VT

    VT -->|是| VR[记录违规]
    VT -->|否| UP[更新位置]

    VR --> TH{达到阈值?}
    TH -->|是| KICK[踢出玩家]
    TH -->|否| CORR[发送修正]
```

### 安全配置 | Security Configuration

| 参数 | 默认值 | 作用 |
|------|--------|------|
| max_delta_per_frame | 2.0 | 单帧最大位移 |
| speed_violation_threshold | 10 | 速度违规阈值 |
| teleport_violation_threshold | 3 | 传送违规阈值（踢出） |

---

## 9. 文件组织 | File Organization

```
trueworld/
├── crates/
│   ├── protocol/src/
│   │   ├── client.rs          # ClientInputPacket, ClientInput
│   │   ├── server.rs          # ServerPositionAck, ServerPositionCorrection
│   │   └── lib.rs             # Channel configuration
│   │
│   ├── core/src/
│   │   ├── types.rs            # PlayerInput, InputAction
│   │   └── math.rs             # Vec3, Quat (glam re-exports)
│   │
│   ├── client/src/
│   │   └── movement/
│   │       ├── mod.rs           # Module exports
│   │       ├── plugin.rs        # MovementConfig, ClientMovementPlugin
│   │       ├── prediction.rs    # PredictedState, predict_movement
│   │       └── correction.rs    # PositionCorrection, correct_position
│   │
│   └── server/src/
│       └── movement/
│           ├── mod.rs           # Module exports
│           ├── config.rs        # ServerMovementConfig, ViolationManager
│           ├── validation.rs    # ValidationResult, MovementValidator
│           └── update.rs        # MovementUpdateProcessor, ProcessInputResult
│
└── docs/
    ├── architecture/
    │   └── movement-system.md   # 本文档
    ├── prd/
    │   └── movement-system.md   # 产品规格书
    └── api/
        └── movement-protocol.md  # 接口文档
```

---

## 10. 未来扩展 | Future Extensions

```mermaid
mindmap
  root((移动系统))
    当前实现
      基础移动
      跳跃
      冲刺
    计划中
      游泳
      爬墙/攀爬
      载具
      飞行
    优化方向
      物理模拟升级
      碰撞检测优化
      AI 寻路
      动画同步
```

---

**文档版本**: 1.0
**最后更新**: 2026-03-02
