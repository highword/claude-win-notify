# Claude Win Notification — 项目研究报告

> 研究日期: 2026-05-14
> 目标: 构建一个 **Windows-First** 的 Claude Code 通知插件，全面超越 claude-notifications-go

---

## 一、市场现状与竞品分析

### 1.1 竞品全景

| 项目 | Stars | 语言 | 平台 | 核心差异化 |
|------|-------|------|------|-----------|
| claude-notifications-go | 617 | Go | macOS/Linux/Windows | Click-to-Focus, 企业 webhook, 跨平台 |
| zellij-attention | 27 | Rust/WASM | Linux | Zellij 原生标签指示器 |
| agent-workflow | 19 | Shell | macOS | Ghostty 终端 + 权限自动处理 |
| claude-shadow | 8 | PowerShell | Windows/Android | 手机推送 + 远程审批 |
| ClaudePulse | 4 | PowerShell | Windows | 轻量 Toast, 中文文档 |
| claude-code-windows-toast-focus | 3 | PowerShell | Windows | Toast + 终端聚焦 |
| claude-code-bark-notify | 3 | - | macOS/Windows | iPhone Bark 推送 |
| cc-paw | 6 | TypeScript | 跨平台 | Web UI 管理会话和配置 |

### 1.2 市场空白

1. **Windows Click-to-Focus** — 无人真正实现（窗口级 + 标签页级）
2. **交互式通知** — 无人支持从通知中直接操作（审批/回复/执行）
3. **AI 智能摘要** — 无人利用 JSONL 日志上下文生成有意义的通知文本
4. **通知聚合与优先级** — 多子 agent 并发时无人做智能合并
5. **远程/SSH 通知转发** — 无解决方案
6. **Web 仪表板** — 无统一的通知历史和实时状态面板
7. **企业 Windows 友好** — 无代码签名、无 Device Guard 适配

---

## 二、标杆项目深度拆解 (claude-notifications-go)

### 2.1 架构概览

```
┌─────────────────────────────────────────────────────┐
│                Claude Code Process                    │
│  hooks.json 注册 5 种事件:                           │
│  PreToolUse | Notification | Stop | SubagentStop     │
│  TeammateIdle                                        │
└──────────────────┬──────────────────────────────────┘
                   │ stdin JSON (session_id, transcript_path)
                   ▼
┌─────────────────────────────────────────────────────┐
│            hook-wrapper.sh / .bat                     │
│  找到平台二进制 → 调用 handle-hook <EventType>       │
└──────────────────┬──────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────┐
│           Go Binary (主逻辑)                         │
│                                                      │
│  1. 读取 stdin JSON                                  │
│  2. 解析 JSONL transcript (pkg/jsonl/)               │
│  3. 状态机判定通知类型 (internal/analyzer/)           │
│  4. 去重检查 (internal/dedup/)                       │
│  5. 冷却时间检查 (internal/state/)                   │
│  6. 分发通知:                                        │
│     ├── 桌面通知 (internal/notifier/)                │
│     ├── 音频播放 (internal/audio/)                   │
│     └── Webhook (internal/webhook/)                  │
└─────────────────────────────────────────────────────┘
```

### 2.2 六种通知类型的检测逻辑

| 类型 | 触发条件 | 检测方式 |
|------|----------|----------|
| Task Complete | Stop hook + 日志含 Write/Edit/Bash 工具调用 | 状态机分析 JSONL |
| Review Complete | Stop hook + 仅 Read/Grep/Glob + 长文本响应 >200字 | 状态机分析 JSONL |
| Question | PreToolUse(AskUserQuestion) 或 Notification(permission_prompt) | Hook matcher |
| Plan Ready | PreToolUse(ExitPlanMode) | Hook matcher |
| Session Limit | 最后3条assistant消息含 "Session limit reached" | 文本匹配 |
| API Error | JSONL 中 isApiErrorMessage=true + error 字段 | 字段检查 |

### 2.3 Windows 上的实际表现

**能用的：**
- go-toast 库发送 Windows 10+ Toast 通知
- Windows 原生 API 音频播放
- 基本 hook 触发（v1.39.0 修复后改用 PowerShell 直接调用 exe）

**不能用的：**
- ❌ Click-to-Focus（明确标注不支持）
- ❌ 安装需要 Git Bash
- ❌ Device Guard 阻止未签名 exe（企业环境）
- ❌ 历史 hook 执行 bug 频繁（#55, #79）
- ❌ 系统声音不可用

### 2.4 用户痛点（从 Issues 提取）

| Issue # | 问题 | 严重度 |
|---------|------|--------|
| #36 | Windows click-to-focus 功能请求（有详细提案） | HIGH |
| #69 | Enterprise Device Guard 阻止二进制 | MEDIUM |
| #70 | 安装时校验和不匹配 | HIGH |
| #82 | macOS 通知偷焦点 | HIGH |
| #72 | Ghostty 只能窗口级，不能标签页级聚焦 | MEDIUM |
| #67 | 最小化窗口 AXRaise 无效 | MEDIUM |
| #55 | Windows hook 双重 sh 执行 | HIGH |
| #79 | Stop hook "no stderr output" 失败 | HIGH |

### 2.5 优势总结

1. 零运行时依赖（Go 单一二进制）
2. 一行安装
3. JSONL 状态机智能分析
4. 15+ 终端 Click-to-Focus（macOS/Linux）
5. 企业级 webhook（熔断/重试/限流）
6. 活跃维护（39+ 版本）
7. 多路复用器支持（tmux/zellij/WezTerm/kitty）

### 2.6 劣势总结

1. Windows 是最薄弱平台
2. 无手机推送
3. 无交互式通知（不能从通知中操作）
4. 无通知历史/仪表板
5. GPL-3.0 许可证（企业不友好）
6. 安装在 Windows 上依赖 Git Bash
7. 通知轰炸（多子 agent 无聚合）
8. 无 AI 智能摘要

---

## 三、技术可行性研究

### 3.1 Windows Click-to-Focus 技术方案

#### 方案 A: Win32 API (推荐)

```
FindWindow / EnumWindows → 找到终端进程窗口
GetWindowThreadProcessId → 确认进程
SetForegroundWindow → 激活窗口
```

**难点：** Windows 限制非前台进程调用 `SetForegroundWindow`。
**解法：**
1. `AllowSetForegroundWindow` + `AttachThreadInput` 绕过限制
2. 模拟 Alt 键按下后再 `SetForegroundWindow`（经典 hack）
3. 使用 `keybd_event(VK_MENU)` 触发前台权限

#### 方案 B: UI Automation API

```csharp
// 通过 UI Automation 查找特定标签页
AutomationElement terminal = AutomationElement.RootElement
    .FindFirst(TreeScope.Children, new PropertyCondition(
        AutomationElement.NameProperty, "Claude Code"));
terminal.SetFocus();
```

适用于 Windows Terminal 标签页级别的聚焦。

#### 方案 C: Windows Terminal 专用

Windows Terminal 支持命令行参数：
```powershell
wt --window 0 focus-tab --index 2
```

也可以通过 `SendKeys` 或 `WindowsTerminal.exe` 的 COM 接口操作。

#### 推荐组合

| 终端 | 方案 |
|------|------|
| Windows Terminal | `wt focus-tab` 命令 |
| VS Code Terminal | VS Code CLI (`code --goto`) 或 UI Automation |
| PowerShell 独立窗口 | `SetForegroundWindow` |
| ConEmu/Cmder | `ConEmuC -GuiMacro` |

### 3.2 Windows Toast 交互式通知

Windows 10+ Toast 通知支持：
- **按钮操作**: 最多 5 个自定义按钮
- **文本输入框**: 用户可直接在通知中输入文字
- **下拉菜单**: 选择预设选项
- **进度条**: 实时显示任务进度

```xml
<toast activationType="background" launch="action=approve">
  <visual>
    <binding template="ToastGeneric">
      <text>Claude Code 请求权限</text>
      <text>执行: rm -rf node_modules</text>
    </binding>
  </visual>
  <actions>
    <action content="批准" arguments="action=approve" activationType="background"/>
    <action content="拒绝" arguments="action=deny" activationType="background"/>
    <action content="查看详情" arguments="action=view" activationType="foreground"/>
  </actions>
</toast>
```

**实现方式:**
1. **PowerShell + BurntToast** — 最简单，但回调能力有限
2. **C#/.NET WinRT** — 完整功能，支持 COM 激活回调
3. **Rust + windows-rs** — 高性能，零依赖，支持 COM 激活
4. **Node.js + node-notifier / electron** — 跨平台但重

**推荐: Rust + windows-rs**
- 编译为单一 exe，无运行时依赖
- 原生 Windows API 访问
- 支持 COM 服务注册（处理按钮回调）
- 体积小（~2MB）

### 3.3 JSONL Transcript 解析

Claude Code 的 JSONL 日志格式：
```json
{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","name":"Write",...}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Done!"}]}}
{"type":"result","result":{"type":"success"}}
```

关键字段：
- `type`: assistant / user / result / system
- `message.content[].type`: text / tool_use / tool_result
- `message.content[].name`: 工具名（Write/Edit/Bash/Read/Grep/Glob...）
- `isApiErrorMessage`: API 错误标志
- `sessionId`: 会话 ID

### 3.4 Claude Code Hook 系统

Hook JSON 格式（hooks.json）：
```json
{
  "hooks": {
    "Stop": [{ "hooks": [{ "type": "command", "command": "..." }] }],
    "PreToolUse": [{ "matcher": "pattern", "hooks": [...] }],
    "Notification": [{ "matcher": "pattern", "hooks": [...] }],
    "SubagentStop": [{ "hooks": [...] }],
    "TeammateIdle": [{ "hooks": [...] }]
  }
}
```

Hook 接收的 stdin JSON：
```json
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.jsonl",
  "tool_name": "AskUserQuestion",  // PreToolUse only
  "hook_event_name": "Stop"
}
```

### 3.5 技术栈选择分析

| 方案 | 优势 | 劣势 |
|------|------|------|
| **Rust** | 性能极致, 单一 exe, windows-rs 原生API, 内存安全 | 开发周期长, 学习曲线 |
| **Go** | 快速开发, 跨平台, 竞品验证可行 | Windows API 需 CGo, Toast 功能受限 |
| **C# (.NET AOT)** | WinRT 原生支持, Toast 功能完整, NativeAOT 无依赖 | 仅 Windows, 编译体积较大 |
| **PowerShell** | 零编译, 用户可修改, BurntToast 生态 | 性能差, 启动慢, 功能受限 |
| **TypeScript/Bun** | 开发快, 生态好, 跨平台 | 需要运行时, 体积大 |

**推荐: C# .NET 8 NativeAOT**

理由：
1. WinRT Toast API 原生支持（完整的交互式通知）
2. NativeAOT 编译为单一 exe，无需 .NET 运行时
3. UI Automation API 原生支持（Click-to-Focus 标签页级）
4. Windows Terminal 集成最方便
5. 开发效率远高于 Rust，Windows API 覆盖完整
6. 可以做代码签名（解决 Device Guard 问题）
7. COM 激活回调简单实现（处理通知按钮）

---

## 四、产品定位与差异化策略

### 4.1 项目定位

**"Windows 上最好的 Claude Code 通知体验"**

不追求跨平台（那是 claude-notifications-go 的优势），专注把 Windows 体验做到极致。

### 4.2 核心差异化功能

#### 第一梯队：必须有（超越竞品的核心）

| 功能 | 描述 | 竞品状态 |
|------|------|----------|
| **Windows Click-to-Focus** | 窗口级 + 标签页级聚焦 | ❌ 无人实现 |
| **交互式通知** | 从通知中 Approve/Deny/Reply | ❌ 无人实现 |
| **AI 智能摘要** | 解析 JSONL 生成有意义的通知文本 | ❌ 无人实现 |
| **零依赖安装** | 纯 PowerShell 一键安装，exe 自包含 | ⚠️ 竞品需 Git Bash |
| **企业友好** | 代码签名 + Intune/GPO 兼容 | ❌ 竞品被 Device Guard 拦 |

#### 第二梯队：有很大加分

| 功能 | 描述 |
|------|------|
| **通知聚合** | 多子 agent 并发时合并通知，设优先级 |
| **手机推送** | Bark / Pushover / 企业微信 / 钉钉 |
| **通知历史** | 本地 SQLite 存储，可查看、搜索、统计 |
| **进度条通知** | 长任务实时进度更新（Toast ProgressBar） |
| **自定义声音** | 内置 + 自定义音效，支持随机播放 |

#### 第三梯队：锦上添花

| 功能 | 描述 |
|------|------|
| **Web 仪表板** | 本地 HTTP 服务，浏览器查看实时状态 |
| **远程转发** | SSH 场景下转发通知到本地 |
| **System Tray 常驻** | 托盘图标显示状态，右键菜单快捷操作 |
| **Windows Widget** | Win11 Widget 面板集成 |
| **语音播报** | Windows SAPI TTS 朗读通知 |

### 4.3 用户故事

1. **任务完成提醒**: Claude Code 完成编码任务 → Toast 弹出"已完成: 重构认证模块（修改 5 个文件）" → 点击通知 → 自动切到 Windows Terminal 的 Claude 标签页

2. **权限审批**: Claude 需要执行 `rm -rf dist/` → Toast 弹出带 [批准] [拒绝] 按钮 → 用户直接在通知中点批准 → Claude 继续执行

3. **问题回答**: Claude 提问"用 JWT 还是 Session?" → Toast 弹出带选项按钮 → 用户点选 → 回答自动送回

4. **多 agent 聚合**: 3 个子 agent 在 2 秒内完成 → 合并为一条通知 "3 个子任务已完成" 而非轰炸 3 条

5. **手机推送**: 用户去泡咖啡 → Claude 完成后推送到手机 → 看到"Feature X 已完成，等待你的验证"

---

## 五、技术架构设计（初步）

### 5.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                  Claude Code Process                      │
│  hooks.json → 5 种事件                                   │
└──────────────────┬──────────────────────────────────────┘
                   │ stdin JSON
                   ▼
┌─────────────────────────────────────────────────────────┐
│         claude-win-notify.exe (C# NativeAOT)             │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
│  │ Hook     │  │ Analyzer │  │ Notifier  │             │
│  │ Receiver │→ │ (JSONL   │→ │ Dispatcher│             │
│  │          │  │  Parser) │  │           │             │
│  └──────────┘  └──────────┘  └─────┬─────┘             │
│                                     │                    │
│                    ┌────────────────┼────────────────┐   │
│                    ▼                ▼                ▼   │
│              ┌──────────┐  ┌────────────┐  ┌────────┐  │
│              │ Toast    │  │ Click-to-  │  │ Push   │  │
│              │ (WinRT)  │  │ Focus      │  │ (HTTP) │  │
│              │ 交互按钮  │  │ Win32 API  │  │ Bark   │  │
│              │ 进度条    │  │ UI Auto    │  │ 企微   │  │
│              │ AI 摘要   │  │ WT CLI     │  │ 钉钉   │  │
│              └──────────┘  └────────────┘  └────────┘  │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
│  │ Audio    │  │ History  │  │ Aggregator│             │
│  │ Player   │  │ (SQLite) │  │ (批量合并) │             │
│  └──────────┘  └──────────┘  └───────────┘             │
└─────────────────────────────────────────────────────────┘
                   │
                   ▼ (可选)
┌─────────────────────────────────────────────────────────┐
│         System Tray Service (可选后台驻留)                │
│  - 托盘图标 + 状态                                       │
│  - COM 激活服务（处理 Toast 按钮回调）                    │
│  - 本地 HTTP API（Web 仪表板）                           │
└─────────────────────────────────────────────────────────┘
```

### 5.2 模块划分

| 模块 | 职责 | 技术 |
|------|------|------|
| HookReceiver | 读取 stdin JSON, 路由事件 | System.Text.Json |
| TranscriptAnalyzer | 解析 JSONL, 状态机判定通知类型 | 流式解析器 |
| SmartSummary | AI 辅助生成通知摘要文本 | 模板 + 规则引擎 |
| ToastNotifier | 发送 Windows Toast (含交互) | Microsoft.Windows.SDK.NET |
| ClickToFocus | 窗口/标签页聚焦 | Win32 API + UI Automation + WT CLI |
| AudioPlayer | 播放通知音效 | NAudio 或 Windows.Media |
| PushService | 手机推送（Bark/Pushover/企微） | HttpClient |
| NotificationHistory | 存储通知历史 | SQLite (Microsoft.Data.Sqlite) |
| Aggregator | 通知合并与优先级 | 时间窗口 + 计数器 |
| ConfigManager | 用户配置读写 | JSON 配置文件 |
| TrayService | 系统托盘常驻（可选） | WinForms NotifyIcon |

### 5.3 配置文件设计（初步）

```jsonc
{
  "version": 1,
  "notifications": {
    "desktop": {
      "enabled": true,
      "clickToFocus": true,
      "interactive": true,      // 启用按钮交互
      "smartSummary": true,     // AI 智能摘要
      "sound": "default",
      "volume": 0.8
    },
    "push": {
      "enabled": false,
      "provider": "bark",       // bark | pushover | wechat | dingtalk
      "config": {
        "url": "https://api.day.app/YOUR_KEY"
      }
    },
    "aggregation": {
      "enabled": true,
      "windowMs": 3000,         // 3 秒内合并
      "maxBatch": 5
    },
    "history": {
      "enabled": true,
      "retentionDays": 30
    }
  },
  "statuses": {
    "taskComplete": { "enabled": true, "sound": "complete.wav", "push": true },
    "reviewComplete": { "enabled": true, "sound": "review.wav", "push": false },
    "question": { "enabled": true, "sound": "question.wav", "push": true, "interactive": true },
    "planReady": { "enabled": true, "sound": "plan.wav", "push": false },
    "sessionLimit": { "enabled": true, "sound": "warning.wav", "push": true },
    "apiError": { "enabled": true, "sound": "error.wav", "push": true },
    "permissionRequest": { "enabled": true, "sound": "permission.wav", "interactive": true }
  },
  "clickToFocus": {
    "terminal": "auto",         // auto | windows-terminal | vscode | conemu | pwsh
    "tabDetection": true        // 尝试定位具体标签页
  },
  "suppress": {
    "cooldownSeconds": 5,
    "filters": []
  }
}
```

---

## 六、项目命名与品牌

### 6.1 候选名称

| 名称 | 含义 | 域名/仓库可用性 |
|------|------|----------------|
| **claude-win-notify** | 直接明了 | ✓ |
| **ClaudePing** | 简洁有趣 | 需检查 |
| **Claude Beacon** | 灯塔/信标 | 需检查 |
| **Claude Pulse** | 脉搏/心跳（已被 ClaudePulse 占） | ❌ |
| **WinClaude** | Windows + Claude | 需检查 |
| **Claude Alert** | 直接 | 需检查 |

### 6.2 项目口号

- "The Windows notification experience Claude Code deserves."
- "Click. Focus. Done."
- "Never miss a Claude moment on Windows."

---

## 七、开发路线图建议

### Phase 1: MVP (核心通知)
- Hook 接收 + JSONL 解析 + 通知类型判定
- Windows Toast 基础通知（标题 + 内容 + 图标）
- Click-to-Focus（窗口级）
- 音效播放
- PowerShell 一键安装脚本
- Claude Code 插件注册

### Phase 2: 交互式体验
- Toast 按钮交互（Approve/Deny）
- Click-to-Focus 标签页级（Windows Terminal）
- AI 智能摘要
- 通知聚合
- 配置系统

### Phase 3: 扩展生态
- 手机推送（Bark/Pushover）
- 通知历史 + 搜索
- System Tray 常驻
- 自定义声音
- 企业部署包（MSI + 代码签名）

### Phase 4: 高级功能
- Web 仪表板（本地 HTTP）
- 远程通知转发
- Windows Widget 集成
- 语音播报（TTS）
- 团队模式通知路由

---

## 八、风险与挑战

| 风险 | 影响 | 缓解策略 |
|------|------|----------|
| SetForegroundWindow 前台限制 | Click-to-Focus 失败 | Alt 键 hack + AttachThreadInput |
| Toast COM 激活复杂 | 交互按钮回调不工作 | 使用 Windows Community Toolkit |
| NativeAOT 兼容性 | 某些 API 不支持 trim | 测试矩阵覆盖 |
| Claude Code hook 格式变更 | 插件失效 | 版本检测 + 优雅降级 |
| Device Guard / SmartScreen | exe 被拦截 | 代码签名证书 |
| JSONL 格式未文档化 | 解析逻辑脆弱 | 参考 claude-notifications-go 实现 + 容错 |
| 用户 Windows 版本碎片化 | Win10/11 API 差异 | 功能检测 + 降级 |

---

## 九、许可证建议

**MIT License**

理由：
1. 对企业友好（竞品用 GPL-3.0 是劣势）
2. 鼓励 fork 和社区贡献
3. 不限制商业使用
4. 是 Claude Code 插件生态的主流选择

---

## 十、结论

### 核心竞争优势

1. **Windows-First** — 不是"也支持 Windows"，而是"为 Windows 而生"
2. **交互式通知** — 从通知中直接操作，省去切换窗口
3. **Click-to-Focus 真正可用** — 窗口级 + 标签页级，竞品做不到
4. **AI 智能摘要** — 不是干巴巴的 "Task Complete"，而是有意义的描述
5. **企业就绪** — 代码签名、零依赖、Intune 兼容

### 技术栈决策

**C# .NET 8 NativeAOT** 是最优选择 — 兼顾开发效率、Windows API 完整性、和零依赖部署。

### 成功标准

- 安装时间 < 30 秒（PowerShell 一行命令）
- Click-to-Focus 成功率 > 95%（Windows Terminal + VS Code）
- Toast 显示延迟 < 500ms
- 二进制体积 < 15MB
- 支持 Windows 10 1903+ 和 Windows 11

---

## 十一、深度前景与计划分析

> 分析日期: 2026-05-14
> 基于 RESEARCH 初版 + 最新市场数据补充

### 11.1 市场时机判断：现在正是最佳窗口

#### Claude Code 生态爆发中

| 指标 | 数据 | 来源 |
|------|------|------|
| Claude Code GitHub Stars | 119K+ | GitHub (2026.04) |
| Anthropic 年化收入 | $140 亿 | 多方报道 (2026.02) |
| Claude Code 单独年化收入 | $25 亿 | shahidshahmiri.com |
| Claude 月活用户 | 1890 万（Web）| DemandSage 2026 |
| Fortune 100 企业采用率 | >70% | 多方报道 |
| Anthropic 估值 | $3800 亿 | 2026.02 Series G |

#### 竞争格局对本项目有利

| 竞品/事件 | 状态 | 对我们的意义 |
|-----------|------|-------------|
| claude-notifications-go Windows Issues | #55, #69, #73, #79 全是 Windows 问题 | 用户痛点明确存在 |
| VS Code `Claude Notifications` 插件 | 142 安装量（2026.02 发布） | 证明需求存在，但仅限 VS Code |
| JetBrains `Claude Code Notifications` | 2026.02 上线 | IDE 生态在动，终端生态空白 |
| Reddit 社区态度 | "no I will not support windows" | 市场被竞品主动放弃 |
| Hook 最佳实践 | 已确立 Node.js `.mjs` 跨平台方案 | 生态成熟，降低开发门槛 |

**结论：Windows 用户被忽视，需求真实存在，竞品主动不做。时机窗口约 3-6 个月。**

### 11.2 SWOT 分析

| 维度 | 分析 |
|------|------|
| **Strengths** | Windows-First 无直接竞品；C# NativeAOT 技术选型正确（WinRT/COM/UIAutomation 原生支持）；MIT 许可证企业友好；开发者本身在企业环境使用（dogfooding） |
| **Weaknesses** | 单人项目；需要代码签名证书成本；NativeAOT 有 trim 兼容性风险；交互式通知需要 COM 后台服务（增加复杂度） |
| **Opportunities** | Claude Code 用户基数 10x 增长中；企业 Windows 用户是最大未满足群体；可被 Anthropic 官方推荐；中文社区缺乏此类工具 |
| **Threats** | Anthropic 可能内建通知；`claude-notifications-go` 可能修好 Windows；VS Code 插件可能扩展到终端；Windows 11 通知 API 可能变更 |

### 11.3 核心风险的现实性评估

| 风险 | 概率 | 现实评估 | 缓解策略 |
|------|------|----------|----------|
| **Anthropic 内建通知** | 低 | Claude Code 119K 星，hook 系统刻意设计为插件化。内建意味着维护所有平台通知逻辑，不符合"平台化"策略 | 保持 hook 接口兼容，随时适配 |
| **竞品修好 Windows** | 中 | Go 的 Windows API 调用需 CGo，交互式 Toast 和 UI Automation 在 Go 中极痛苦。架构性劣势不易弥补 | 速度优先，抢占先发 |
| **SetForegroundWindow 限制** | 低 | 已有成熟绕过方案。`keybd_event(VK_MENU)` 是经典已验证 hack | Phase 0 即验证 |
| **NativeAOT + WinRT** | 低 | .NET 8+ 对 WinRT 支持已成熟。`CsWinRT` 项目积极维护 | 测试矩阵覆盖 Win10/11 |
| **COM 激活回调** | 高 | 最大技术风险。需注册 COM 服务才能处理 Toast 按钮点击 | Phase 1 先跳过交互按钮，Phase 2 再引入 |

### 11.4 修订版开发路线图

```
Phase 0: Proof of Concept (1-2 天)
├── PowerShell 原型验证 hook → Toast 链路
├── 验证 SetForegroundWindow hack 可行性
└── 验证 Windows Terminal `wt focus-tab` 方案

Phase 1: MVP Release (2-3 周) ← 关键里程碑
├── C# NativeAOT 项目脚手架
├── Hook stdin 接收 + JSON 解析
├── JSONL transcript 解析 + 6 种通知类型检测
├── Windows Toast 基础通知（标题+摘要，无交互按钮）
├── Click-to-Focus 窗口级（SetForegroundWindow）
├── Click-to-Focus 标签页级（wt focus-tab）
├── 音效播放（Windows.Media.Playback）
├── 冷却时间 + 去重
├── PowerShell 一键安装脚本（含 hooks.json 注入）
└── README + 动图演示

Phase 2: 差异化体验 (3-4 周)
├── Toast 交互按钮（Approve/Deny）+ COM 激活服务
├── AI 智能摘要（模板引擎，不需要调 LLM API）
├── 通知聚合（时间窗口合并）
├── VS Code Terminal Click-to-Focus
├── 配置文件系统（JSON）
├── 自定义声音
└── 去重优化（基于 session_id + event hash）

Phase 3: 生态扩展 (4-6 周)
├── 手机推送（Bark 优先，最简单）
├── 通知历史（SQLite）
├── System Tray 常驻模式
├── 企业部署（MSI 打包 + 代码签名）
├── winget / scoop 分发
└── GitHub Actions CI/CD（自动构建 + Release）

Phase 4: 高级功能 (按需)
├── Web 仪表板（本地 Kestrel）
├── 进度条通知
├── 远程转发（Named Pipe / WebSocket）
├── Windows Widget
└── 语音播报（SAPI TTS）
```

### 11.5 成功指标

| 阶段 | 指标 | 目标值 |
|------|------|--------|
| Phase 1 发布后 1 个月 | GitHub Stars | > 50 |
| Phase 1 发布后 1 个月 | 安装用户 | > 30 |
| Phase 2 发布后 | GitHub Stars | > 200 |
| Phase 2 发布后 | 被 awesome-claude-code 收录 | Yes |
| 6 个月后 | Stars | > 500 |
| 6 个月后 | 企业用户反馈 | > 5 |

### 11.6 推广策略

1. **首发**：Reddit r/ClaudeCode + r/ClaudeAI（社区活跃且欢迎工具分享）
2. **演示**：录制 30 秒 GIF 展示 Toast + Click-to-Focus + 交互按钮的流程
3. **差异化叙事**："claude-notifications-go 说 Windows Click-to-Focus 不支持，我们做到了"
4. **Issue 引流**：在 claude-notifications-go #36（Windows click-to-focus 功能请求）下留言分享
5. **awesome-list**：提交到 awesome-claude-code / awesome-anthropic
6. **SEO 布局**：README 英文为主 + 中文文档辅助（覆盖两个市场）

### 11.7 项目命名决策：`claude-win-notify`

**最终推荐：`claude-win-notify`**

选择理由：

| 维度 | 分析 |
|------|------|
| **SEO 友好** | 名称包含三个核心搜索词：`claude`（产品）+ `win`（平台）+ `notify`（功能）。用户搜索 "claude windows notification" / "claude code win notify" / "claude win toast" 时能自然命中，无需额外推广成本 |
| **对比其他候选** | `ClaudePing` — 搜 "claude ping" 会混入网络延迟结果；`Claude Beacon` — "beacon" 在 web 领域指追踪像素，容易歧义；`WinClaude` — 搜索时与 "Windows Claude" 安装教程混淆 |
| **命名惯例一致** | 与竞品 `claude-notifications-go` 风格一致（小写 + 连字符），符合 GitHub 仓库命名习惯 |
| **简洁性** | 16 个字符，易于记忆、输入和口口相传 |
| **域名/仓库可用性** | GitHub 仓库名可用 ✓ |

**项目口号**："The Windows notification experience Claude Code deserves."

### 11.8 关键决策汇总

| # | 决策 | 理由 |
|---|------|------|
| 1 | 项目名 `claude-win-notify` | SEO 友好 + 简洁 + 仓库可用 |
| 2 | 优先 Phase 0 PoC | 先用 2 小时验证核心链路，降低技术风险 |
| 3 | 不做跨平台 | "反定位"优势：专注 Windows = 更少代码、更好体验、更快迭代 |
| 4 | Phase 1 不做交互按钮 | COM 激活服务复杂度高，MVP 先做好"通知+聚焦" |
| 5 | 安装方式 | PowerShell `iwr ... \| iex` + 自动注入 hooks.json，优于竞品的 Git Bash 依赖 |
| 6 | MIT License | 企业友好，对标竞品 GPL-3.0 劣势 |
| 7 | C# .NET 8 NativeAOT | 技术选型最终确认（见第三章分析） |

### 11.9 总结

本项目有明确的市场空白、真实的用户痛点、和可行的技术路径。最大的竞争优势是**定位清晰**——不是"又一个跨平台通知工具"，而是"Windows 上唯一做到 Click-to-Focus + 交互式通知的方案"。

Claude Code 正处于 10x 增长期（119K Stars, $25 亿 ARR），Windows 企业用户是最大的未被服务群体。竞品 `claude-notifications-go` 的 Windows 支持持续存在严重问题（至少 4 个 HIGH severity issues），且其 Go 技术栈在 Windows 原生 API 集成上有架构性劣势。

建议立即启动 Phase 0 PoC，验证技术可行性后推进 Phase 1 MVP。目标 2-3 周内有可用版本发布到 GitHub。
