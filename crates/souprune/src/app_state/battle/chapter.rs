//! # chapter.rs
//!
//! # chapter.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Chapter is the minimal unit of the linear sequence in the battle system.
//! It is an enum type representing different events in the battle.
//! For example, player choices, bullet pattern generation, dialogues, and nested Chapters.
//! Chapter itself does not contain definitions or implementations of bullet patterns or UI.
//!
//! Chapter 是 战斗系统中线性序列的最小单位。
//! 它是一个枚举类型，表示战斗中的不同事件。
//! 例如，玩家选择、弹幕生成、对话、以及 Chapter 的嵌套等。
//! Chapter 本身不包含 弹幕 或 UI 的定义与具体实现。

pub(crate) enum Chapter {
    /// UI Interaction Chapter.
    ///
    /// The Chapter allows players to interact with the UI.
    /// Chapters involving UI interaction should apply this, such as player choices, dialogues, etc.
    ///
    /// UI 交互章节。
    ///
    /// 此章节允许玩家与 UI 交互。
    /// 涉及 UI 交互的章节都应应用此项，如 玩家选择、对话 等。
    UIInteraction { ui_layout: String },

    /// Bullet Pattern Chapter.
    ///
    /// The Chapter is responsible for generating bullet patterns.
    ///
    /// 弹幕生成章节。
    ///
    /// 此章节负责生成弹幕模式。
    BulletPattern {
        pattern_id: Vec<String>,
        // TODO: 我们需要评估是否需要在这里添加更多的参数，例如 击破条件、name、时间限制 等。
    },

    /// Simple Wait Chapter.
    ///
    /// 简单的等待章节。
    Wait(f32),

    /// Nested Chapter.
    ///
    /// 章节的嵌套。
    Nested(Vec<Chapter>),
}
