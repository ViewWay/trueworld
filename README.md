# TrueWorld ⚔️

> 使用 Rust 和现代 AI 技术开发的类似 SAO 的 VR MMO RPG 游戏

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg) [![Bevy](https://img.shields.io/badge/Bevy-0.15-purple.svg) [![License](https://img.shields.io/badge/License-MIT%2FApache--blue.svg)

## 🎮 项目简介

TrueWorld 是一款创新性的 MMORPG 游戏，通过摄像头和麦克风实现体感操作，让不擅长键盘操作的玩家也能完成高难度游戏操作。

### 核心特性

-   **🎯 多模态输入融合**: 键盘/鼠标 + 摄势识别 + 语音命令，灵活组合触发技能
-   **🤖 AI 增强识别**: 使用 Whisper (语音识别)、Candle (LLM) 和 MediaPipe (姿态识别) 提高准确度
-   **⚔️ SAO 风格战斗**: 剑技系统、连携、等级、装备等经典 MMORPG 要素
-   **🌍 开放世界**: 探索、战斗、社交、成长
-   **🎤 语音交互**: 语音命令控制角色 + 实时语音聊天
-   **👀 动作捕捉**: 通过摄像头识别玩家动作转化为游戏技能

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      TrueWorld Architecture                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    Game Layer                          │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────────────────┐ │   │
│  │  │  Client │  │  Server │  │     Services        │ │   │
│  │  │ (Bevy)  │  │  Renet  │  │ Signaling/Matchmaker│ │   │
│  │  └─────────┘  └─────────┘  └─────────────────────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
│                         ▲                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    AI Layer                            │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌───────┐ │   │
│  │  │Percept  │  │   ASR   │  │   NLP   │  │Action │ │   │
│  │  │(Camera) │  │(Voice)  │  │(LLM)   │  │(Motion)│ │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └───────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    Core Layer                          │   │
│  │  • Protocol  •  Types  •  Math  •  Network             │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## 📦 项目结构

```
trueworld/
├── crates/              # Rust Crates
│   ├── core/            # 共享核心库
│   ├── protocol/        # 网络协议
│   ├── client/          # 游戏客户端
│   ├── server/          # 游戏服务器
│   ├── ai/              # AI 能力
│   ├── bevy/            # Bevy 插件
│   └── net/             # 网络层
├── services/            # 微服务
│   ├── signaling/       # 信令服务
│   ├── matchmaker/      # 匹配服务
│   └── ai-inference/    # AI 推理服务
├── assets/              # 游戏资源
│   ├── models/          # 3D 模型
│   ├── audio/           # 音频
│   ├── textures/        # 纹理
│   └── config/          # 配置文件
├── .tasks/              # 实施计划文档
├── .mem/                # 架构分析文档
└── docs/                # 项目文档
```

## 🚀 快速开始

### 环境要求

-   Rust 1.80+
-   Blender 4.0+ (建模)
-   Audacity (音频编辑)

### 构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/trueworld.git
cd trueworld

# 构建所有 crates
cargo build --workspace

# 运行客户端
cargo run --bin trueworld

# 运行服务器
cargo run --bin trueworld-server
```

### 运行服务

```bash
# 信令服务
cd services/signaling
cargo run

# 匹配服务
cd services/matchmaker
cargo run

# AI 推理服务
cd services/ai-inference
cargo run
```

## 📚 文档

-   [项目总计划](.tasks/project-master-plan.md)
-   [架构分析](.mem/)
-   [实施计划](.tasks/README.md)
-   [API 文档](docs/api/)
-   [贡献指南](CONTRIBUTING.md)

## 🛠️ 开发

### 当前状态

模块

状态

Core Crate

✅ 设计完成

Protocol Crate

✅ 设计完成

Client Crate

✅ 设计完成

Server Crate

✅ 设计完成

AI Crates

✅ 设计完成

Bevy Plugins

✅ 设计完成

Network Layer

✅ 设计完成

Services

✅ 设计完成

Assets

✅ 设计完成

### 贡献指南

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 代码规范

-   使用 `rustfmt` 格式化代码
-   使用 `clippy` 检查代码质量
-   遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
-   所有公共 API 必须有文档注释

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

-   [Bevy](https://bevyengine.org/) - 游戏引擎
-   [Renet](https://github.com/lucaspoffin/renet) - 网络库
-   [Candle](https://github.com/huggingface/candle) - ML 框架
-   [Whisper](https://github.com/openai/whisper) - 语音识别
-   [MediaPipe](https://google.github.io/mediapipe/) - 姿态检测

---

**注意**: 这是一个长期开发项目，预计需要 6-12 个月完成基本功能。