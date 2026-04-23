//! Stores the built-in Simplified Chinese Fluent messages for the Souprune editor UI.
//!
//! 保存 Souprune 编辑器界面使用的内置简体中文 Fluent 文案。
//!
//! Like the English bundle, this locale bundle is a compiled-in source rather
//! than executable logic. It provides the editor with a shipped Simplified
//! Chinese translation set so the UI can be localized immediately in local
//! builds and distribution packages.
//!
//! 和英文语言包一样，这份语言包是编译进程序的本地化数据源，而不是逻辑代码。
//! 它为编辑器提供一整套随程序分发的简体中文翻译，让本地开发和分发包都能在
//! 不依赖额外资源下载的情况下直接完成界面本地化。

pub const ZH_CN_FTL: &str = "\
# 面板标题
panel-sequence-timeline = 序列时间线
panel-chapter-inspector = 章节属性
panel-asset-browser = 资源浏览器
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

# 资源浏览器
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

# 资源浏览器分类
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
