# Phase 2: Hook & Toast Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-20
**Phase:** 02-hook-toast
**Areas discussed:** Project Structure & Architecture, Hook stdin Parsing, Toast Notification Content & Style, Development & Testing Strategy

---

## Project Structure & Architecture

### Crate Structure

| Option | Description | Selected |
|--------|-------------|----------|
| 单 crate + 模块分文件 | src/main.rs + src/lib.rs + 模块文件，标准小型 Rust CLI 做法 | ✓ |
| Workspace 多 crate | core/cli/toast 独立 crate，过度设计 | |
| 单 crate 纯 binary | 无 lib.rs，集成测试不方便 | |

**User's choice:** 单 crate + 模块分文件
**Notes:** 用户询问了主流 Rust 项目的结构实践。调研后确认：<5K LOC 项目用单 crate 是标准做法（fd, bat 等均如此）。Workspace 适合 20K+ LOC 需独立发布的场景。

### CLI 设计

| Option | Description | Selected |
|--------|-------------|----------|
| 子命令模式 (clap) | `hook` / `focus` 子命令，职责清晰，可扩展 | ✓ |
| Flag 模式 | 默认=hook，--focus 触发跳转，更简单但不利扩展 | |

**User's choice:** clap 子命令模式
**Notes:** 用户询问了 clap 是什么。解释后确认使用 clap derive macros。

### Exe 命名

| Option | Description | Selected |
|--------|-------------|----------|
| claude-notify | 简短，但和仓库名不一致 | |
| claude-win-notify | 和仓库名一致 | ✓ |
| cn-notify | 最短，但辨识度低 | |

**User's choice:** claude-win-notify

---

## Hook stdin Parsing

### Hook 事件范围

| Option | Description | Selected |
|--------|-------------|----------|
| 最小集：Stop + Notification | 只处理任务完成和系统通知，其他静默退出 | ✓ |
| 全量：所有事件都通知 | 简单但导致通知轰炸 | |
| 极简：仅 Stop | 最简单的 MVP | |

**User's choice:** Stop + Notification
**Notes:** Phase 3 再扩展通知类型判定。

### 解析失败处理

| Option | Description | Selected |
|--------|-------------|----------|
| 静默退出 + 本地 log | exit 0 不干扰 Claude Code，错误写本地日志 | ✓ |
| 静默退出，无 log | 最简单但调试困难 | |
| 返回非零退出码 | 可能干扰 Claude Code 工作流 | |

**User's choice:** 静默退出 + 本地 log

---

## Toast Notification Content & Style

### Toast 内容

| Option | Description | Selected |
|--------|-------------|----------|
| 简洁：标题 + 项目名 | Title="Claude Code", Body="✔ Task Complete", Attribution=项目名 | ✓ |
| 详细：事件类型 + 完整路径 | 信息全但较丑 | |
| 品牌化：利用 attribution text | 需要自定义 AUMID（Phase 7） | |

**User's choice:** 简洁版

### Toast 点击行为

| Option | Description | Selected |
|--------|-------------|----------|
| 无操作（Phase 4 再加） | 点击仅消除通知 | ✓ |
| 预埋 Protocol URI | 点击触发但无实际效果 | |

**User's choice:** 无操作

---

## Development & Testing Strategy

### 测试方案

| Option | Description | Selected |
|--------|-------------|----------|
| stdin pipe + fixture + 单元测试 | 模拟 stdin + JSON fixture 文件 + 单元测试 | ✓ (part of) |
| 真实 Claude Code 集成测试 | 配置 hooks.json，dogfooding | ✓ (part of) |

**User's choice:** 三层都做（单元测试 + stdin pipe + 真实 dogfooding）
**Notes:** 用户询问真实 Claude Code 集成测试是否可行。解释后确认：hook 是 fire-and-forget，exit 0 不影响 Claude Code，完全安全。

### 二进制优化

| Option | Description | Selected |
|--------|-------------|----------|
| 极致压缩 | strip + LTO + opt-level=z + codegen-units=1 | ✓ |
| 默认 release | 不额外优化，3-5MB | |
| 后续再优化 | Phase 8 打包时再做 | |

**User's choice:** 极致压缩

---

## Claude's Discretion

- Log 文件存放位置（建议 `%LOCALAPPDATA%\claude-win-notify\logs\`）
- Log rotation 机制
- 具体模块文件拆分粒度

## Deferred Ideas

None — discussion stayed within phase scope
