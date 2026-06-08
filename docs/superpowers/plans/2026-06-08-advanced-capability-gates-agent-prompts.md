# v0.4 Advanced Capability Gates Agent Prompts

用途：把 `2026-06-08-advanced-capability-gates.md` 拆成可直接交给其他 agent 执行的独立提示词。

通用约束：

- 默认工作目录是 `I:\GameResManger`。
- 回复用户使用中文。
- 开始前先读：
  - `I:\GameResManger\AGENTS.md`
  - `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md`
  - `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
  - `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`
- 运行 `git status --short`，只处理本任务相关文件。
- 禁止删除、移动、重命名、修改原始素材文件。
- 禁止批量删除命令：`del /s`、`rd /s`、`rmdir /s`、`Remove-Item -Recurse`、`rm -rf`。
- v0.4 只做文档门禁，不实现 AI 自动打标、图集工具、完整 3D/Spine/音频预览、云同步或团队后台。

---

## Prompt 1: Task 50 - 创建高级能力门禁研究文档

```text
你是 Codex 子代理，请执行 `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md` 中的 Task 50。

目标：
在策划仓库中创建 v0.4 高级能力门禁研究文档，明确 AI 自动打标、图集工具、完整 3D/Spine/音频预览、云/团队后台这些高风险能力只进入研究门禁，不进入实现。

必须先读：
- `I:\GameResManger\AGENTS.md`
- `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md`
- `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`

只允许创建/修改：
- `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`

执行步骤：
1. 在 `I:\GameResManger` 运行 `git status --short`，记录但不要处理无关改动。
2. 确保目录 `G:\oklai999的策划仓库\游戏资源管理器\研究记录` 存在。
3. 创建 `2026-06-08-advanced-capability-gates.md`。
4. 文档必须包含这些章节：
   - Purpose
   - Global Rules
   - AI Auto Tagging
   - Texture Atlas Tools
   - Full 3D, Spine, And Audio Workstation Preview
   - Cloud Sync Or Team Backend
   - v0.4 Decision
5. 每个高风险能力都要写清：
   - Current status
   - Allowed research questions
   - Entry criteria before implementation
   - Minimum prototype shape after approval
6. 运行：
   `Select-String -LiteralPath 'G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md' -Pattern 'Current status|Entry criteria|No advanced capability'`
7. 运行：
   `git -C 'G:\oklai999的策划仓库\游戏资源管理器' status --short`

禁止事项：
- 不要实现任何高级能力。
- 不要修改 `I:\GameResManger` 的源代码。
- 不要删除、移动、重命名、修改任何原始素材文件。
- 不要使用任何批量删除命令。

完成后回复：
- 创建的文件路径。
- 验证命令结果摘要。
- 策划仓库是否是 Git 仓库，以及 status 摘要。
- 明确说明没有实现高级能力、没有触碰源素材。
```

---

## Prompt 2: Task 51 - 更新 Sharp Stock 路线里程碑

```text
你是 Codex 子代理，请执行 `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md` 中的 Task 51。

目标：
在里程碑文档中把 `v0.4 Advanced Capability Gates` 从 `Planned` 更新为 `Documented`，并追加完成记录。

前置条件：
确认下面文件已经存在：
`G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`

必须先读：
- `I:\GameResManger\AGENTS.md`
- `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md`
- `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

只允许修改：
- `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

执行步骤：
1. 在 `I:\GameResManger` 运行 `git status --short`，记录但不要处理无关改动。
2. 检查研究文档是否存在；如果不存在，停止并回复前置条件缺失。
3. 在 Milestone Board 中，将 `v0.4 Advanced Capability Gates` 状态改为 `Documented`。
4. 将该行 Required Verification 指向：
   `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`
5. 将 Completion Notes 改为：
   `Decision gates documented; no advanced capability implementation started.`
6. 在 Completion Log 中追加：
   `| 2026-06-08 | v0.4 Advanced Capability Gates | Documented | Review G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md | AI auto-tagging, atlas tools, rich previews, and cloud/team backend remain gated behind explicit safety criteria. |`
7. 运行：
   `Select-String -LiteralPath 'G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md' -Pattern '2026-06-08-advanced-capability-gates|Documented'`
8. 运行：
   `git -C 'G:\oklai999的策划仓库\游戏资源管理器' status --short`

禁止事项：
- 不要修改应用源码。
- 不要实现任何高级能力。
- 不要回滚或清理无关工作区改动。
- 不要删除、移动、重命名、修改任何原始素材文件。

完成后回复：
- 修改的文件路径。
- v0.4 当前状态。
- 验证命令结果摘要。
- 明确说明没有实现高级能力、没有触碰源素材。
```

---

## Prompt 3: Task 52 - 添加高级能力门禁烟测清单

```text
你是 Codex 子代理，请执行 `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md` 中的 Task 52。

目标：
在项目烟测文档中新增“高级能力门禁烟测复核”，用于确认 v0.4 阶段没有把高风险功能偷偷暴露到应用界面里。

必须先读：
- `I:\GameResManger\AGENTS.md`
- `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md`
- `I:\GameResManger\tests\smoke\README.md`

只允许修改：
- `I:\GameResManger\tests\smoke\README.md`

执行步骤：
1. 在 `I:\GameResManger` 运行 `git status --short`，记录但不要处理无关改动。
2. 在 `tests\smoke\README.md` 末尾追加章节：

```markdown
## Advanced Capability Gate Smoke Review

- Confirm the app does not show AI auto-tagging actions in the main workbench.
- Confirm the app does not show batch delete, batch move, or batch rename actions.
- Confirm the app does not show texture atlas export actions.
- Confirm the app does not require network access for search, preview, tags, favorites, collections, notes, recent activity, or path copying.
- Confirm non-previewable 3D, Spine, PSD, audio, and video files still show safe actions: open file, open containing folder, and copy path.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this review.
```

3. 运行：
   `git diff -- tests/smoke/README.md`
4. 确认 diff 只新增 `Advanced Capability Gate Smoke Review` 章节。
5. 不要提交，除非用户明确要求提交。

禁止事项：
- 不要修改源码。
- 不要修改策划仓库。
- 不要实现任何高级能力。
- 不要删除、移动、重命名、修改任何原始素材文件。
- 不要暂存或回滚无关删除项、release 文件、未跟踪文件。

完成后回复：
- 修改的文件路径。
- diff 摘要。
- 明确说明没有提交、没有实现高级能力、没有触碰源素材。
```

---

## Prompt 4: Task 53 - v0.4 最终验证

```text
你是 Codex 子代理，请执行 `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md` 中的 Task 53。

目标：
对 v0.4 文档门禁阶段做最终验证，确认文档改动没有破坏应用构建和测试，并确认工作区只包含预期文档改动。

必须先读：
- `I:\GameResManger\AGENTS.md`
- `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-capability-gates.md`
- `I:\GameResManger\tests\smoke\README.md`
- `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

不要修改文件，除非只是记录验证结果且用户明确要求。

执行步骤：
1. 在 `I:\GameResManger` 运行：
   `git status --short`
2. 运行前端验证：
   `npm test`
   `npm run build`
3. 运行后端验证：
   在 `I:\GameResManger\src-tauri` 下运行：
   `cargo test`
   `cargo check`
4. 如果 Rust 因本机 MSVC 或 Windows SDK 失败，只记录环境错误，不要改业务代码规避。
5. 运行：
   `git -C 'G:\oklai999的策划仓库\游戏资源管理器' status --short`
6. 汇总验证结果。

禁止事项：
- 不要运行 `npm run tauri dev`，除非用户明确要求 GUI 烟测。
- 不要提交。
- 不要实现任何高级能力。
- 不要回滚或清理无关工作区改动。
- 不要删除、移动、重命名、修改任何原始素材文件。
- 不要使用任何批量删除命令。

完成后回复：
- `npm test` 结果。
- `npm run build` 结果。
- `cargo test` 结果。
- `cargo check` 结果。
- `I:\GameResManger` 工作区状态摘要。
- 策划仓库状态摘要。
- 明确说明 v0.4 仍是文档门禁阶段，没有实现高级能力，没有触碰源素材。
```

---

## 推荐分发顺序

1. 先把 Prompt 1 交给一个 agent，创建研究文档。
2. Prompt 1 完成后，把 Prompt 2 交给另一个 agent，更新里程碑。
3. Prompt 3 可以并行执行，只改项目内烟测文档。
4. Prompt 1、2、3 都完成后，再执行 Prompt 4 做最终验证。

## 注意

- Prompt 1 和 Prompt 2 会写入 `G:\oklai999的策划仓库\游戏资源管理器`，如果当前执行环境没有该路径写权限，需要请求用户授权或改由用户在本机执行。
- Prompt 3 和 Prompt 4 只涉及 `I:\GameResManger`，但仍必须避开已有无关删除项和 release 目录。
