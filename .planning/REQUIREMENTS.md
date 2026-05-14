# Requirements: claude-win-notify

**Defined:** 2026-05-15
**Core Value:** Click-to-Focus 在 Windows 上真正可用——通知弹出后点击即可跳转到正确的终端窗口和标签页

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Notification Core

- [ ] **NOTIF-01**: Hook 接收 Claude Code stdin JSON 并正确解析所有字段（session_id, transcript_path, cwd, hook_event_name）
- [ ] **NOTIF-02**: 检测并发送 Task Complete 通知（Stop hook + 任务完成判定）
- [ ] **NOTIF-03**: 检测并发送 Permission Request 通知（Notification hook + permission_prompt 匹配）
- [ ] **NOTIF-04**: 检测并发送 Question 通知（助手消息以问号结尾或 AskUserQuestion 工具调用）
- [ ] **NOTIF-05**: 检测并发送 Error 通知（API 错误、会话限制、异常退出）
- [ ] **NOTIF-06**: 冷却时间机制防止同一 session 短时间内重复通知（默认 5 秒）
- [ ] **NOTIF-07**: 去重机制防止相同事件多次触发（文件锁 2 秒 TTL）
- [ ] **NOTIF-08**: 用户可通过配置文件自定义通知行为（启用/禁用各类型、冷却时间、声音）

### Click-to-Focus

- [ ] **FOCUS-01**: 点击 Toast 通知后激活对应终端窗口到前台（SetForegroundWindow + fallback chain）
- [ ] **FOCUS-02**: 窗口已最小化时自动恢复后激活
- [ ] **FOCUS-03**: Windows Terminal 标签页级聚焦（通过 UI Automation 定位并切换到正确标签页）
- [ ] **FOCUS-04**: Warp 标签页级聚焦（通过 UI Automation 或可用 API 定位并切换到正确标签页）
- [ ] **FOCUS-05**: 自动检测用户使用的终端类型（Warp / Windows Terminal / 其他）
- [ ] **FOCUS-06**: Click-to-Focus 成功率 > 95%（窗口级）

### Toast Notification

- [ ] **TOAST-01**: 使用 Windows 原生 Toast 通知（WinRT API）
- [ ] **TOAST-02**: 通知显示延迟 < 500ms（从 hook 触发到 Toast 显示）
- [ ] **TOAST-03**: 每种通知类型有不同的视觉样式和声音
- [ ] **TOAST-04**: 通知显示项目名称（从 cwd 提取）作为 attribution text
- [ ] **TOAST-05**: AUMID 注册实现品牌化通知（自定义图标 + "Claude Code Notifications" 名称）

### Installation

- [ ] **INST-01**: PowerShell 一键安装（`irm https://url | iex`）完成全部配置
- [ ] **INST-02**: 安装脚本自动下载 exe 到用户目录并添加到 PATH
- [ ] **INST-03**: 安装脚本自动注入 hooks.json 配置（不覆盖已有 hooks）
- [ ] **INST-04**: 安装脚本注册 Protocol 激活 URI scheme（`claude-notify://`）
- [ ] **INST-05**: 安装脚本创建 Start Menu 快捷方式（AUMID 要求）
- [ ] **INST-06**: 单一可执行文件，零运行时依赖（NativeAOT 或 Rust 编译）
- [ ] **INST-07**: 提供卸载命令清理所有注册项（registry、快捷方式、hooks.json）
- [ ] **INST-08**: 二进制体积 < 15MB

### Tech Foundation

- [ ] **TECH-01**: 技术栈通过 Phase 0 spike 验证确定（C# NativeAOT 或 Rust）
- [ ] **TECH-02**: Protocol 激活处理 Toast 点击回调（`claude-notify://focus?session=...&pid=...`）
- [ ] **TECH-03**: 支持 Windows 10 1903+ 和 Windows 11
- [ ] **TECH-04**: 正确处理 CJK 路径和 UTF-8 编码
- [ ] **TECH-05**: 音效播放（Windows 原生 API，每种通知类型不同声音）

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Interactive Toast

- **INTERACT-01**: Permission Request 通知带 Approve/Deny 按钮（绿/红色）
- **INTERACT-02**: 点击按钮后将决策反馈给 Claude Code
- **INTERACT-03**: Question 通知带文本输入框和发送按钮

### Intelligence

- **INTEL-01**: AI 智能摘要（解析 JSONL 上下文生成有意义的通知描述）
- **INTEL-02**: JSONL transcript 深度分析（state machine 判定通知子类型）
- **INTEL-03**: 通知聚合（3 秒窗口内多条通知合并）

### Ecosystem

- **ECO-01**: 手机推送（Bark/Pushover/企业微信）
- **ECO-02**: 代码签名（EV 证书，解决 Device Guard/SmartScreen）
- **ECO-03**: winget/scoop 包管理器分发
- **ECO-04**: 通知历史记录（本地存储 + 查询）
- **ECO-05**: 自定义声音（用户自定义 WAV 文件）
- **ECO-06**: 更多终端支持（VS Code Terminal、ConEmu、PowerShell 独立窗口）

## Out of Scope

| Feature | Reason |
|---------|--------|
| 跨平台支持（macOS/Linux） | claude-notifications-go 已覆盖；专注 Windows 极致体验 |
| Web 仪表板 | 增加维护负担，核心价值不在此 |
| System Tray 常驻 | 违背按需触发架构；增加资源消耗和企业限制 |
| 自定义 UI（WPF/WinForms 弹窗） | 重造轮子，不如用原生 Toast |
| 语音播报（TTS） | 锦上添花，优先级极低 |
| Windows Widget 集成 | Win11 专属，API 不稳定 |
| Slack/Teams/Discord 集成 | 范围蔓延，用 webhook 替代 |
| 插件/扩展系统 | 过度设计，配置文件足够 |
| 自动更新机制 | 交给 winget/scoop 处理 |
| 多 Claude 实例监控 | 增加状态管理复杂度 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TECH-01 | Phase 1 | Pending |
| NOTIF-01 | Phase 2 | Pending |
| TOAST-01 | Phase 2 | Pending |
| TOAST-02 | Phase 2 | Pending |
| TECH-03 | Phase 2 | Pending |
| TECH-04 | Phase 2 | Pending |
| INST-06 | Phase 2 | Pending |
| INST-08 | Phase 2 | Pending |
| NOTIF-02 | Phase 3 | Pending |
| NOTIF-03 | Phase 3 | Pending |
| NOTIF-04 | Phase 3 | Pending |
| NOTIF-05 | Phase 3 | Pending |
| TOAST-03 | Phase 3 | Pending |
| TOAST-04 | Phase 3 | Pending |
| TECH-05 | Phase 3 | Pending |
| FOCUS-01 | Phase 4 | Pending |
| FOCUS-02 | Phase 4 | Pending |
| FOCUS-05 | Phase 4 | Pending |
| FOCUS-06 | Phase 4 | Pending |
| TECH-02 | Phase 4 | Pending |
| FOCUS-03 | Phase 5 | Pending |
| FOCUS-04 | Phase 5 | Pending |
| NOTIF-06 | Phase 6 | Pending |
| NOTIF-07 | Phase 6 | Pending |
| NOTIF-08 | Phase 7 | Pending |
| TOAST-05 | Phase 7 | Pending |
| INST-01 | Phase 8 | Pending |
| INST-02 | Phase 8 | Pending |
| INST-03 | Phase 8 | Pending |
| INST-04 | Phase 8 | Pending |
| INST-05 | Phase 8 | Pending |
| INST-07 | Phase 8 | Pending |

**Coverage:**
- v1 requirements: 32 total
- Mapped to phases: 32
- Unmapped: 0

---
*Requirements defined: 2026-05-15*
*Last updated: 2026-05-15 after roadmap creation*
