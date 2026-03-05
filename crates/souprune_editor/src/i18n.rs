//! # Internationalization (i18n)
//!
//! # 编辑器本地化支持
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides internationalization support for the SoupRune editor.
//! It embeds Fluent (.ftl) strings and registers them via bevy_workbench I18n.
//!
//! 此模块为 SoupRune 编辑器提供本地化支持。
//! 它内嵌 Fluent (.ftl) 字符串并通过 bevy_workbench I18n 注册。

use bevy_workbench::i18n::{I18n, Locale};

const EN_FTL: &str = "\
# Panel titles
panel-sequence-timeline = Sequence Timeline
panel-chapter-inspector = Chapter Inspector
panel-asset-browser = Asset Browser
panel-game-preview = Game Preview
panel-playback = Playback Control
panel-fre = FRE Panel
panel-view-editor = View Editor

# Chapter type names
chapter-spawn-view = Spawn View
chapter-await-fact = Await Fact
chapter-set-view-fact = Set View Fact
chapter-danmaku-performance = Danmaku Performance
chapter-am-performance = AM Performance
chapter-tween-view-element = Tween View Element
chapter-wait = Wait
chapter-sequence = Sequence
chapter-parallel = Parallel
chapter-set-player = Set Player
chapter-set-ui = Set UI
chapter-modify-view-element = Modify View Element
chapter-set-camera = Set Camera
chapter-conditional = Conditional
chapter-fact-switch = Fact Switch
chapter-emit-fact-event = Emit Fact Event
chapter-modify-fact = Modify Fact
chapter-load-fre = Load FRE
chapter-run-sequence = Run Sequence
chapter-load-map = Load Map
chapter-set-bgm = Set BGM
chapter-custom = Custom

# Actions
action-add = Add
action-delete = Delete
action-copy = Copy
action-paste = Paste
action-undo = Undo
action-redo = Redo
action-save = Save
action-open = Open
action-refresh = Refresh
action-create = Create
action-cancel = Cancel
action-find-refs = Find References
action-play-from-here = Play From Here
action-add-subcondition = + Add Subcondition
action-add-modification = + Add Modification
action-add-item = + Add

# Playback
playback-play = ▶ Play
playback-pause = ⏸ Pause
playback-stop = ⏹ Stop
playback-resume = ▶ Resume
playback-step = ⏭ Step
playback-chapter-progress = Chapter {$processed}/{$total}
playback-mode-edit = Edit
playback-mode-playing = ▶ Playing
playback-mode-paused = ⏸ Paused

# Common labels
label-needs-world = Requires World access
label-no-sequence = Open a .sequence.ron file to begin editing.
label-selected-chapter = Selected chapter: #{$index}
label-chapters = {$count} chapters
label-modified = ● Modified
label-unsaved = ● Unsaved
label-no-file-open = No file open
label-no-view-open = No View file open. Double-click a .view.ron in Asset Browser.
label-no-fre-open = No FRE file open
label-preview-not-init = Preview not initialized
label-no-data = No data
label-select-node = Select a node to edit properties
label-node-path-invalid = Node path invalid
label-parse-error = Parse error: {$err}
label-not-initialized = Not initialized
label-no-sequence-open = No sequence open
label-select-chapter = Select a chapter to view properties
label-invalid-chapter = Invalid chapter index
label-chapter-count = {$count} chapters
label-sub-chapters = Sub-chapters: {$count}
label-branch-count = Branches: {$count}
label-param-count = Parameters: {$count}
label-modification-count = Modifications: {$count}
label-no-project = Project directory not found
label-crossref-todo = (Cross-reference will be available in a future version)
label-find-refs-for = Finding references for '{$path}'...
label-no-simulated-facts = No simulated facts (FRE file has no initial facts)
label-no-facts = (no facts)
label-empty = (empty)
label-read-error = Failed to read file: {$err}
label-save-error = Save failed: {$err}
label-count-suffix = {$label}: {$count}

# Property labels
prop-name = Name
prop-tags = Tags
prop-condition = Condition
prop-fact-key = Fact Key
prop-event-id = Event ID
prop-action-type = Action Type
prop-variants = Variants
prop-key = Key
prop-value = Value
prop-branch = Branch {$index}
prop-duration-sec = Duration (sec)
prop-view-layout-file = View layout file
prop-bindings = Bindings
prop-perf-file = Performance file
prop-position = Position
prop-amproj-file = AMPROJ file
prop-am-config = AM Config
prop-data = Data
prop-fre-files = FRE Files
prop-seq-path = Sequence path
prop-dynamic-path-fact = Dynamic path Fact
prop-map-path = Map path
prop-bgm-path = BGM path
prop-fade-in-sec = Fade in (sec)
prop-params = Parameters
prop-texture-path = Texture path
prop-visibility = Visibility
prop-width = Width
prop-height = Height
prop-scale-x = ScaleX
prop-scale-y = ScaleY
prop-scale-z = ScaleZ
prop-modes = Modes
prop-config-path = Config path
prop-duration = Duration
prop-intensity = Intensity
prop-path-id = Path/ID
prop-element-id = Element ID
prop-content = Content
prop-variable-name = Variable name
prop-anim-clip = Animation clip

# Tree context menu
tree-add-child = Add Child Node
tree-move-up = Move Up
tree-move-down = Move Down

# View editor
view-node-tree = Node Tree
view-add-root = Add root node
view-properties = Properties
view-basics = Basics
view-width = Width:
view-height = Height:
view-data-requirements = Data Requirements
view-initial-facts = Initial Facts
view-repeat = Repeat
view-color = Color
view-font = Font

# View preview
preview-play = Play
preview-stop = Stop
preview-reset = Reset
preview-zoom = Zoom: {$percent}%
preview-input-active = Input Active

# FRE panel
fre-filter = Filter:
fre-db-unavailable = LayeredFactDatabase not available.
fre-global-layer = Global Layer
fre-local-layer = Local Layer
fre-add-fact = Add New Fact
fre-key = Key:
fre-value = Value:
fre-type = Type:
fre-layer = Layer:
fre-registry-unavailable = LayeredRuleRegistry not available.
fre-rules-total = Total: {$total} (Global: {$global}, Local: {$local})
fre-no-rules = No rules registered.
fre-global-rules = Global Rules ({$count})
fre-local-rules = Local Rules ({$count})
fre-trigger = Trigger:
fre-event-tracking-not-init = Event tracking not initialized.
fre-recent-events = Recent events: {$count}
fre-no-events = No events recorded yet.
fre-current = Current:
fre-state-config-not-loaded = StateConfig not loaded
fre-rules-count = FRE Rules ({$count})
fre-fact-simulator = Fact Simulator
fre-fact-simulator-live = Fact Simulator (Live)
fre-facts-count = Facts ({$count})
fre-priority = Priority: {$value}
fre-conditions = Conditions:
fre-actions = Actions:
fre-modifications = Modifications:
fre-outputs = Outputs: {$value}
fre-tabs-facts = Facts
fre-tabs-rules = Rules
fre-tabs-events = Events
fre-tabs-states = States

# Asset browser
browser-new-sequence = New Sequence
browser-new-view = New View
browser-new-rule = New Rule
browser-new-folder = New Folder
browser-refresh-tree = Refresh file tree
browser-search-hint = Search...
browser-new-file = New File
browser-name = Name:
browser-directory = Directory: {$path}

# Chapter inspector
inspector-use-local-facts = Use local Facts
inspector-wait-completion = Wait for completion
inspector-default-branch = Default branch
inspector-generate-collision = Generate collision
inspector-process-objects = Process objects
inspector-setup-camera-bounds = Setup camera bounds
inspector-selector = Selector:
inspector-modify-type = Modify type: {$label}
inspector-specify-position = Specify position
inspector-position = Position:
inspector-active = Active
inspector-follow-player = Follow player

# Sequence timeline
timeline-open-sequence = Open sequence file
timeline-save = Save

# Widgets
widget-browse-file = Browse file
widget-static = Static
widget-expression = Expression
widget-subconditions = Sub-conditions: {$count}

# Undo descriptions
undo-insert-chapter = Insert chapter
undo-remove-chapter = Remove chapter
undo-move-chapter = Move chapter
undo-modify-chapter = Modify chapter

# Chapter palette categories
palette-flow = Flow Control
palette-scene = Scene
palette-view = View
palette-logic = Logic
palette-combat = Combat
palette-audio = Audio
palette-extension = Extension

# Asset browser categories
browser-cat-sequence = Sequence
browser-cat-view = View
browser-cat-rule = Rule
browser-cat-performance = Performance
browser-cat-config = Config
browser-cat-other = Other
browser-cat-directory = Directory

# Preview
label-preview-init = Preview (initializing...)

# File picker
picker-all-files = All files

# Zoom prefix
prop-zoom-prefix = Zoom: 
";

const ZH_CN_FTL: &str = "\
# 面板标题
panel-sequence-timeline = 序列时间线
panel-chapter-inspector = 章节属性
panel-asset-browser = 资产浏览器
panel-game-preview = 游戏预览
panel-playback = 回放控制
panel-fre = FRE 面板
panel-view-editor = 视图编辑器

# Chapter type names
chapter-spawn-view = 生成视图
chapter-await-fact = 等待条件
chapter-set-view-fact = 设置视图变量
chapter-danmaku-performance = 弹幕演出
chapter-am-performance = AM 动画
chapter-tween-view-element = 视图元素过渡
chapter-wait = 等待
chapter-sequence = 序列
chapter-parallel = 并行
chapter-set-player = 设置玩家
chapter-set-ui = 设置 UI
chapter-modify-view-element = 修改视图元素
chapter-set-camera = 设置摄像机
chapter-conditional = 条件分支
chapter-fact-switch = 事实分支
chapter-emit-fact-event = 发送事件
chapter-modify-fact = 修改事实
chapter-load-fre = 加载 FRE
chapter-run-sequence = 运行序列
chapter-load-map = 加载地图
chapter-set-bgm = 设置 BGM
chapter-custom = 自定义

# 操作
action-add = 添加
action-delete = 删除
action-copy = 复制
action-paste = 粘贴
action-undo = 撤销
action-redo = 重做
action-save = 保存
action-open = 打开
action-refresh = 刷新
action-create = 创建
action-cancel = 取消
action-find-refs = 查找引用
action-play-from-here = 从这里播放
action-add-subcondition = + 添加子条件
action-add-modification = + 添加修改
action-add-item = + 添加

# 回放
playback-play = ▶ 播放
playback-pause = ⏸ 暂停
playback-stop = ⏹ 停止
playback-resume = ▶ 继续
playback-step = ⏭ 单步
playback-chapter-progress = 章节 {$processed}/{$total}
playback-mode-edit = 编辑
playback-mode-playing = ▶ 播放中
playback-mode-paused = ⏸ 已暂停

# 通用标签
label-needs-world = 需要 World 访问权限
label-no-sequence = 请打开 .sequence.ron 文件开始编辑。
label-selected-chapter = 选中章节: #{$index}
label-chapters = {$count} 个章节
label-modified = ● 已修改
label-unsaved = ● 未保存
label-no-file-open = 未打开文件
label-no-view-open = 在 Asset Browser 中双击 .view.ron 文件来编辑
label-no-fre-open = 未打开任何 FRE 文件
label-preview-not-init = Preview 未初始化
label-no-data = 无数据
label-select-node = 选择节点以编辑属性
label-node-path-invalid = 节点路径无效
label-parse-error = 解析错误: {$err}
label-not-initialized = 未初始化
label-no-sequence-open = 未打开序列
label-select-chapter = 选择一个章节以查看属性
label-invalid-chapter = 无效的章节索引
label-chapter-count = {$count} 个章节
label-sub-chapters = 子章节数: {$count}
label-branch-count = 分支数: {$count}
label-param-count = 参数: {$count} 个
label-modification-count = 修改: {$count} 个
label-no-project = 未找到项目目录
label-crossref-todo = (交叉引用功能将在后续版本实现)
label-find-refs-for = 查找 '{$path}' 的引用…
label-no-simulated-facts = 无模拟 Fact（FRE 文件未定义初始 facts）
label-no-facts = （无数据）
label-empty = （空）
label-read-error = 读取文件失败: {$err}
label-save-error = 保存失败: {$err}
label-count-suffix = {$label}: {$count}

# 属性标签
prop-name = 名称
prop-tags = 标签
prop-condition = 条件表达式
prop-fact-key = Fact 键
prop-event-id = 事件 ID
prop-action-type = 动作类型
prop-variants = 变体
prop-key = 键
prop-value = 值
prop-branch = 分支 {$index}
prop-duration-sec = 持续时间 (秒)
prop-view-layout-file = 视图布局文件
prop-bindings = 绑定
prop-perf-file = 演出文件
prop-position = 位置
prop-amproj-file = AMPROJ 文件
prop-am-config = AM 配置
prop-data = 数据
prop-fre-files = FRE 文件
prop-seq-path = 序列路径
prop-dynamic-path-fact = 动态路径 Fact
prop-map-path = 地图路径
prop-bgm-path = BGM 路径
prop-fade-in-sec = 淡入时间 (秒)
prop-params = 参数
prop-texture-path = 贴图路径
prop-visibility = 可见性
prop-width = 宽度
prop-height = 高度
prop-scale-x = 缩放X
prop-scale-y = 缩放Y
prop-scale-z = 缩放Z
prop-modes = 模式
prop-config-path = 配置路径
prop-duration = 持续时间
prop-intensity = 强度
prop-path-id = 路径/ID
prop-element-id = 元素 ID
prop-content = 内容
prop-variable-name = 变量名
prop-anim-clip = 动画片段

# 节点树右键菜单
tree-add-child = 添加子节点
tree-move-up = 上移
tree-move-down = 下移

# 视图编辑器
view-node-tree = 节点树
view-add-root = 添加根节点
view-properties = 属性
view-basics = 基本
view-width = 宽:
view-height = 高:
view-data-requirements = 数据依赖
view-initial-facts = 初始 Facts
view-repeat = 重复
view-color = 颜色
view-font = 字体

# 视图预览
preview-play = Play
preview-stop = Stop
preview-reset = Reset
preview-zoom = Zoom: {$percent}%
preview-input-active = 输入激活

# FRE 面板
fre-filter = 筛选:
fre-db-unavailable = LayeredFactDatabase 不可用。
fre-global-layer = 全局层
fre-local-layer = 局部层
fre-add-fact = 添加 Fact
fre-key = 键:
fre-value = 值:
fre-type = 类型:
fre-layer = 层:
fre-registry-unavailable = LayeredRuleRegistry 不可用。
fre-rules-total = 共 {$total} 条（全局: {$global}, 局部: {$local}）
fre-no-rules = 未注册规则。
fre-global-rules = 全局规则 ({$count})
fre-local-rules = 局部规则 ({$count})
fre-trigger = 触发器:
fre-event-tracking-not-init = 事件跟踪未初始化。
fre-recent-events = 最近事件: {$count}
fre-no-events = 暂无事件记录。
fre-current = 当前:
fre-state-config-not-loaded = StateConfig 未加载
fre-rules-count = FRE 规则 ({$count})
fre-fact-simulator = Fact 模拟器
fre-fact-simulator-live = Fact 模拟器（实时）
fre-facts-count = Facts ({$count})
fre-priority = 优先级: {$value}
fre-conditions = 条件:
fre-actions = 动作:
fre-modifications = 修改:
fre-outputs = 输出: {$value}
fre-tabs-facts = Facts
fre-tabs-rules = 规则
fre-tabs-events = 事件
fre-tabs-states = 状态

# 资产浏览器
browser-new-sequence = 新建序列
browser-new-view = 新建视图
browser-new-rule = 新建规则
browser-new-folder = 新建目录
browser-refresh-tree = 刷新文件树
browser-search-hint = 搜索…
browser-new-file = 新建文件
browser-name = 名称:
browser-directory = 目录: {$path}

# 章节属性
inspector-use-local-facts = 使用局部 Facts
inspector-wait-completion = 等待完成
inspector-default-branch = 默认分支
inspector-generate-collision = 生成碰撞
inspector-process-objects = 处理对象
inspector-setup-camera-bounds = 设置摄像机边界
inspector-selector = 选择器:
inspector-modify-type = 修改类型: {$label}
inspector-specify-position = 指定位置
inspector-position = 位置:
inspector-active = 激活
inspector-follow-player = 跟随玩家

# 序列时间线
timeline-open-sequence = 打开序列文件
timeline-save = 保存

# 控件
widget-browse-file = 浏览文件
widget-static = 静态
widget-expression = 表达式
widget-subconditions = 子条件: {$count}

# 撤销描述
undo-insert-chapter = 插入章节
undo-remove-chapter = 删除章节
undo-move-chapter = 移动章节
undo-modify-chapter = 修改章节属性

# 章节面板分类
palette-flow = 流程控制
palette-scene = 场景
palette-view = 界面
palette-logic = 逻辑
palette-combat = 战斗
palette-audio = 音频
palette-extension = 扩展

# 资产浏览器分类
browser-cat-sequence = 序列
browser-cat-view = 视图
browser-cat-rule = 规则
browser-cat-performance = 弹幕
browser-cat-config = 配置
browser-cat-other = 其他
browser-cat-directory = 目录

# 预览
label-preview-init = Preview (初始化中...)

# 文件选择器
picker-all-files = 所有文件

# 缩放前缀
prop-zoom-prefix = 缩放: 
";

/// 将编辑器的本地化字符串注册到 bevy_workbench I18n 系统。
pub fn register_editor_i18n(i18n: &mut I18n) {
    i18n.add_custom_source(Locale::En, EN_FTL);
    i18n.add_custom_source(Locale::ZhCn, ZH_CN_FTL);
}

/// Translate a message ID using the I18n resource from the given World.
pub fn t(world: &bevy::prelude::World, id: &str) -> String {
    world
        .get_resource::<I18n>()
        .map_or_else(|| id.to_string(), |i| i.t(id))
}

/// Translate a message ID with arguments.
pub fn t_args(
    world: &bevy::prelude::World,
    id: &str,
    args: &bevy_workbench::i18n::FluentArgs,
) -> String {
    world
        .get_resource::<I18n>()
        .map_or_else(|| id.to_string(), |i| i.t_args(id, args))
}
