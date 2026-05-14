# Research Summary

**Project:** claude-win-notify
**Synthesized:** 2026-05-15
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Executive Summary

claude-win-notify 有明确的市场空白（竞品明确不支持 Windows Click-to-Focus）和可行的技术路径。**C# .NET 9 NativeAOT 是推荐技术栈**，但存在 COM 互操作风险需要 Phase 0 验证。架构应采用"按需触发 + Protocol 激活"模式，避免后台守护进程。

---

## Key Stack Decision

### Recommendation: C# .NET 9 NativeAOT

| Criterion | C# NativeAOT | Rust | Go |
|-----------|--------------|------|-----|
| Toast interactive buttons | Native WinRT via CsWinRT | 需手写 COM boilerplate | **不可能**（go-toast 不支持回调） |
| Click-to-Focus | P/Invoke 一行代码 | windows-rs 可用 | syscall 极痛苦 |
| COM activation callback | `[GeneratedComInterface]` | `#[implement]` macro | **不可能** |
| Single exe size | ~8-12 MB | ~2-4 MB | ~8-15 MB |
| Startup time | <50ms | <10ms | ~30-50ms |
| Development speed | 最快（async/await, LINQ） | 2-3x 慢（生命周期管理） | 中等 |
| Contributor 门槛 | 最低（Windows 生态主流） | 高 | 中等 |

**关键风险：** NativeAOT 明确说明 "No built-in COM"。需要 Phase 0 验证 Windows App SDK `AppNotificationManager` 或 `[GeneratedComInterface]` 在 NativeAOT 下是否可用。如果不行，备选是 Rust。

### Go 被排除

Go 在此项目中有**架构性劣势**：go-toast 库是 shell out 到 PowerShell 的，无法接收 COM 回调。这正是 claude-notifications-go 在 Windows 上做不到 Click-to-Focus 的根本原因。

---

## Feature Priorities

### Table Stakes（缺少即不可用）
- 4 种通知类型（任务完成、权限请求、提问、错误）
- Click-to-Focus（窗口级）
- 冷却时间/去重
- Hook 系统对接（stdin JSON）
- 音效提示
- 配置文件
- 一键安装

### Differentiators（核心竞争力）
- Click-to-Focus（标签页级）— WT `focus-tab` 不可用，需 UI Automation
- 交互式 Toast 按钮（Approve/Deny）— 需 COM 激活
- AI 智能摘要 — 解析 JSONL 上下文
- 通知聚合 — 多子 agent 合并
- PowerShell 一行安装 — 自动注入 hooks.json

### Anti-Features（明确不做）
- 跨平台支持
- 后台常驻 System Tray
- Web 仪表板
- 自定义 UI（WPF 弹窗）
- 插件/扩展系统

---

## Architecture Decision

### 模式：按需触发 + Protocol 激活

```
Claude Code hook → 启动 exe → 读 stdin → 分析 → 发 Toast → 退出 (<500ms)
Toast 点击 → Protocol 激活 (claude-notify://focus?...) → 新进程 → 聚焦窗口 → 退出
```

**为什么不用 COM 激活：** Protocol 激活只需一个 registry key，无需 COM 注册、后台服务、message loop。对 Click-to-Focus 足够用。

**为什么不用守护进程：** Hook 设计就是按需触发。守护进程增加复杂度、资源消耗、企业限制（startup entries）。

**交互式按钮的取舍：** Protocol 激活会启动新进程处理按钮点击。对 Approve/Deny 场景足够（不需要毫秒级响应）。

---

## Critical Pitfalls (Top 5)

| # | Pitfall | Impact | Mitigation |
|---|---------|--------|-----------|
| 1 | SetForegroundWindow 静默失败 | 核心功能不可用 | AttachThreadInput + Alt 键 hack + 多策略 fallback |
| 2 | NativeAOT "No built-in COM" | 可能迫使换 Rust | Phase 0 spike 验证 AppNotificationManager + AOT |
| 3 | Toast COM 注册三步舞 | 点击无响应 | Protocol 激活绕过 COM；或用 Windows App SDK |
| 4 | Hook shell 执行混乱 | 插件完全不工作 | 直接调 exe，设 `"shell": "powershell"` |
| 5 | WT/Warp 无外部标签 API | 只能窗口级聚焦 | UI Automation 探索；文档标注限制 |

---

## Recommended Phasing

```
Phase 0: Tech Spike (1-2 天)
├── 验证 NativeAOT + CsWinRT Toast 可用
├── 验证 SetForegroundWindow fallback chain
├── 验证 Protocol 激活可接收 Toast 点击
└── 如果 NativeAOT COM 不行 → 切换 Rust

Phase 1: MVP (2-3 周)
├── Hook stdin 接收 + JSON 解析
├── 4 种通知类型检测（基于 hook 事件类型）
├── Windows Toast 基础通知
├── Click-to-Focus 窗口级（SetForegroundWindow + fallback）
├── Protocol 激活处理 Toast 点击
├── 音效播放
├── 冷却时间 + 去重（文件锁）
├── 配置文件（JSON）
├── PowerShell 一键安装
└── AUMID 注册（品牌化通知）

Phase 2: 差异化 (3-4 周)
├── JSONL transcript 解析 + 智能通知类型判定
├── Click-to-Focus 标签页级（UI Automation for WT + Warp）
├── AI 智能摘要（模板引擎解析上下文）
├── 通知聚合（时间窗口合并）
├── 多终端检测与支持

Phase 3: 交互式体验 (2-3 周)
├── Toast 交互按钮（Approve/Deny）
├── 自定义声音
├── VS Code Terminal 支持
├── 配置系统完善

Phase 4: 生态扩展
├── 手机推送（Bark/Pushover）
├── 代码签名 + 企业部署
├── winget/scoop 分发
├── 通知历史
```

---

## Open Questions (需 Phase 0 解决)

1. NativeAOT + Windows App SDK `AppNotificationManager` 是否兼容？
2. Warp 的 Accessibility Tree 是否暴露 TabItem pattern？
3. Protocol 激活的新进程是否获得前台权限（能否 SetForegroundWindow）？
4. Claude Code stdin 在 CJK Windows 上的编码是 UTF-8 还是系统 codepage？

---

## Sources

- STACK.md: C# NativeAOT vs Rust vs Go 技术对比（Context7 验证）
- FEATURES.md: Windows Toast 功能详情（Microsoft Learn 验证）
- ARCHITECTURE.md: 按需触发 + Protocol 激活架构（Context7 + 文档验证）
- PITFALLS.md: 17 个领域特定陷阱（多源验证）
