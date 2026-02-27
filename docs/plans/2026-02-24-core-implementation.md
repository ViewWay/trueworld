# Core Crate - Error & Types Module

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现共享核心库的错误类型和基础数据类型

**Architecture**: 使用 thiserror 定义错误类型，serde 定义可序列化的数据类型，glam 提供数学类型

**Tech Stack:** Rust 2024, thiserror, serde, glam

---

## 模块概述

Core Crate 是所有其他模块的基础依赖，提供：
- 错误类型 (Error)
- 基础数据类型 (types)
- 数学工具 (math)
- 实体定义 (entity)
- 网络协议 (net)
- 常量 (constants)

---

### Task 1: 创建 Core Crate 基础结构

**Files:**
- Create: `crates/core/Cargo.toml`
- Create: `crates/core/src/lib.rs`

**Step 1: 创建 Cargo.toml**

```toml
# crates/core/Cargo.toml
[package]
name = "trueworld-core"
version.workspace = true
edition.workspace = true

[dependencies]
# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 数学
glam = { version = "0.29", features = ["serde"] }

# 网络
renet = "0.0.16"

# 时间
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.11", features = ["serde", "v4"] }

# 错误
thiserror = "2.0"
anyhow = "1.0"

# 常用
derive_more = { version = "1.0", features = ["from", "display"] }
strum = { version = "0.26", features = ["derive"] }
```

**Step 2: 创建 lib.rs 根文件**

```rust
// crates/core/src/lib.rs

pub mod error;
pub mod types;
pub mod math;
pub mod time;
pub mod net;

pub use error::{Error, Result};
```

**Step 3: 提交**

```bash
git add crates/core/
git commit -m "feat(core): add core crate base structure"
```

---

### Task 2: 实现错误类型模块

**Files:**
- Create: `crates/core/src/error.rs`

**Step 1: 编写错误定义**

```rust
// crates/core/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] renet::RenetError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AI model error: {0}")]
    AiModel(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Step 2: 添加测试**

```rust
// crates/core/src/error.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidInput("test".to_string());
        assert_eq!(err.to_string(), "Invalid input: test");
    }
}
```

**Step 3: 运行测试**

```bash
cd crates/core
cargo test
```

Expected: PASS

**Step 4: 提交**

```bash
git add crates/core/src/error.rs
git commit -m "feat(core): add error type definitions"
```

---

### Task 3: 实现基础类型模块 (types)

**Files:**
- Create: `crates/core/src/types.rs`

**Step 1: 编写类型定义**

```rust
// crates/core/src/types.rs

use serde::{Deserialize, Serialize};
use glam::{Vec2, Vec3, Quat};
use std::time::Duration;

// 类型别名
pub type PlayerId = uuid::Uuid;
pub type EntityId = u64;
pub type RoomId = uuid::Uuid;
pub type SkillId = String;
pub type ItemId = String;

pub type Position = Vec3;
pub type Rotation = Quat;
pub type Coord2 = Vec2;
pub type Coord3 = Vec3;

// 游戏时间
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTime {
    pub elapsed: Duration,
    pub tick: u64,
}

// 游戏状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Lobby,
    Loading,
    Playing,
    Paused,
    Ended,
}

// 元素类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element {
    Physical,
    Fire,
    Ice,
    Lightning,
    Earth,
    Wind,
    Light,
    Dark,
}

// 稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

// 输入动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputAction {
    None,
    Move { direction: Vec2 },
    Look { rotation: Quat },
    Attack,
    Block,
    Skill { skill_id: SkillId, target: Option<EntityId> },
    Interact { target: EntityId },
    Jump,
    Sprint(bool),
}

// 玩家输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub sequence: u32,
    pub tick: u64,
    pub actions: Vec<InputAction>,
    pub position: Vec3,
    pub rotation: Quat,
}

// 变换状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformState {
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
}

impl Default for TransformState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
        }
    }
}
```

**Step 2: 添加测试**

```rust
// crates/core/src/types.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let state = TransformState::default();
        assert_eq!(state.position, Vec3::ZERO);
    }

    #[test]
    fn test_rarity_ordering() {
        assert!(Rarity::Rare > Rarity::Common);
        assert!(Rarity::Legendary > Rarity::Epic);
    }
}
```

**Step 3: 运行测试**

```bash
cargo test
```

Expected: PASS

**Step 4: 提交**

```bash
git add crates/core/src/types.rs
git commit -m "feat(core): add basic type definitions"
```

---

### Task 4: 实现数学工具模块 (math)

**Files:**
- Create: `crates/core/src/math.rs`

**Step 1: 编写数学工具函数**

```rust
// crates/core/src/math.rs

use glam::{Vec2, Vec3, Quat};
use std::f32::consts::PI;

/// 角度转弧度
#[inline]
pub const fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

/// 弧度转角度
#[inline]
pub const fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

/// 检测点是否在扇形内
pub fn point_in_sector(
    point: Vec3,
    sector_origin: Vec3,
    sector_direction: Vec3,
    sector_angle: f32,
    sector_radius: f32,
) -> bool {
    let to_point = point - sector_origin;
    let distance = to_point.length();

    if distance > sector_radius {
        return false;
    }

    let normalized = to_point.normalize();
    let cos_angle = sector_direction.dot(normalized);
    let angle_threshold = (sector_angle * 0.5).cos();

    cos_angle >= angle_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_sector() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;

        // 在扇形内
        assert!(point_in_sector(
            Vec3::X * 5.0,
            origin,
            direction,
            90.0_f32.to_radians(),
            10.0,
        ));

        // 不在扇形内
        assert!(!point_in_sector(
            Vec3::NEG_X * 5.0,
            origin,
            direction,
            90.0_f32.to_radians(),
            10.0,
        ));
    }
}
```

**Step 2: 运行测试**

```bash
cargo test
```

Expected: PASS

**Step 3: 提交**

```bash
git add crates/core/src/math.rs
git commit -m "feat(core): add math utility functions"
```

---

### Task 5: 实现网络模块 (net)

**Files:**
- Create: `crates/core/src/net.rs`

**Step 1: 编写网络类型定义**

```rust
// crates/core/src/net.rs

use serde::{Deserialize, Serialize};

// 客户端消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Connect {
        version: String,
        auth_token: Option<String>,
    },
    PlayerInput {
        sequence: u32,
        input: super::PlayerInput,
    },
    Ping,
}

// 服务器消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    ConnectResult {
        success: bool,
        player_id: Option<super::PlayerId>,
        reason: Option<String>,
    },
    WorldUpdate {
        tick: u64,
        entities: Vec<EntityUpdate>,
    },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityUpdate {
    pub id: super::EntityId,
    pub transform: super::TransformState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialize() {
        let msg = ClientMessage::Ping;
        let serialized = serde_json::to_string(&msg).unwrap();
        assert!(serialized.contains("Ping"));
    }
}
```

**Step 2: 运行测试**

```bash
cargo test
```

Expected: PASS

**Step 3: 提交**

```bash
git add crates/core/src/net.rs
git commit -m "feat(core): add network message types"
```

---

## 检查清单

完成以上任务后：
- [ ] Core Crate 可独立编译
- [ ] 所有测试通过
- [ ] 代码格式化 (`cargo fmt`)
- [ ] 代码检查通过 (`cargo clippy`)
