# 客户端输入与移动系统 - 实施计划

**设计文档**: [2026-03-01-client-movement-design.md](./2026-03-01-client-movement-design.md)
**创建日期**: 2026-03-01
**最后更新**: 2026-03-02 00:30
**状态**: Phase 3 已完成，准备文档化

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

### 第四阶段：集成与测试

- [ ] 连接客户端和服务器
- [ ] 端到端移动测试
- [ ] 网络延迟模拟测试
- [ ] 修正逻辑验证
- [ ] 性能测试 (多玩家)

## 当前状态

**阻塞**: 无
**进行中**: 文档化 (产品规格书、接口设计)
**已完成**:
- ✅ Phase 1: 协议扩展 (2026-03-01 22:00-23:00)
- ✅ Phase 2: 客户端移动模块 (2026-03-01 23:00-23:26)
- ✅ Phase 3: 服务器验证模块 (2026-03-02 00:00-00:30)

## 下一步

1. 产品规格说明书 (PRD)
   - 移动系统需求定义
   - 性能指标
   - 安全要求

2. 接口设计文档
   - 协议接口定义
   - API 接口说明
   - 数据结构定义

3. Phase 4: 集成与测试
   - 连接客户端和服务器
   - 端到端移动测试
   - 网络延迟模拟测试
   - 修正逻辑验证
   - 性能测试 (多玩家)
