# claude-win-notify

## What This Is

Windows-First 的 Claude Code 通知插件。点击通知直接聚焦到正在运行 Claude Code 的终端窗口和标签页，解决竞品在 Windows 上 Click-to-Focus 完全缺失的痛点。目标是成为 Windows 上唯一真正可用的 Claude Code 通知方案，在 GitHub 上获得广泛认可。

## Core Value

Click-to-Focus 在 Windows 上真正可用——通知弹出后点击即可跳转到正确的终端窗口和标签页，这是竞品明确标注不支持的功能。

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Windows Toast 通知（4 种类型：任务完成、权限请求、提问、错误告警）
- [ ] Click-to-Focus 窗口级聚焦（SetForegroundWindow）
- [ ] Click-to-Focus 标签页级聚焦（Warp + Windows Terminal）
- [ ] Hook 系统对接（读取 Claude Code stdin JSON + JSONL transcript 解析）
- [ ] 通知类型智能判定（状态机分析 JSONL 日志）
- [ ] 冷却时间 + 去重（防止通知轰炸）
- [ ] PowerShell 一键安装（自动下载 exe + 注入 hooks.json）
- [ ] 零运行时依赖（单一可执行文件）
- [ ] 交互式通知（从 Toast 中 Approve/Deny/Reply）
- [ ] AI 智能摘要（解析上下文生成有意义的通知描述）
- [ ] 通知聚合（多子 agent 并发时合并通知）
- [ ] 扩展终端支持（VS Code、ConEmu、PowerShell 独立窗口等）
- [ ] 手机推送（Bark/Pushover）
- [ ] 企业友好（代码签名 + Intune/GPO 兼容）
- [ ] winget/scoop 分发

### Out of Scope

- 跨平台支持（macOS/Linux） — 那是 claude-notifications-go 的优势，本项目专注 Windows 极致体验
- Web 仪表板 — 增加维护负担，核心价值不在这里
- 语音播报（TTS） — 锦上添花，优先级极低
- Windows Widget 集成 — Win11 专属且 API 不稳定

## Context

- 竞品 claude-notifications-go（617+ Stars）明确标注 Windows Click-to-Focus 不支持
- Claude Code 正处于爆发期（119K Stars），Windows 企业用户是最大未被服务群体
- 竞品 Go 技术栈在 Windows 原生 API（WinRT/COM/UI Automation）集成上有架构性劣势
- Warp 和 Windows Terminal 是目标用户的主力终端
- 用户企业环境有 Device Guard 限制，未签名 exe 会被拦截
- 项目口号："The Windows notification experience Claude Code deserves."

## Constraints

- **Platform**: Windows 10 1903+ 和 Windows 11 — 不做向下兼容
- **Distribution**: 单一可执行文件，零运行时依赖 — 这是核心卖点
- **Enterprise**: 必须考虑代码签名路径 — 否则企业用户无法使用
- **Tech Stack**: Rust (windows-rs) — Phase 1 spike 验证通过，二进制 0.38 MB
- **License**: MIT — 企业友好，对标竞品 GPL-3.0 劣势
- **Performance**: Toast 显示延迟 < 500ms，Click-to-Focus 成功率 > 95%

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Windows-First 定位，不做跨平台 | 专注 = 更少代码、更好体验、更快迭代；竞品已覆盖跨平台 | — Pending |
| MVP 先支持 Warp + Windows Terminal | 用户主力终端，后续扩展所有主流终端 | — Pending |
| PowerShell 一键安装 | 零前置依赖，30 秒搞定，README 演示效果好 | — Pending |
| MIT License | 企业友好，对标竞品 GPL-3.0 劣势 | — Pending |
| **Production tech stack: Rust** | Enterprise control 平手 (D-10)；Rust 以 8.6x 更小二进制、更简构建链、编译时内存安全胜出 (D-11) | ✓ Phase 1 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-15 after initialization*
