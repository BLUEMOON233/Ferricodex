# Ferricodex

中文 | [English](README.en.md)

Ferricodex 是一个 Rust 原生、本地优先的 Codex 历史管理器。

本项目刻意聚焦在 Codex 历史记录管理，而不是做成更宽泛的 Agent 切换器。第一个稳定目标是提供一个以读取为主的界面，用于浏览、搜索、预览和整理本地 Codex 会话。

## 友链

[![友链 linux.do](https://img.shields.io/badge/LINUX--DO-Community-blue.svg?logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMTIwIiBoZWlnaHQ9IjEyMCIgdmlld0JveD0iMCAwIDEyMCAxMjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI%2BPGNsaXBQYXRoIGlkPSJhIj48Y2lyY2xlIGN4PSI2MCIgY3k9IjYwIiByPSI0NyIvPjwvY2xpcFBhdGg%2BPGNpcmNsZSBmaWxsPSIjZjBmMGYwIiBjeD0iNjAiIGN5PSI2MCIgcj0iNTAiLz48cmVjdCBmaWxsPSIjMWMxYzFlIiBjbGlwLXBhdGg9InVybCgjYSkiIHg9IjEwIiB5PSIxMCIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIzMCIvPjxyZWN0IGZpbGw9IiNmMGYwZjAiIGNsaXAtcGF0aD0idXJsKCNhKSIgeD0iMTAiIHk9IjQwIiB3aWR0aD0iMTAwIiBoZWlnaHQ9IjQwIi8%2BPHJlY3QgZmlsbD0iI2ZmYjAwMyIgY2xpcC1wYXRoPSJ1cmwoI2EpIiB4PSIxMCIgeT0iODAiIHdpZHRoPSIxMDAiIGhlaWdodD0iMzAiLz48L3N2Zz4%3D&style=flat)](https://linux.do/)

## 技术栈

- Tauri 2：桌面应用外壳
- Rust：Codex 数据访问、索引和安全文件操作
- Svelte 5 + SvelteKit static adapter + Vite：前端 UI
- TypeScript：前端代码
- Tailwind CSS 4：样式基础
- shadcn-svelte/Bits UI：随着 UI 扩展，计划用于可复用控件

## 本地数据策略

应用应避免污染用户环境：

- 不修改 shell profile、PATH、登录项或系统服务。
- 如果设置了 `CODEX_HOME`，从该目录读取 Codex 数据；否则从平台默认用户目录读取。
- 应用自己的设置、索引和日志集中保存在一个应用数据目录中。
- 搜索索引视为可丢弃缓存，可从 Codex session 文件重新构建。
- 在写操作被明确设计之前，对 Codex 拥有的文件默认保持只读。
- 对 Codex 拥有文件的写操作必须范围小、经用户确认，并兼容 Codex 自身的存储结构。

预期读取的 Codex 输入：

```text
~/.codex/state_5.sqlite
~/.codex/session_index.jsonl
~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
~/.codex/archived_sessions/rollout-*.jsonl
```

预期由应用拥有的输出：

```text
<platform app data>/Ferricodex/
  settings.json
  app.db
  search-index/
  logs/
```

## 开发

安装前置依赖：

- Node.js 18 或更高版本
- 带有 `cargo` 的 Rust 工具链
- 对应操作系统所需的 Tauri 桌面依赖

安装前端依赖：

```bash
npm install
```

运行前端检查：

```bash
npm run check
```

运行桌面应用：

```bash
npm run tauri dev
```

构建发布包：

```bash
npm run tauri build
```

## 当前状态

项目目前可以从 `state_5.sqlite` 读取本地 Codex thread 元数据，渲染可搜索的活跃/归档 session 与 workspace 视图，打开本地文件夹，并且只在选中 session 时按需解析/筛选有边界的 transcript 预览。项目也支持在活跃、归档或全部 Codex session 中进行有边界的按需全局 transcript 搜索，并可通过历史面板中的同步按钮手动重新读取 Codex 新增的会话。

应用可以在用户确认后，以兼容 Codex 的方式在活跃/归档位置之间移动 session，或删除选中的 session 与 workspace 历史，并清理已知 Codex 数据库记录和 `session_index.jsonl` 条目。位于 `~/Documents/Codex/YYYY-MM-DD/<folder>` 下的 Codex 生成任务目录，可以随同一规范化 `cwd` 下的所有关联 session 一起移动到系统废纸篓，也可以先保存副本到 `~/Documents/Ferricodex Saved Workspaces/` 后再删除原目录。用户项目 workspace 可以从 Codex 历史中移除，而不会触碰实际项目文件。

已知问题：

- [ ] Windows 桌面快捷方式图标可能显示为空白文档，而不是 Tauri 应用图标；发布前需要检查生成的 `.ico`、安装器快捷方式元数据以及 Windows 图标缓存。

## 更新日志

### v0.2.1

- 新增历史面板同步按钮，可在 Ferricodex 已打开时手动重新读取 Codex 新增的会话，并尽量保留当前选中的 session 或 workspace。
- 修复 Codex 生成任务目录被多个 session 引用时无法删除的问题；现在会删除同一规范化 `cwd` 下的所有关联 session，再处理生成目录。
- 更新生成 workspace 删除说明和安全边界文档，明确多会话关联目录的处理方式。
- 同步版本号到 `0.2.1`，用于发布 `v0.2.1`。

### v0.2.0

- 新增 Provider 设置管理，可在设置页读取并保存 Codex `config.toml` 中的默认模型与 `model_providers` 配置。
- 新增 API Key 更新入口，写入 Codex `auth.json`，不会显示或回填已有密钥。
- 新增工作区 `AGENTS.md` 管理，可在工作区详情中读取、创建和保存项目级 Agent 指令。
- 新增全局 `~/.codex/AGENT.md` 管理，可在设置页编辑 Codex 全局提示词。
- Agent 文档保存加入大小限制、符号链接拒绝和 revision 冲突检测，避免覆盖外部编辑器中的改动。
- 优化历史列表展示：移除工作区标题旁的来源标签，强化“项目 / 对话”分组标题，减少标题遮挡。

### v0.1.1

- 将项目完整重命名为 Ferricodex，包括应用显示名、Tauri bundle identifier、前端包名、Rust crate/lib 名、文档和 release workflow。
- 将默认 README 改为中文，并新增 `README.en.md` 英文版，支持中英文互跳。
- 同步版本号到 `0.1.1`，并让 release workflow 在发布前校验 package/Tauri/Cargo 版本一致性。
- 加固 Codex 本地路径访问边界：transcript 与 workspace 元数据读取仅允许访问当前 Codex 历史引用的路径。
- 为 transcript JSONL 解析增加单文件、单行和消息长度上限，避免异常历史文件阻塞应用。
- 删除 session 时兼容 transcript 文件已缺失的脏数据状态，只要路径仍属于 Codex session 目录，就继续清理数据库和 `session_index.jsonl`。
- 修正非生成 workspace 的删除弹窗语义，明确仅清理关联会话历史，不移动或删除用户项目目录。
- 优化批量选择工具条布局，并将“选中当前可见”精简为“全选”。
- 修复长 Windows 路径撑开删除弹窗的问题，避免弹窗按钮被裁切。

### v0.1.0

- 搭建 Tauri 2 + Svelte 5 + Rust 桌面应用基础架构。
- 从 `CODEX_HOME` 或默认用户目录检测 Codex home，并以只读方式读取 `state_5.sqlite` 中的 `threads` 表。
- 提供活跃/归档 session 列表、搜索、预览、选中 session 详情和 workspace 分组视图。
- 按需解析选中 session 的 transcript JSONL，并支持文本与角色筛选。
- 提供有边界的全局 transcript 搜索，可在活跃、归档或全部 Codex session 中扫描匹配内容。
- 支持 session 归档/取消归档，按 Codex 存储语义移动 rollout JSONL 并更新数据库状态。
- 支持单个或批量 session 移动到系统废纸篓，并清理 Codex thread、已知引用和 `session_index.jsonl` 条目。
- 支持删除 Codex 生成任务目录对应的 session 与 workspace，或先保存 workspace 副本再删除原目录。
- 支持从 Codex 历史中移除用户项目 workspace，同时保留实际项目文件。
- 添加多平台 GitHub release workflow，用于构建 macOS、Windows 和 Linux 未签名安装包。
- 生成 Tauri 桌面应用图标，并完成初始 GitHub 仓库推送。

已完成：

- [x] 搭建 Tauri 2 + Svelte 5 桌面应用脚手架。
- [x] 安装 Rust 工具链，并使用固定的 `rust-toolchain.toml`。
- [x] 从 `CODEX_HOME` 或默认用户目录检测 Codex home。
- [x] 在不修改 Codex 文件的前提下读取 `state_5.sqlite` 中的 `threads` 表。
- [x] 显示活跃和归档 session，并支持搜索、预览和选中 session 详情。
- [x] 添加 `Sessions` / `Workspaces` 分栏视图。
- [x] 按规范化后的 `cwd` 分组 session，使一个 workspace 可以展示多个相关对话。
- [x] 分类 workspace 来源，包括用户项目、生成的 `~/Documents/Codex/YYYY-MM-DD/<folder>` 任务目录和 `$CODEX_HOME/worktrees`。
- [x] 懒加载 workspace 元数据：是否存在、大小、最后修改时间和相关 session 数量。
- [x] 从主 Svelte 页面拆分前端 API、格式化、打开器和 workspace helper。
- [x] 将 Codex 后端访问拆分为聚焦的 `home`、`threads`、`transcript` 和 `workspaces` 模块。
- [x] 通过有边界的后端 opener 命令打开用户 home 下的文件夹。
- [x] 为选中 session 按需解析 transcript JSONL，并使用有边界的只读加载。
- [x] 将 transcript 和 workspace 元数据读取限制在当前 Codex 历史引用的路径内。
- [x] 为 transcript JSONL 解析添加单文件和单行字节上限。
- [x] 在不读取额外文件的情况下，按文本和消息角色筛选已加载的 transcript 预览。
- [x] 通过有边界的按需 JSONL 扫描，在活跃、归档或全部 session 中全局搜索 transcript 内容。
- [x] 通过在 `sessions` 与 `archived_sessions` 之间移动 rollout JSONL 文件并更新 Codex `state_5.sqlite`，实现选中 session 的归档和取消归档。
- [x] 将选中 session 移动到系统废纸篓：移动 rollout JSONL、删除匹配的 Codex thread 行、清理已知 thread 引用、删除匹配的 `session_index.jsonl` 条目，并要求用户确认。
- [x] 支持选择当前可见或单个 session，并通过同一套经确认且兼容 Codex 的废纸篓清理流程批量删除选中 session。
- [x] 对生成的 `~/Documents/Codex/YYYY-MM-DD/<folder>` workspace，支持将绑定 session 与生成目录一起删除，或先保存目录副本再将原目录移动到废纸篓。
- [x] 通过删除所有关联 session，将用户项目 workspace 从 Codex 历史中移除，同时保留项目文件不变。
- [x] 准备未签名的多平台 GitHub release workflow，用于 macOS、Windows 和 Linux 构建包。
- [x] 在 release workflow 中校验 release tag 格式，以及 package/Tauri/Cargo 版本一致性。
- [x] 使用项目 logo 生成 Tauri 桌面应用图标。
- [x] 保持生成的构建产物和依赖目录不进入 Git。
- [x] 将初始仓库推送到 GitHub。

下一步：

- [ ] 改进破坏性操作的恢复提示，尤其是 mutation 已成功但后续刷新失败的情况。
- [ ] 在添加任何 resume launcher 前，先调研官方 Codex Desktop deep link。
