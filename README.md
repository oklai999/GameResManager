# Game Resource Manager / 游戏资源管理器

Game Resource Manager is a local-first desktop MVP for indexing local game asset folders, browsing thumbnails, searching and filtering assets, applying tags, marking favorites, and using safe file actions.

游戏资源管理器是一个本地优先的桌面端 MVP，用于索引本地游戏素材文件夹、浏览缩略图、搜索和筛选资源、添加标签、标记收藏，并提供安全的文件操作。

The app does not delete, move, rename, or modify original asset files.

本应用不会删除、移动、重命名或修改原始素材文件。

## Tech Stack

- 技术栈：

- Tauri 2
- React 18
- TypeScript
- Vite
- Rust
- SQLite via `sqlx`
- Image thumbnails via Rust `image`
- Frontend tests via Vitest

## Features

- 功能：

- Add local asset folders.
- 添加本地素材文件夹。
- Scan folders and index common game asset file types.
- 扫描文件夹并索引常见游戏素材类型。
- Generate WebP thumbnails for image assets.
- 为图片素材生成 WebP 缩略图。
- Store thumbnails in the Tauri app data directory.
- 将缩略图保存到 Tauri 应用数据目录。
- Browse assets in a three-column workbench.
- 使用三栏工作台浏览资源。
- Search by file name, tag, note, or path.
- 按文件名、标签、备注或路径搜索。
- Filter by asset type, favorites, and missing files.
- 按资源类型、收藏和缺失文件筛选。
- Favorite and unfavorite assets.
- 收藏或取消收藏资源。
- Apply tags to one or more assets.
- 给单个或多个资源添加标签。
- Open files, reveal containing folders, and copy asset paths.
- 打开文件、打开所在目录并复制资源路径。
- Keep source files unchanged.
- 保持源文件不变。

## Supported Asset Types

- 支持的资源类型：

- Image: `png`, `jpg`, `jpeg`, `webp`, `bmp`, `gif`, `psd`
- Audio: `mp3`, `wav`, `ogg`, `flac`
- Video: `mp4`, `mov`, `webm`, `avi`
- Font: `ttf`, `otf`, `woff`, `woff2`
- 3D: `gltf`, `glb`, `obj`, `fbx`
- Spine: `skel`, `json`, `atlas`

## Project Structure

项目结构：

```text
.
|-- src/
|   |-- api/             # Typed Tauri command wrappers
|   |-- components/      # React UI components
|   |-- test/            # Frontend test setup
|   |-- types/           # Frontend domain types
|   |-- App.tsx          # Main workbench state and layout
|   |-- main.tsx         # React entry point
|   `-- styles.css       # App styles
|-- src-tauri/
|   |-- capabilities/    # Tauri v2 permissions
|   |-- migrations/      # SQLite migrations
|   |-- src/
|   |   |-- commands.rs  # Tauri command boundary
|   |   |-- db.rs        # SQLite connection and repositories
|   |   |-- file_actions.rs
|   |   |-- indexer.rs
|   |   |-- lib.rs
|   |   |-- main.rs
|   |   |-- models.rs
|   |   |-- tags.rs
|   |   `-- thumbnails.rs
|   |-- Cargo.toml
|   `-- tauri.conf.json
|-- tests/
|   `-- smoke/           # Manual QA checklist
|-- package.json
|-- tsconfig.json
`-- vite.config.ts
```

## Requirements

- 环境要求：

- Node.js and npm
- Rust stable toolchain
- Tauri prerequisites for your operating system
- Windows build tools for Rust native dependencies

On Windows, make sure the MSVC toolchain and Windows SDK headers are installed correctly. Missing headers such as `stdarg.h` or `excpt.h` indicate a local Visual Studio or Windows SDK setup issue.

在 Windows 上，请确认 MSVC 工具链和 Windows SDK 头文件已正确安装。如果出现缺少 `stdarg.h` 或 `excpt.h`，通常表示本机 Visual Studio 或 Windows SDK 环境有问题。

## Setup

安装依赖：

```powershell
npm install
```

## Development

Run the Tauri desktop app:

运行 Tauri 桌面应用：

```powershell
npm run tauri dev
```

Run only the Vite frontend:

只运行 Vite 前端：

```powershell
npm run dev
```

## Verification

Run frontend tests:

运行前端测试：

```powershell
npm test
```

Build the frontend:

构建前端：

```powershell
npm run build
```

Run Rust tests:

运行 Rust 测试：

```powershell
cd src-tauri
cargo test
```

Run Rust compile checks:

运行 Rust 编译检查：

```powershell
cd src-tauri
cargo check
```

Expected current status:

当前预期结果：

- `npm test`: 20+ tests passing.
- `npm test`：20+ 个测试通过。
- `npm run build`: succeeds.
- `npm run build`：构建成功。
- `cargo test`: 57+ tests passing when the local MSVC and Windows SDK environment is configured.
- `cargo test`：本机 MSVC 和 Windows SDK 环境配置正确时 57+ 测试通过。
- `cargo check`: passes when the local MSVC and Windows SDK environment is configured.
- `cargo check`：本机 MSVC 和 Windows SDK 环境配置正确时通过。

## Manual Smoke QA

See [tests/smoke/README.md](tests/smoke/README.md).

参见 [tests/smoke/README.md](tests/smoke/README.md)。

Recommended flow:

推荐流程：

1. Create a local fixture folder outside the repository.
1. 在仓库外创建一个本地测试素材文件夹。
2. Add sample files such as `icon.png`, `music.wav`, `video.mp4`, `font.ttf`, `model.glb`, and `hero.skel`.
2. 放入示例文件，例如 `icon.png`、`music.wav`、`video.mp4`、`font.ttf`、`model.glb` 和 `hero.skel`。
3. Add the fixture folder in the app.
3. 在应用中添加该素材文件夹。
4. Run scan.
4. 执行扫描。
5. Confirm assets appear in the grid.
5. 确认资源出现在网格中。
6. Confirm image thumbnails render.
6. 确认图片缩略图正常显示。
7. Confirm search, filters, tags, favorites, and file actions work.
7. 确认搜索、筛选、标签、收藏和文件操作可用。
8. Confirm source files are not modified.
8. 确认源文件没有被修改。

## Data Storage

- 数据存储：

- SQLite database: Tauri app data directory, `data.sqlite`.
- SQLite 数据库：Tauri 应用数据目录中的 `data.sqlite`。
- Thumbnail cache: Tauri app data directory, `thumbnails/`.
- 缩略图缓存：Tauri 应用数据目录中的 `thumbnails/`。
- Original files remain in their source folders.
- 原始文件保留在源文件夹中。

## Current Limitations

- 当前限制：

- Thumbnail generation runs inside the scan command and may take time for large image folders.
- 缩略图生成仍在扫描命令中执行，大型图片文件夹可能耗时较长。
- Scan database writes are transactional, but very large folders may create large transactions.
- 扫描写库是事务化的，但超大文件夹可能产生较大的事务。
- Folder input currently accepts a typed absolute path instead of a polished directory picker workflow.
- 当前文件夹输入使用手动填写绝对路径，还不是完整的目录选择器流程。
- No SVN or version-control integration.
- 暂不支持 SVN 或版本控制集成。
- No AI tagging.
- 暂不支持 AI 标签。
- No delete, batch move, or batch rename support.
- 不支持删除、批量移动或批量重命名。
- Release artifact names must be checked before publishing. A v0.2.0 release directory containing a `0.1.0` installer name is considered a release-blocking mismatch.
- Database and thumbnail cache paths are shown for transparency, but v0.2 does not support moving or migrating them from the UI.
- Search results are capped at 2,000 rows in v0.2. Very large libraries may need pagination, virtualized grids, and SQLite FTS in a future release.

## License

MIT License. See [LICENSE](LICENSE).
