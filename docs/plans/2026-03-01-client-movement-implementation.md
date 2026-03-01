# 客户端输入与移动系统 - 实施计划

**设计文档**: [2026-03-01-client-movement-design.md](./2026-03-01-client-movement-design.md)
**创建日期**: 2026-03-01
**状态**: 待开始

## 实施步骤

### 第一阶段：协议扩展

- [ ] 添加 `ClientInputPacket` 到 `crates/protocol/src/client.rs`
- [ ] 添加 `ServerPositionAck` 到 `crates/protocol/src/server.rs`
- [ ] 添加 `ServerPositionCorrection` 到 `crates/protocol/src/server.rs`
- [ ] 添加 `CorrectionReason` 枚举
- [ ] 更新 `PacketId` 枚举
- [ ] 添加序列化/反序列化测试

### 第二阶段：客户端移动模块

- [ ] 创建 `crates/client/src/movement/` 目录
- [ ] 实现 `movement/mod.rs` - 模块导出
- [ ] 实现 `movement/plugin.rs` - `ClientMovementPlugin`
- [ ] 实现 `movement/prediction.rs` - 本地预测系统
  - [ ] `PredictedState` 组件
  - [ ] `InputSnapshot` 结构
  - [ ] `MovementConfig` 资源
  - [ ] `predict_movement` 系统
- [ ] 实现 `movement/correction.rs` - 服务器位置修正
  - [ ] `correct_position` 系统
  - [ ] 平滑插值逻辑
- [ ] 实现 `send_input_to_server` 系统
- [ ] 注册 `ClientMovementPlugin` 到 `app.rs`

### 第三阶段：服务器验证模块

- [ ] 创建 `crates/server/src/movement/` 目录
- [ ] 实现 `movement/mod.rs` - 模块导出
- [ ] 实现 `movement/validation.rs` - 移动验证系统
  - [ ] `ValidationResult` 枚举
  - [ ] `ServerPlayerMovement` 组件
  - [ ] `ServerMovementConfig` 资源
  - [ ] `validate_movement` 函数
- [ ] 实现 `movement/update.rs` - 位置更新处理
  - [ ] 处理客户端输入
  - [ ] 广播位置更新 (20Hz)
  - [ ] 发送修正数据包
- [ ] 注册到 `server.rs`

### 第四阶段：集成与测试

- [ ] 连接客户端和服务器
- [ ] 端到端移动测试
- [ ] 网络延迟模拟测试
- [ ] 修正逻辑验证
- [ ] 性能测试 (多玩家)

## 当前状态

**阻塞**: 无
**进行中**: 无
**已完成**: 设计文档

## 下一步

1. 开始第一阶段：协议扩展
