# AGENTS.md

本文件是后续自动化代理执行“游戏资源管理器”相关任务时的项目级约定。该项目不是 Godot 项目；不要按 Godot 工程规则处理。

## 项目定位

- 产品名称：游戏资源管理器 / Game Resource Manager。
- 产品形态：本地优先的独立 Windows 桌面应用。
- 第一版目标：把多个本地素材文件夹变成可搜索、可预览、可收藏、可复制路径的资源库。
- 核心边界：完全本地化，不走云端，不接管原文件结构，不强绑定 Godot。
- 技术栈：Tauri 2 + React 18 + TypeScript + Vite + Rust + SQLite(sqlx)。

## 固定路径

- 策划文档目录：`G:\oklai999的策划仓库\游戏资源管理器`
- 需求文档：`G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
- 实施计划：`G:\oklai999的策划仓库\游戏资源管理器\实施计划_v0.1.md`
- 项目代码目录：`I:\GameResManger`
- 打包路径: I:\GameResManger\releases

执行代码、测试、构建、查看源码时，默认进入 `I:\GameResManger`。

## 开始任务前

1. 先读取需求文档和实施计划中与当前任务相关的章节。
2. 在 `I:\GameResManger` 运行 `git status --short`，确认工作区状态。
3. 使用 `rg --files` 查找相关文件；如果没有 `rg`，再使用 PowerShell 替代。
4. 先看项目已有实现和测试，再修改代码。
5. 如果实施计划任务板仍在使用，优先选择第一个未完成任务，除非用户明确指定其他任务。

## 安全边界

本项目管理的是用户本地素材文件。自动化代理必须保护原始素材文件：

- 不删除原始素材文件。
- 不移动原始素材文件。
- 不重命名原始素材文件。
- 不修改原始素材文件内容。
- 不向原始素材目录写入缩略图、缓存或数据库文件。
- 缩略图缓存和 SQLite 数据只能写入应用数据目录或项目约定的位置。

MVP 允许的安全动作只有：

- 打开文件。
- 打开所在目录。
- 复制绝对路径。
- 添加/移除索引记录。
- 编辑应用内标签、收藏、集合、备注。
- 标记文件缺失，而不是自动删除索引或源文件。

## 删除命令禁令

禁止执行任何批量删除文件或目录的指令，包括但不限于：

- `del /s`
- `rd /s`
- `rmdir /s`
- `Remove-Item -Recurse`
- `rm -rf`

需要删除文件时，只能一次删除一个明确路径的文件，例如：

```powershell
Remove-Item -LiteralPath "C:\path\to\file.txt"
```

如果任务看起来需要批量删除文件或目录，必须停止操作并向用户请求确认，让用户手动删除或明确授权具体路径和范围。

## 代码结构

代码位于 `I:\GameResManger`：

- `src/`：React + TypeScript 前端。
- `src/api/tauri.ts`：Tauri command 的类型化前端封装。
- `src/components/`：三栏工作台 UI 组件。
- `src/types/`：前端领域类型。
- `src-tauri/src/commands.rs`：Tauri command 边界。
- `src-tauri/src/db.rs`：SQLite 连接、迁移和 repository。
- `src-tauri/src/indexer.rs`：目录遍历、文件分类、扫描索引。
- `src-tauri/src/thumbnails.rs`：图片缩略图生成。
- `src-tauri/src/tags.rs`：标签、收藏等整理能力。
- `src-tauri/src/file_actions.rs`：安全文件动作。
- `src-tauri/migrations/`：SQLite migration。
- `tests/smoke/README.md`：手动烟测清单。

## 实现原则

- 前端不直接访问文件系统或数据库；本地能力必须通过 Tauri command 调用 Rust 后端。
- Rust 后端负责文件夹选择、扫描、元数据读取、打开文件/所在目录、复制路径、缩略图生成和 SQLite 写入。
- 扫描逻辑使用 `path + file_size + modified_at` 判断变化；MVP 不做内容哈希。
- 文件不存在时标记 `is_missing`，不要自动删除数据库记录或源文件。
- 缩略图生成失败时，资源仍应进入索引，前端显示类型占位状态。
- 标签名保存前应 trim，并做一致化处理，避免视觉重复标签。
- 同一时间只运行一个扫描任务，避免数据库写入和缩略图生成互相抢占。
- 不新增 SVN、版本控制集成、AI 自动打标、删除、批量移动、批量重命名、图集处理或完整 3D/Spine 运行时预览，除非用户明确改变范围。

## UI 与产品约束

- 主界面是三栏工作台：左侧资源库/筛选，中间搜索与资源网格，右侧详情/批量操作。
- MVP 以网格视图为主，资源卡片显示缩略图、格式、文件名、尺寸/大小、收藏状态。
- 搜索范围包括文件名、标签、备注、路径。
- 右侧面板单选显示资源详情，多选显示批量标签/集合/收藏操作。
- UI 文案默认中文，保持工具型桌面软件的清晰、克制和高信息密度。

## 常用命令

在 `I:\GameResManger` 执行：

```powershell
npm test
npm run build
npm run dev
npm run tauri dev
```

在 `I:\GameResManger\src-tauri` 执行：

```powershell
cargo test
cargo check
```

Windows 上如果 Rust 编译报缺少 `stdarg.h`、`excpt.h`、`msvcrt.lib` 等，优先判断为本机 MSVC 或 Windows SDK 环境问题，不要随意改业务代码规避。

## 验证要求

完成改动后，按影响范围选择验证：

- 前端组件或 TypeScript 改动：运行 `npm test` 和/或 `npm run build`。
- Rust 后端改动：在 `src-tauri` 运行 `cargo test` 和/或 `cargo check`。
- Tauri command 或端到端行为改动：尽量运行 `npm run tauri dev` 做桌面应用烟测。
- 资源扫描、缩略图、路径、文件动作相关改动：参考 `tests/smoke/README.md` 做手动烟测。

如果因为 GUI、依赖、MSVC/SDK 或权限问题无法运行某项验证，必须在回复中说明原因，并列出已完成的替代检查。

## Git 与任务记录

- 不使用 `git reset --hard`、`git checkout --` 等会覆盖用户改动的命令，除非用户明确要求。
- 不回滚用户已有改动。
- 如果按实施计划执行任务，完成后更新实施计划中的任务状态和备注。
- 提交前确认 `git status --short`，只包含本任务相关改动。

## 当前已知状态

- 项目已按 MVP 计划完成基础实现与测试文档。
- README 记录的当前验证预期：`npm test` 约 5 个测试通过，`npm run build` 成功；`cargo test` 和 `cargo check` 依赖本机 MSVC/Windows SDK 配置。
- 实施计划中曾记录过一个历史风险：Tauri command 依赖的数据库连接池状态需要在运行时正确注册；后续涉及命令调用或启动流程时应优先复查 `src-tauri/src/lib.rs`。
