# 客户端输入与移动系统 - 实施计划

**设计文档**: [2026-03-01-client-movement-design.md](./2026-03-01-client-movement-design.md)
**创建日期**: 2026-03-01
**最后更新**: 2026-03-03 07:45
**状态**: Phase 4 网络集成完成，待端到端测试

## 实施步骤

### 第一阶段：协议扩展 ✅ 已完成

- [x] 添加 `ClientInputPacket` 到 `crates/protocol/src/client.rs`
- [x] 添加 `ServerPositionAck` 到 `crates/protocol/src/server.rs`
- [x] 添加 `ServerPositionCorrection` 到 `crates/protocol/src/server.rs`
- [x] 添加 `CorrectionReason` 枚举
- [x] 更新 `PacketId` 枚举
- [x] 添加序列化/反序列化测试

### 第二阶段：客户端移动模块 ✅ 已完成

- [x] 创建 `crates/client/src/movement/` 目录
- [x] 实现 `movement/mod.rs` - 模块导出
- [x] 实现 `movement/plugin.rs` - `ClientMovementPlugin`
- [x] 实现 `movement/prediction.rs` - 本地预测系统
  - [x] `PredictedState` 组件
  - [x] `InputSnapshot` 结构
  - [x] `MovementConfig` 资源
  - [x] `predict_movement` 系统
- [x] 实现 `movement/correction.rs` - 服务器位置修正
  - [x] `correct_position` 系统
  - [x] 平滑插值逻辑
- [x] 实现 `send_input_to_server` 系统
- [x] 注册 `ClientMovementPlugin` 到 `app.rs`

### 第三阶段：服务器验证模块 ✅ 已完成

- [x] 创建 `crates/server/src/movement/` 目录
- [x] 实现 `movement/mod.rs` - 模块导出
- [x] 实现 `movement/config.rs` - 配置资源
  - [x] `ServerMovementConfig` 资源
  - [x] `PlayerViolationTracker` 违规追踪
  - [x] `ViolationManager` 违规管理器
- [x] 实现 `movement/validation.rs` - 移动验证系统
  - [x] `ValidationResult` 枚举
  - [x] `ServerPlayerMovement` 组件
  - [x] `MovementValidator` 验证器
- [x] 实现 `movement/update.rs` - 位置更新处理
  - [x] `MovementUpdateProcessor` 处理器
  - [x] `ProcessInputResult` 结果类型
  - [x] 处理客户端输入
  - [x] 发送位置确认包
  - [x] 发送修正数据包
- [x] 注册到 `main.rs`
- [x] 单元测试通过 (29 tests)

### 第四阶段：集成与测试 🔄 进行中

- [x] 协议层扩展：添加 ClientInputPacket 到 ClientMessage 枚举
- [x] 服务器添加 ClientInputPacket 处理
- [x] 客户端网络集成：发送 ClientInputPacket
- [x] 服务器集成：使用 MovementUpdateProcessor
- [x] 添加 ServerPositionAck 和 ServerPositionCorrection 到 ServerMessage
- [x] 客户端接收位置确认和修正事件
- [ ] 连接客户端和服务器
- [ ] 端到端移动测试
- [ ] 网络延迟模拟测试
- [ ] 修正逻辑验证
- [ ] 性能测试 (多玩家)

## 当前状态

**阻塞**: 无
**进行中**: Phase 4 - 端到端测试
**已完成**:
- ✅ Phase 1: 协议扩展 (2026-03-01 22:00-23:00)
- ✅ Phase 2: 客户端移动模块 (2026-03-01 23:00-23:26)
- ✅ Phase 3: 服务器验证模块 (2026-03-02 00:00-00:30)
- ✅ 文档化完成：架构文档、PRD、接口文档 (2026-03-02 00:30-01:00)
- ✅ Phase 4 网络集成 (2026-03-03 07:30-07:45):
  - ✅ 服务器集成 MovementUpdateProcessor
  - ✅ 客户端发送 ClientInputPacket
  - ✅ 位置确认和修正消息处理

## 下一步

1. 端到端测试
   - 在服务器主循环中创建 MovementUpdateProcessor
   - 处理 ClientInputPacket 并发送响应

2. 客户端网络集成
   - 修改网络模块发送 ClientInputPacket
   - 接收 ServerPositionAck 和 ServerPositionCorrection

3. 端到端测试
   - 连接测试
   - 移动测试
   - 延迟模拟测试

4. 性能测试
   - 多玩家压力测试
   - 带宽使用分析
