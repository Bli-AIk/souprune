# Deltarune Preset Overworld 背包设计

> 给 agentic worker：本设计通过 Superpowers brainstorming 流程产出。进入代码或仓库变更前，必须先基于本文进入 implementation-planning 流程。

## 目标

创建新的 `deltarune_preset` 前置 mod，作为与 `undertale_preset` 平级的独立 preset 接入 SoupRune 主仓库。第一阶段只实现基础可用的 Deltarune 风格 overworld 背包/菜单，使用户能在本地 smoke mod 中进入菜单、浏览 party 状态、切换 ITEM / STORAGE / KEYITEM 三类列表，并看到接近 DR 原作的坐标、排布和交互反馈。

本阶段优先保证架构、数据边界和验收路径正确。具体数值可以在原作坐标基础上小幅微调，但坐标系策略必须支持后续直接搬运 Deltarune 原作 UI 坐标。

## 当前确认决策

- `deltarune_preset` 与 `undertale_preset` 平级，不依赖 `undertale_preset`。
- 主仓库以 git submodule 方式接入 `projects/deltarune_preset`。
- 新远端仓库由 `gh` 创建，目标仓库名为 `Bli-AIk/souprune_deltarune_preset`。
- 本轮只做 overworld 背包/菜单，不做战斗界面。
- UI 目标是适配 DR 原作表现，不做对比图，不使用视觉 companion。
- View 可调整轴点和坐标系，所以 Deltarune 菜单 View 使用 640x480、左上原点、Y 向下的 GMS 风格坐标。
- 为验收创建本地私有 `projects/deltarune_smoke_test`，可从 `projects/mad_dummy_example` 的最小结构复制并改前置依赖。该 smoke mod 只用于本机验收，不作为主仓库提交内容。

## 非范围

本阶段不实现以下内容：

- Deltarune 战斗界面、多角色战斗指令队列、ACT / MAGIC / DEFEND / ITEM 战斗闭环。
- 装备、能力值计算、TP、Magic、SAVE、SHOP、Dark World / Light World 完整切换。
- 完整道具效果执行。ITEM / KEYITEM 只需要能展示和选择；不可用项目播放或触发不可选择反馈即可。
- 将 `mad_dummy_example` 迁移为公开 DR 示例项目。当前只需要本地私有 smoke fixture。
- 运行时支持“一个 mod 同时依赖两个前置”的通用机制。未来如需支持再单独设计。

## 参考来源

本地原生参考：

- `/home/aik/Documents/ut`
- `/home/aik/Documents/dr`

本阶段重点使用的 DR 原作脚本：

- `/home/aik/Documents/dr/code1/gml_Object_obj_darkcontroller_Create_0.gml`
- `/home/aik/Documents/dr/code1/gml_Object_obj_darkcontroller_Draw_0.gml`
- `/home/aik/Documents/dr/code1/gml_Object_obj_darkcontroller_Step_0.gml`
- `/home/aik/Documents/dr/code1/gml_GlobalScript_scr_charbox.gml`
- `/home/aik/Documents/dr/code1/gml_GlobalScript_scr_darkbox.gml`

外部参考：

- Kristal 核心：`https://github.com/KristalTeam/Kristal/`
- Kristal Shadow 文档：`https://github.com/KristalTeam/Shadow`

Kristal 仅作为数据命名、party 概念和菜单结构参考；第一阶段实现应优先贴合 SoupRune 现有 `undertale_preset` 架构，避免引入超出当前需求的新抽象。

## 仓库接入设计

`deltarune_preset` 使用与 `undertale_preset` 相同的双 crate / content 生成模式：

- `projects/deltarune_preset/mod.toml` 声明前置 mod 元信息。
- `projects/deltarune_preset/content/` 存放 RON、Mortar、WASM content crate 和生成脚本。
- `projects/deltarune_preset/runtime/` 只有在 content crate 无法表达菜单状态或 View 坐标配置时才创建；第一阶段默认不创建 runtime。

主仓库变更：

- `.gitmodules` 增加 `projects/deltarune_preset` 指向 `https://github.com/Bli-AIk/souprune_deltarune_preset.git`。
- `.gitignore` 保留 `projects/*/` 作为本地用户项目默认忽略规则，并增加 `!projects/deltarune_preset/` 白名单。
- 不提交 `projects/deltarune_smoke_test`，它继续被 `projects/*/` 忽略。

远端仓库创建顺序：

1. 使用 `gh repo create Bli-AIk/souprune_deltarune_preset` 创建仓库。
2. 从 `projects/undertale_preset` 的结构复制出初始文件。
3. 移除 UT 专属命名和内容，替换为 DR overworld 背包所需的最小实现。
4. 推送 `deltarune_preset` 仓库后，再在主仓库添加 submodule gitlink。

## 坐标与 View 策略

Deltarune overworld 背包 View 使用 DR 原作屏幕空间：

- 画布语义尺寸：640x480。
- 原点：左上角。
- Y 轴：向下增加。
- UI 位置以 DR 原作脚本中的 `xx`、`yy`、`tp`、`xchunk` 等计算为主要来源。

SoupRune 侧需要在 View 层明确支持该坐标空间，而不是把 DR 坐标反推到 `undertale_preset` 的坐标习惯。实现上可通过新的 view axis / coordinate transform 配置，让 DR View 的绘制节点直接表达原作数值。

第一阶段可采用稳定菜单根节点：

- 菜单 root 覆盖 640x480。
- 顶部按钮区沿用 DR 原作的横向布局。
- party charbox 沿用 `scr_charbox` 的 1 / 2 / 3 人 `xchunk` 规则。
- 列表区沿用 ITEM / STORAGE / KEYITEM 的两列、最多 12 行为基础。

## Overworld 背包 UI 组成

菜单包含以下区域：

- 顶部菜单条：显示 ITEM、EQUIP、TALK、TECH、CONFIG 图标按钮和金钱文本。第一阶段只有 ITEM 可进入，其余可以显示但不可选择或作为占位项。
- Party 状态区：显示 1 到 3 名 party 成员的头像、姓名、HP 数字、HP 条和基础状态。初始 smoke 数据建议使用 Kris、Susie、Ralsei。
- ITEM 子菜单：包含 ITEM、STORAGE、KEYITEM 三段选择。
- ITEM / STORAGE 列表：两列网格，最多显示 12 个条目，支持上下左右移动。条目为空时显示 DR 风格的空列表反馈。
- KEYITEM 列表：同样使用两列网格，确认不可用项目时给不可选择反馈。
- 文本框 / 边框：优先复用 `scr_darkbox` 观察到的四角与边框拉伸语义；初始可用 FRE/View 现有 box 能力表达。

应优先使用 DR 资源名和布局语义组织资产引用，例如：

- `spr_darkitembt`
- `spr_darkequipbt`
- `spr_darktalkbt`
- `spr_darktechbt`
- `spr_darkconfigbt`
- `spr_darkmenudesc`
- `spr_dmenu_captions`
- `spr_dmenu_items`
- `spr_dmenu_equip`
- `spr_headkris`
- `spr_headsusie`
- `spr_headralsei`
- `spr_bnamekris`
- `spr_bnamesusie`
- `spr_bnameralsei`
- `spr_hpname`
- `spr_hpslash`
- `spr_heart`
- `spr_heart_harrows`

第一阶段必须把上述 DR 资源名作为稳定的资源键写入数据或 View 定义。若现有 asset pipeline 暂时无法直接导入全部原作图像，smoke 验收可用同名占位资源或已有文本/box primitives，但坐标、资源键和交互状态不得因此改变。

## 菜单交互设计

第一阶段菜单状态最小集合：

- `Closed`：菜单关闭。
- `TopMenu`：顶部菜单焦点。初始焦点为 ITEM。
- `ItemCategory`：ITEM / STORAGE / KEYITEM 三段选择。
- `ItemList`：浏览 ITEM。
- `StorageList`：浏览 STORAGE。
- `KeyItemList`：浏览 KEYITEM。

按键行为按 DR 原作 `obj_darkcontroller` 的菜单语义实现：

- 打开菜单后进入 `TopMenu`，焦点在 ITEM。
- 在 `TopMenu` 按确认进入 `ItemCategory`。
- 在 `ItemCategory` 中，Left / Right 在三段分类间循环。
- 在 `ItemCategory` 中，确认进入当前分类列表。
- 在列表中，Left / Right 在两列间移动；Up / Down 按两列网格上移或下移。
- 取消从列表返回 `ItemCategory`，从 `ItemCategory` 返回 `TopMenu`，从 `TopMenu` 关闭菜单。
- 选择不可使用项目时，不执行效果，只触发不可选择反馈。

UI 必须由菜单状态和 facts 驱动，不能把当前选择硬编码进绘制逻辑。

## 数据与 facts 设计

`deltarune_preset` 第一阶段需要定义可被 smoke mod 初始化和菜单读取的最小 DR 数据：

- Party slots：当前队伍成员顺序，最多 3 人。
- Party member state：角色 id、显示名、头像资源、姓名资源、当前 HP、最大 HP、状态文本或状态枚举。
- Inventory：overworld ITEM 条目列表。
- Storage：STORAGE 条目列表。
- Key items：KEYITEM 条目列表。
- Money：当前金钱数量。
- Menu state：当前菜单层级、顶部菜单焦点、分类焦点、列表游标。

推荐命名保持 DR 语义，但避免把核心概念放入 SoupRune core。party、item、menu 都属于 preset 或 user content 层。

初始 smoke 数据：

- Party：Kris、Susie、Ralsei。
- Money：可使用固定测试值。
- ITEM：至少 3 个条目，覆盖两列移动。
- STORAGE：至少 1 个条目。
- KEYITEM：至少 1 个不可使用条目。

## 与 `undertale_preset` 的关系

实现应参考 `undertale_preset` 的模块边界，而不是复制 UT 行为：

- 可复用 content crate 架构、build.rs 生成方式、mod.toml 组织方式。
- 可复用 overworld rules / view 的目录组织风格。
- 不复用 UT 单角色背包坐标、HP 展示逻辑或菜单状态机。
- 不让 `deltarune_preset` 依赖 `undertale_preset`，避免未来 DR 与 UT preset 生命周期互相绑定。

## 本地 smoke 验收夹具

创建 `projects/deltarune_smoke_test` 作为本地私有 mod：

- 从 `projects/mad_dummy_example` 复制最小可运行结构。
- `mod.toml` 改为依赖 `deltarune_preset`。
- 初始化一段最小 overworld 场景或脚本，使菜单能被打开。
- 写入 DR 初始 facts：party、money、ITEM、STORAGE、KEYITEM。
- 不提交该目录到主仓库。

验收者可以通过这个 smoke mod 验证 `deltarune_preset` 作为前置 mod 的行为，而不是直接运行 preset 本身。

## 验收标准

仓库与结构：

- `Bli-AIk/souprune_deltarune_preset` 仓库存在并可访问。
- 主仓库 `.gitmodules` 包含 `projects/deltarune_preset`。
- `git submodule update --init --recursive` 能拉取 `deltarune_preset`。
- `projects/deltarune_preset/mod.toml` 声明 `name = "deltarune_preset"`。
- `projects/deltarune_smoke_test` 存在于本地但不出现在主仓库 staged/tracked 文件中。

功能：

- 运行 smoke mod 后能打开 Deltarune overworld 菜单。
- 菜单以 640x480、左上原点、Y 向下的坐标表达 UI。
- Party 状态区能显示 1 到 3 名角色，并且三人布局符合 DR `scr_charbox` 的横向分块语义。
- ITEM / STORAGE / KEYITEM 分类能循环切换。
- ITEM / STORAGE / KEYITEM 列表能以两列方式移动游标。
- 选择不可使用 key item 不崩溃，并给出不可选择反馈。

质量检查：

- `cargo fmt --all` 通过。
- `cargo clippy --workspace --all-targets -D warnings` 通过。
- 与项目现有 smoke / packaging 流程相关的检查通过；仅当检查本身已有 optional project submodule 跳过策略时，缺少非本阶段项目可按既有策略跳过。

## 实施顺序建议

1. 创建并初始化 `souprune_deltarune_preset` 远端仓库。
2. 复制 `undertale_preset` 的结构作为初始骨架，替换命名为 `deltarune_preset`。
3. 建立 DR overworld 背包 facts 与菜单状态机。
4. 建立 640x480 左上原点 View 配置和 DR 菜单绘制结构。
5. 添加最小 ITEM / STORAGE / KEYITEM 交互。
6. 在主仓库添加 submodule 和 `.gitignore` 白名单。
7. 创建本地 `deltarune_smoke_test` 并跑通验收。
8. 运行格式化和 clippy。

## 后续扩展点

第一阶段完成后，可独立设计以下扩展：

- 完整 DR battle View 和多角色战斗指令分配。
- 更完整的 party system，包括装备、能力、状态异常、死亡/倒地状态和角色入队离队。
- ITEM、KEYITEM、STORAGE 的真实效果执行与脚本联动。
- 允许一个 user mod 同时依赖多个 preset / library mod 的通用依赖解析。
- 从 DR 解包资源到 SoupRune asset pipeline 的正式导入规范。

## 主要风险

- 直接复制 `undertale_preset` 代码可能残留 UT 单角色假设。实施时需要先删除或替换这些假设，再接 DR 状态。
- 如果 View 坐标转换不先明确，后续会在每个 UI 元素中累积手工偏移。第一阶段必须把 DR 坐标空间作为 View 级能力处理。
- 本地 smoke mod 不提交主仓库，验收说明必须清楚，否则其他开发者会误以为缺少示例。公开示例项目应在本阶段之后单独设计。
