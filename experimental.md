# 实验性功能 (Experimental Features)

这些功能目前处于实验阶段，仅在启用 `experimental` 功能标志（feature flag）时生效。它们可能不稳定、不完整，或者在未来发生重大变化。

## 待完成/进行中的实验性功能

- [ ] **音乐同步与瓦片地图效果 (Music-Synced Tilemap Effects)**
    - **节拍追踪器 (Beat Tracker)**：一个追踪音乐节拍及其细分（最高支持 32 分音符）的系统，用于将游戏事件与背景音乐同步。
    - **涟漪揭示效果 (Ripple Reveal)**：一种视觉效果，地图瓦片会根据音乐节拍，以玩家或特定点为中心呈涟漪状逐个显现。
    - **黑白着色器 (Black & White Shader)**：一种自定义着色器系统，支持可控的淡入淡出级别，与揭示效果配合使用，使地图从纯白渐变至黑白。

- [ ] **Seedling 音频后端 (Seedling Audio Backend)**
    - 尝试集成 Seedling 音频框架，以提供比标准后端更先进的音频控制功能。
    - **注意**：目前在某些 Linux 配置下存在已知问题（如 ALSA 缓冲区欠载）。

## 已完成并准备移入标准版的功能

以下功能此前属于实验性功能，但现在已准备成为项目的默认组成部分：

- [x] **FRE (Fact-Rule-Event) 触发器系统**：为 Overworld 提供的完全数据驱动的事件触发机制。
- [x] **Overworld 追逐战机制 (Chase Mode)**：包括屏幕变暗、玩家描边、心形判定标记以及 Overworld 中的伤害检测逻辑。
  -  [ ] **待完成**: Overworld 受伤时的 UI 效果。
- [x] **弹幕系统通用化 (Danmaku Integration)**：统一的弹幕系统，支持在战斗和 Overworld 两种场景下运行，并包含完善的碰撞检测与无敌时间处理。