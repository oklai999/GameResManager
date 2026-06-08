# 模块地图

## 1. 顶层目录结构

```text
I:\GameResManger
├── src/                         React + TypeScript 前端
│   ├── api/                     Tauri command 类型化封装
│   ├── components/              三栏工作台组件
│   ├── test/                    前端测试 setup
│   ├── types/                   前端领域类型
│   ├── App.tsx                  主状态容器
│   ├── main.tsx                 React 入口
│   └── styles.css               全局样式
├── src-tauri/                   Tauri + Rust 后端
│   ├── capabilities/            Tauri v2 权限
│   ├── migrations/              SQLite migration
│   ├── src/                     Rust 源码
│   ├── Cargo.toml               Rust 依赖和 crate 配置
│   └── tauri.conf.json          Tauri 应用配置
├── tests/smoke/                 手动烟测清单
├── docs/superpowers/plans/      历史实施计划
├── releases/                    打包产物
├── package.json                 npm 配置
├── vite.config.ts               Vite/Vitest 配置
├── tsconfig.json                TypeScript 配置
├── package.ps1                  Windows 打包脚本
└── README.md                    项目说明
```

## 2. 前端模块

### `src/main.tsx`

React 入口文件，创建 root，渲染 `App`，加载全局 CSS。

### `src/App.tsx`

主应用容器，负责：

- 初始数据加载。
- 资源、文件夹、集合、标签、扫描任务、扫描设置状态。
- 搜索请求组装和结果管理。
- 轮询 running 扫描任务。
- 左侧筛选、中间网格、右侧详情之间的状态协调。
- 调用 Tauri API 并处理 toast 错误。

主要调用：

- 调用 `src/api/tauri.ts` 中的所有业务 API。
- 渲染 `LibrarySidebar`、`SearchToolbar`、`ScanStatusBar`、`AssetGrid`、`DetailsPanel`、`SettingsPanel`、`EmptyState`。

### `src/api/tauri.ts`

前端到后端的唯一调用封装层。每个函数基本对应一个 Rust command：

- 资源/文件夹：`listAssets`、`listLibraryFolders`、`createLibraryFolderFromPath`、`deleteLibraryFolder`。
- 搜索：`searchAssets`。
- 扫描：`startScan`、`cancelScan`、`latestScanJob`、`getScanSettings`、`saveScanSettings`。
- 标签：`listAssetTags`、`listTags`、`getAssetTags`、`listCommonTags`、`applyTagToAssets`。
- 集合：`listCollections`、`createCollection`、`addAssetsToCollection`、`removeAssetFromCollection`、`listCollectionAssets`。
- 文件动作：`openAssetFile`、`revealAssetInFolder`。
- 缩略图/备注：`assetThumbnailUrl`、`updateAssetNote`。

### `src/types/asset.ts`

前端领域类型，和 Rust `models.rs` 保持字段名一致。核心类型：

- `Asset`
- `LibraryFolder`
- `SearchScope`
- `ScanJob`
- `AssetSearchRequest`
- `Tag`
- `Collection`
- `ScanSettings`

### `src/components/LibrarySidebar.tsx`

左侧栏组件。负责展示：

- 固定筛选项：全部、收藏、缺失、图片、音频、视频、字体、3D、Spine。
- 素材文件夹列表。
- 扫描/取消扫描按钮。
- 删除文件夹索引按钮。
- 集合列表和创建集合表单。
- 设置面板插槽。

### `src/components/SearchToolbar.tsx`

中间顶部搜索栏。负责搜索输入和搜索域勾选：

- 文件名
- 标签
- 备注
- 路径

### `src/components/AssetGrid.tsx`

资源网格。负责：

- 渲染资源卡片。
- 显示缩略图或类型占位状态。
- 使用 `convertFileSrc` 把本地缩略图路径转成前端可显示 URL。
- 多选切换。
- 收藏按钮。

### `src/components/DetailsPanel.tsx`

右侧详情和批量操作面板。三种状态：

- 未选中：空状态。
- 单选：显示文件信息、尺寸、路径、缩略图错误、备注、标签、集合、收藏、打开文件、打开所在目录、复制路径。
- 多选：显示数量、共同标签、批量添加标签、批量加入集合。

### `src/components/SettingsPanel.tsx`

扫描设置面板。负责：

- 文件类型扫描开关。
- PSD 索引和 PSD 缩略图开关。
- 忽略目录编辑。
- 忽略扩展名编辑。
- 数据库路径和缩略图缓存路径只读展示。

### `src/components/ScanStatusBar.tsx`

扫描状态条。展示：

- running/completed/failed/cancelled 状态。
- 当前路径。
- 发现、新增、更新、未变化、缺失、跳过统计。
- 错误信息。

### `src/components/TagEditor.tsx`

标签编辑组件。加载已有标签，提供输入、添加和候选标签按钮。

### `src/components/ToastHost.tsx`

轻量 toast context。`AppInner` 通过 `useToast` 展示成功、错误、信息提示。

### `src/components/EmptyState.tsx`

空状态组件，用于无文件夹、无资源、无搜索结果、文件缺失等场景。

## 3. 后端模块

### `src-tauri/src/main.rs`

Rust 二进制入口，只调用 `game_resource_manager_lib::run()`。

### `src-tauri/src/lib.rs`

Tauri 应用启动模块。负责：

- 注册 dialog 和 opener 插件。
- 解析应用数据目录。
- 初始化 SQLite。
- 创建缩略图目录。
- 注册 Tauri State：`SqlitePool`、`ThumbnailDir`、`ScanRuntime`。
- 写入数据库路径/缩略图路径到 `scan_settings`。
- 后台清理旧扫描任务。
- 注册所有 command。

### `src-tauri/src/commands.rs`

Tauri command 边界。职责：

- 把前端参数转换为后端调用。
- 从 Tauri State 取 `SqlitePool`、`ThumbnailDir`、`ScanRuntime`。
- 把 anyhow/sqlx 错误统一成 `{ message }`。
- 启动后台扫描任务。
- 调用 dialog 选择文件夹。

主要 command：

- `list_assets`
- `list_library_folders`
- `set_asset_favorite`
- `apply_tag_to_assets`
- `open_asset_file`
- `reveal_asset_in_folder`
- `add_library_folder`
- `pick_library_folder`
- `create_library_folder_from_path`
- `delete_library_folder`
- `list_asset_tags`
- `list_tags`
- `get_asset_tags`
- `list_common_tags`
- `list_collections`
- `create_collection`
- `add_assets_to_collection`
- `remove_asset_from_collection`
- `list_collection_assets`
- `asset_thumbnail_url`
- `update_asset_note`
- `start_scan`
- `cancel_scan`
- `latest_scan_job`
- `get_scan_settings`
- `save_scan_settings`
- `search_assets`

### `src-tauri/src/db.rs`

SQLite 连接、migration 和 repository。主要职责：

- 解析数据库路径：`resolve_database_path`。
- 打开 SQLite 并执行 migrations：`connect`。
- 管理素材文件夹：创建、列表、按 id/path 查询、删除索引、更新最后扫描时间。
- 管理资源：列表、收藏、备注更新。
- 管理标签：创建/获取、绑定、查询所有标签、查询资源标签、查询共同标签。
- 管理集合：创建、列表、加入/移出资源、查询集合资源。
- 管理扫描任务：创建、查询 latest/running、更新进度、完成、失败、取消、清理历史任务。
- 管理扫描缺失标记：`mark_missing_assets_from_seen`。
- 管理扫描设置和应用路径：`get_scan_settings`、`save_scan_settings`、`ensure_app_paths`。

### `src-tauri/src/scan_service.rs`

后台扫描服务。主要职责：

- `ScanRuntime` 维护扫描取消、活跃任务、启动中任务和文件夹锁。
- `run_scan_job` 执行实际扫描。
- 使用 `WalkDir` 遍历素材目录。
- 调用 `indexer` 判断文件类型、忽略目录、忽略扩展名、缩略图策略。
- 按 500 个资源一批 flush。
- 在批次 flush 中生成缩略图、写入 `scan_seen_paths`、upsert `assets`、更新扫描进度。
- 扫描末尾标记缺失文件，完成任务并更新 `last_scanned_at`。
- 取消时保留已入库资源，并把任务设为 cancelled。

### `src-tauri/src/indexer.rs`

扫描规则和文件识别模块：

- `classify_asset`：按扩展名分类 image/audio/video/font/model3d/spine/other。
- `normalize_path`：把路径转为绝对路径并统一 `/`。
- `should_ignore_dir`：按设置忽略目录名。
- `should_ignore_extension`：按设置忽略扩展名。
- `asset_type_allowed`：按扫描设置判断类型是否启用。
- `should_generate_thumbnail`：判断是否生成缩略图，PSD 默认受设置控制。
- `ScannedAsset`：扫描中间结构。

### `src-tauri/src/search.rs`

搜索服务。根据 `AssetSearchRequest` 动态组合 SQL：

- query 可匹配文件名、备注、路径、标签。
- 可叠加类型、文件夹、集合、收藏、缺失过滤。
- `LIKE` 模式会转义 `%`、`_` 和反斜杠。
- limit 被限制在 1 到 2,000。

### `src-tauri/src/thumbnails.rs`

缩略图模块：

- 使用 `source_path + modified_at` 计算 SHA1，生成稳定 WebP 缓存路径。
- 使用 `image` 读取图片，生成最大 320x320 的 WebP 缩略图。
- `generate_image_thumbnail_async` 通过 `spawn_blocking` 避免阻塞 async runtime。

### `src-tauri/src/file_actions.rs`

安全文件动作：

- `open_file`：确认文件存在后调用系统默认方式打开。
- `reveal_in_folder`：确认父目录存在后打开所在目录。

不包含删除、移动、重命名、写入源文件等危险操作。

### `src-tauri/src/tags.rs`

标签规范化：

- trim 首尾空白。
- 合并连续空白。
- 转小写。

### `src-tauri/src/models.rs`

Rust 领域模型和序列化结构：

- `AssetType`
- `LibraryFolder`
- `Asset`
- `Tag`
- `Collection`
- `ScanJobStatus`
- `ScanJob`
- `ScanSettings`
- `AssetSearchRequest`

## 4. 数据表和模块对应关系

| 数据表 | 主要读写模块 | 前端使用位置 |
| --- | --- | --- |
| `library_folders` | `db.rs`, `commands.rs`, `scan_service.rs` | `LibrarySidebar`, `App.tsx` |
| `assets` | `db.rs`, `search.rs`, `scan_service.rs` | `AssetGrid`, `DetailsPanel`, `App.tsx` |
| `tags` | `db.rs`, `tags.rs` | `TagEditor`, `DetailsPanel` |
| `asset_tags` | `db.rs`, `search.rs` | `TagEditor`, `DetailsPanel`, 搜索 |
| `collections` | `db.rs` | `LibrarySidebar`, `DetailsPanel` |
| `collection_assets` | `db.rs`, `search.rs` | 集合筛选、加入集合 |
| `scan_jobs` | `db.rs`, `scan_service.rs`, `commands.rs` | `LibrarySidebar`, `ScanStatusBar`, `App.tsx` |
| `scan_seen_paths` | `db.rs`, `scan_service.rs` | 不直接暴露 |
| `scan_settings` | `db.rs`, `indexer.rs`, `scan_service.rs` | `SettingsPanel` |

## 5. 典型调用路径速查

### 添加文件夹

```text
LibrarySidebar 添加文件夹
-> App.handlePickFolder
-> pickLibraryFolder
-> commands::pick_library_folder
-> Tauri dialog
-> createLibraryFolderFromPath
-> commands::create_library_folder_from_path
-> indexer::normalize_path
-> db::create_library_folder
-> App.loadData
```

### 扫描文件夹

```text
LibrarySidebar 扫描
-> App.handleScanFolder
-> startScan
-> commands::start_scan
-> db::create_scan_job
-> ScanRuntime 标记 active
-> spawn scan_service::run_scan_job
-> WalkDir 遍历
-> indexer 分类/规则
-> thumbnails 生成缩略图
-> db 写 assets / scan_seen_paths / scan_jobs
-> App 轮询 latestScanJob
```

### 搜索资源

```text
SearchToolbar / Sidebar 改变条件
-> App.executeSearch
-> searchAssets
-> commands::search_assets
-> search::search_assets
-> SQLite 查询 assets + tag/collection 子查询
-> AssetGrid 渲染
```

### 添加标签

```text
DetailsPanel / TagEditor
-> App.handleApplyTag
-> applyTagToAssets
-> commands::apply_tag_to_assets
-> db::apply_tag_to_assets
-> db::create_or_get_tag
-> tags::normalize_tag_name
-> 写 tags / asset_tags
-> App.loadData
```

### 编辑备注

```text
DetailsPanel textarea blur / 保存按钮
-> updateAssetNote
-> commands::update_asset_note
-> db::update_asset_note
-> 返回更新后的 Asset
-> App.handleUpdateNote 局部更新 assets/gridAssets
```

### 收藏资源

```text
AssetGrid 或 DetailsPanel
-> App.handleToggleFavorite
-> setAssetFavorite
-> commands::set_asset_favorite
-> db::set_asset_favorite
-> App 局部更新 assets/gridAssets
```

### 打开文件/目录

```text
DetailsPanel
-> openAssetFile / revealAssetInFolder
-> commands::open_asset_file / reveal_asset_in_folder
-> file_actions::open_file / reveal_in_folder
-> open::that
```

## 6. 测试覆盖地图

前端测试：

- `AssetGrid.test.tsx`
- `AssetGridFavorite.test.tsx`
- `DetailsPanel.test.tsx`
- `LibrarySidebar.test.tsx`
- `TagEditor.test.tsx`

后端测试分布：

- `indexer.rs`：分类、路径、忽略目录/扩展名、扫描设置。
- `thumbnails.rs`：缩略图缓存路径、生成和缩放。
- `tags.rs`：标签规范化。
- `search.rs`：查询条件组合和集成搜索。
- `db.rs`：集合和应用路径设置。
- `scan_service.rs`：扫描完成、取消、增量、缺失恢复、缩略图保留、stale job recovery、running 唯一性、取消进度等。

手动烟测：

- `tests/smoke/README.md` 覆盖基础资源扫描、搜索、筛选、标签、文件动作、安全边界，以及大目录 release 扫描、取消、增量和 PSD 行为。
