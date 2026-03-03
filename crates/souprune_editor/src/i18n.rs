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
sequence-timeline = Sequence Timeline
chapter-inspector = Chapter Inspector
asset-browser = Asset Browser
game-preview = Game Preview

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

# Labels
label-no-sequence = Open a .sequence.ron file to begin editing.
label-selected-chapter = Selected chapter: #{$index}
label-chapters = {$count} chapters
";

const ZH_CN_FTL: &str = "\
sequence-timeline = 序列时间线
chapter-inspector = 章节属性
asset-browser = 资产浏览器
game-preview = 游戏预览

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

# Actions
action-add = 添加
action-delete = 删除
action-copy = 复制
action-paste = 粘贴
action-undo = 撤销
action-redo = 重做

# Labels
label-no-sequence = 请打开 .sequence.ron 文件开始编辑。
label-selected-chapter = 选中章节: #{$index}
label-chapters = {$count} 个章节
";

/// 将编辑器的本地化字符串注册到 bevy_workbench I18n 系统。
pub fn register_editor_i18n(i18n: &mut I18n) {
    i18n.add_custom_source(Locale::En, EN_FTL);
    i18n.add_custom_source(Locale::ZhCn, ZH_CN_FTL);
}
