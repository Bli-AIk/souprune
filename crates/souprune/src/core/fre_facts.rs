//! Centralized FRE fact key constants.
//!
//! 集中管理的 FRE fact key 常量。
//! 所有在 core/ 代码中引用的 fact key 字符串都应在此定义，
//! 以避免拼写错误和方便全局搜索。
//!
//! 本文件只包含框架基础设施所需的 core facts。
//! 游戏特定的 fact key（如 `player:hp`、`enemy:*`）
//! 由各 app_state 模块自行定义或直接使用字符串字面量。

// ============================================================================
// Core Facts — 框架基础设施，不随 preset 变化
// ============================================================================

// ── Dialogue system facts ──────────────────────────────────────────

/// 对话系统是否持有输入焦点
pub const DIALOGUE_HAS_FOCUS: &str = "dialogue:has_focus";
/// 是否有打字机正在播放
pub const DIALOGUE_TYPEWRITER_PLAYING: &str = "dialogue:typewriter_playing";
/// 所有打字机是否都已完成
pub const DIALOGUE_ALL_TYPEWRITERS_FINISHED: &str = "dialogue:all_typewriters_finished";
/// 是否有任意打字机已完成
pub const DIALOGUE_ANY_TYPEWRITER_FINISHED: &str = "dialogue:any_typewriter_finished";
/// 简单文本模式是否激活
pub const DIALOGUE_SIMPLE_TEXT_ACTIVE: &str = "dialogue:simple_text_active";
/// 简单文本内容
pub const DIALOGUE_SIMPLE_TEXT: &str = "dialogue:simple_text";
/// 是否使用打字机效果
pub const DIALOGUE_HAS_TYPEWRITER: &str = "dialogue:has_typewriter";
/// 触发对话启动的待处理标志
pub const DIALOGUE_PENDING_START: &str = "dialogue:pending_start";
/// 待处理的 Mortar 脚本路径
pub const DIALOGUE_PENDING_MORTAR_PATH: &str = "dialogue:pending_mortar_path";
/// 待处理的 Mortar 节点名
pub const DIALOGUE_PENDING_MORTAR_NODE: &str = "dialogue:pending_mortar_node";
/// 待处理的 View 路径
pub const DIALOGUE_PENDING_VIEW: &str = "dialogue:pending_view";
/// 对话是否处于活跃状态
pub const DIALOGUE_ACTIVE: &str = "dialogue:active";
/// 对话是否有 Mortar 后端
pub const DIALOGUE_HAS_MORTAR: &str = "dialogue:has_mortar";
/// 焦点模式（"all_finished" 或 "first_finished"）
pub const DIALOGUE_FOCUS_MODE: &str = "dialogue:focus_mode";
/// 对话即将结束的内部标志（用于延迟一帧发送 ended 事件）
pub const DIALOGUE_PENDING_ENDED: &str = "dialogue:pending_ended";
/// 对话结束事件 ID
pub const DIALOGUE_ENDED: &str = "dialogue:ended";
/// 对话开始事件 ID
pub const DIALOGUE_STARTED: &str = "dialogue:started";
/// 对话语音路径
pub const DIALOGUE_VOICE: &str = "dialogue:voice";
/// 打字机速度
pub const DIALOGUE_TYPEWRITER_SPEED: &str = "dialogue:typewriter_speed";
/// 恢复时是否重播打字机
pub const DIALOGUE_REPLAY_ON_RESUME: &str = "dialogue:replay_on_resume";
/// 停止打字机事件前缀（匹配 "dialogue:stop*"）
pub const DIALOGUE_STOP_PREFIX: &str = "dialogue:stop";

// ── State facts (synced from Bevy states) ──────────────────────────

/// 当前 SequenceSubState 名称
pub const STATE_SEQUENCE_SUB_STATE: &str = "state:sequence_sub_state";
/// 当前 AppState 名称
pub const STATE_APP_STATE: &str = "state:app_state";

// ── View internal control facts ────────────────────────────────────

/// 请求关闭当前 View 的局部标志
pub const VIEW_CLOSE_REQUESTED: &str = "view:close_requested";
/// 请求切换状态的局部标志（值为目标状态名）
pub const VIEW_SWITCH_STATE: &str = "view:switch_state";
