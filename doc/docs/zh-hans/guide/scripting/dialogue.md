# 对话与事件

Mortar 最强大的功能是将文本与事件精确同步。这对于制作富有表现力的 RPG 对话至关重要。

## 事件绑定 (with events)

在 `text` 字段之后，你可以使用 `with events` 块来定义在特定时间点触发的事件。

```rust
node Intro {
    text: "看！那是什么？"
    
    with events: [
        // 格式: 索引, 事件函数
        
        // 在第 0 个字符（开始）时播放声音
        0, play_sound("audio/surprise.wav"),
        
        // 在第 2 个字符时改变颜色
        2, set_color("#FF0000"),
        
        // 在第 5 个字符时震动屏幕
        5, shake_screen(1.0)
    ]
}
```

### 索引类型

*   **整数 (Int)**: 代表字符索引。例如 `2` 表示当打字机打出第 3 个字符时触发。
*   **浮点数 (Float)**: 代表时间（秒）。这在非打字机模式下或需要精细控制音频同步时很有用。

## 常用事件模式

虽然具体的事件函数取决于你的 Mod 代码实现（在 Rust 中注册），但以下是一些常见的设计模式：

### 1. 改变文本颜色

```rust
fn set_color(hex: String);

node ColorDemo {
    text: "这是普通的，这是红色的。"
    with events: [
        6, set_color("#FF0000"),
        10, set_color("#FFFFFF") // 恢复白色
    ]
}
```

### 2. 播放语音/音效

```rust
fn play_sfx(name: String);

node SoundDemo {
    text: "Boom! 爆炸了。"
    with events: [
        0, play_sfx("explosion")
    ]
}
```

### 3. 角色动画

```rust
fn set_face(expression: String);

node FaceDemo {
    text: "我很高兴... 现在我很生气！"
    with events: [
        0, set_face("happy"),
        10, set_face("angry")
    ]
}
```
