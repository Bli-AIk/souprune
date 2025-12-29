# Mortar 脚本基础

**Mortar** 是 SoupRune 专用的脚本语言，用于编写对话、剧情流和简单的逻辑控制。它的语法设计简洁，类似于编写剧本。

## 节点 (Node)

Mortar 脚本由多个 **节点 (Node)** 组成。每个节点代表对话或剧情的一个片段。

```rust
node Start {
    text: "你好，世界！"
} -> NextNode
```

### 节点结构

*   `node Name { ... }`: 定义一个节点。
*   `text: "..."`: 节点显示的文本内容。
*   `-> Target`: 定义节点结束后的跳转目标。

## 变量与插值

你可以在文本中使用 `{}` 来插入变量或函数调用的结果。

```rust
node Greeting {
    text: $"你好，{get_player_name()}！"
}
```
注意：使用插值时，字符串前需要加 `$` 符号。

## 外部函数 (External Functions)

要在 Mortar 中使用 Rust 端定义的逻辑，你需要声明 `fn`。

```rust
// 声明一个播放声音的函数
fn play_sound(path: String);

node Music {
    // 稍后在事件中使用它
    text: "听听这段音乐..."
}
```

## 条件分支 (Choice)

你可以提供选项供玩家选择，甚至根据条件显示不同的选项。

```rust
node Question {
    text: "你想去哪里？"
    
    choice: [
        "森林" -> Forest,
        "城堡" -> Castle,
        
        // 带有条件的选项
        "秘密基地" when has_key() -> SecretBase,
        
        // 嵌套选项
        "查看背包" -> [
            "苹果" -> EatApple;
            "面包" -> EatBread;
        ]
    ]
}
```

*   `when condition()`: 只有当条件满足时，该选项才可用。
*   `-> [ ... ]`: 选项可以嵌套。
