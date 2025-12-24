# 项目架构重构完成报告

## 重构目标
将项目资源按功能模块清晰分类，分离battle和overworld系统，提高代码可维护性。

## 新目录结构

```
projects/example_mod/
├── battle/                      # 战斗系统专用模块
│   ├── chapters/                # 战斗章节定义
│   │   └── demo.chapter.ron
│   ├── players/                 # 战斗玩家配置
│   │   └── player.battle_player.ron
│   └── ui/                      # 战斗UI布局
│       └── undertale.ui_layout.ron
│
├── overworld/                   # 大地图系统专用模块
│   ├── characters/              # 角色定义和动画
│   │   ├── frisk/
│   │   │   └── animations.animation.ron
│   │   └── frisk.character.ron
│   ├── levels/                  # 关卡地图
│   │   ├── ruins/
│   │   │   ├── ruins_2.tmx
│   │   │   ├── ruins_3.tmx
│   │   │   └── ...
│   │   └── ...
│   ├── players/                 # 大地图玩家配置
│   │   └── player_behavior.ron
│   └── ui/                      # 大地图UI布局
│       └── undertale_backpack.ui_layout.ron
│
├── shared/                      # 系统间共享资源
│   ├── items/                   # 物品系统
│   │   └── basic.item.ron
│   ├── locales/                 # 本地化文件
│   │   ├── en-US/
│   │   └── zh-Hans/
│   └── shaders/                 # 着色器
│       └── ui_solid_fill.wgsl
│
├── textures/                    # 纹理资源（保持原有结构）
│   ├── battle/
│   ├── common/
│   └── overworld/
│
├── code/                        # 代码模块（保持原有结构）
│   └── mod_example/
│
└── mod.toml                     # 模组配置文件
```

## 路径更新列表

### 战斗系统 (Battle)
- `battle/demo.chapter.ron` → `battle/chapters/demo.chapter.ron`
- `battle/player.battle_player.ron` → `battle/players/player.battle_player.ron`
- `ui/battle/undertale.ui_layout.ron` → `battle/ui/undertale.ui_layout.ron`

### 大地图系统 (Overworld)
- `characters/` → `overworld/characters/`
- `levels/` → `overworld/levels/`
- `player/player_behavior.ron` → `overworld/players/player_behavior.ron`
- `ui/undertale_backpack.ui_layout.ron` → `overworld/ui/undertale_backpack.ui_layout.ron`

### 共享资源 (Shared)
- `items/` → `shared/items/`
- `locales/` → `shared/locales/`
- `shaders/` → `shared/shaders/`

## 源代码修改

### 1. 战斗系统路径
**文件**: `crates/souprune/src/app_state/battle/sequencer.rs`
```rust
// 修改前
"battle/demo.chapter.ron"
// 修改后
"battle/chapters/demo.chapter.ron"
```

**文件**: `projects/example_mod/battle/chapters/demo.chapter.ron`
```ron
// 修改前
ui_layout: "ui/battle/undertale.ui_layout.ron"
config_path: "battle/player.battle_player.ron"
// 修改后
ui_layout: "battle/ui/undertale.ui_layout.ron"
config_path: "battle/players/player.battle_player.ron"
```

### 2. 大地图系统路径
**文件**: `crates/souprune/src/app_state/overworld/player/config.rs`
```rust
// 修改前
const PLAYER_BEHAVIOR_PATH: &str = "player/player_behavior.ron";
// 修改后
const PLAYER_BEHAVIOR_PATH: &str = "overworld/players/player_behavior.ron";
```

**文件**: `crates/souprune/src/app_state/overworld/tilemap/systems.rs`
```rust
// 修改前
"levels/ruins/ruins_3.tmx"
// 修改后
"overworld/levels/ruins/ruins_3.tmx"
```

**文件**: `projects/example_mod/overworld/players/player_behavior.ron`
```ron
// 修改前
character_asset: "characters/frisk.character.ron"
// 修改后
character_asset: "overworld/characters/frisk.character.ron"
```

**文件**: `projects/example_mod/overworld/characters/frisk.character.ron`
```ron
// 修改前
animation_config: "characters/frisk/animations.animation.ron"
// 修改后
animation_config: "overworld/characters/frisk/animations.animation.ron"
```

**文件**: `projects/example_mod/overworld/levels/ruins/ruins_3.tmx`
```xml
<!-- 修改前 -->
<property name="backpack_ui" value="ui/undertale_backpack.ui_layout.ron"/>
<!-- 修改后 -->
<property name="backpack_ui" value="overworld/ui/undertale_backpack.ui_layout.ron"/>
```

### 3. 共享资源路径
**文件**: `crates/souprune/src/extra/mortar.rs`
```rust
// 修改前
format!("locales/{}", locale.0)
format!("locales/{}/", locale.0)
// 修改后
format!("shared/locales/{}", locale.0)
format!("shared/locales/{}/", locale.0)
```

**文件**: `crates/souprune/src/core/item.rs`
```rust
// 修改前
asset_server.load_folder("items")
// 修改后
asset_server.load_folder("shared/items")
```

**文件**: `crates/souprune/src/core/ui/shaders.rs`
```rust
// 修改前
format!("projects/{}/shaders/ui_solid_fill.wgsl", config.project.mod_name)
// 修改后
format!("projects/{}/shared/shaders/ui_solid_fill.wgsl", config.project.mod_name)
```

## 优势总结

### 1. 清晰的模块分离
- **Battle**: 所有战斗相关资源集中管理
- **Overworld**: 所有大地图相关资源集中管理
- **Shared**: 系统间共享资源统一存放

### 2. 更好的可维护性
- 新增功能时明确知道文件应该放在哪里
- 减少命名冲突（battle和overworld各有自己的players/ui目录）
- 便于团队协作时的任务分工

### 3. 扩展性强
- 未来添加新系统（如商店、小游戏）可以遵循相同模式
- 每个系统的资源自包含，方便模块化开发

### 4. 资产加载系统兼容
- `config::resolve_path`支持任意深度的子目录查找
- 无需修改核心加载器代码
- 所有路径都能正确解析

## 测试验证
✅ 编译通过  
✅ 大地图场景正常加载  
✅ 战斗场景正常加载  
✅ UI正确渲染  
✅ 战斗玩家正确生成  
✅ 所有资产路径正确解析  

## 迁移指南（为其他模组开发者）

如果你有现有的模组需要迁移到新架构：

1. **备份现有模组**
2. **创建新目录结构**（参考上文）
3. **移动文件到对应目录**
4. **批量更新路径引用**：
   ```bash
   # 示例：更新所有.ron文件中的路径
   find . -name "*.ron" -exec sed -i 's|"characters/|"overworld/characters/|g' {} +
   find . -name "*.ron" -exec sed -i 's|"items/|"shared/items/|g' {} +
   ```
5. **测试运行并修复剩余问题**

## 后续建议

1. 考虑在文档中添加资源放置规范
2. 可以创建模板生成器帮助开发者创建新模组
3. 未来如有更多系统，继续保持这种清晰的分层结构
