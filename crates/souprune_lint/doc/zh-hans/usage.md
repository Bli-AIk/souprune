# souprune-lint — 使用指南

用于校验 SoupRune RON 文件 Schema 的独立 CLI 工具。

## 安装

从源码构建（需要 Rust 工具链）：

```bash
cargo build -p souprune-lint --release
```

二进制文件位于 `target/release/souprune-lint`。

## 支持的文件类型

| 扩展名         | Schema         | 说明       |
|-------------|----------------|----------|
| `.view.ron` | `ViewLayout`   | 视图布局定义   |
| `.sdf.ron`  | `SdfStructure` | SDF 结构定义 |

## 命令

### `check` — 校验 RON 文件

```bash
# 校验单个文件
souprune-lint check path/to/file.view.ron

# 递归校验目录
souprune-lint check projects/

# 校验多个路径
souprune-lint check projects/mod_a/ projects/mod_b/some_file.view.ron
```

### 输出格式

使用 `--format` 控制输出样式：

#### `pretty`（默认）

丰富的诊断输出，带源码上下文，由 [ariadne](https://crates.io/crates/ariadne) 驱动：

```
Error: Unexpected missing field named `name` in `ViewNodeDef`
   ╭─[path/to/file.view.ron:5:9]
   │
 5 │         ),
   │         ┬
   │         ╰── Unexpected missing field named `name` in `ViewNodeDef`
───╯
```

#### `jetbrains`

兼容 JetBrains IDE File Watcher 输出过滤器的单行格式：

```
path/to/file.view.ron:5:9: error: Unexpected missing field named `name` in `ViewNodeDef`
```

#### `json`

供工具集成的机器可读 JSON：

```json
{
  "file": "path/to/file.view.ron",
  "diagnostics": [
    { "severity": "error", "line": 5, "column": 9, "message": "Unexpected missing field named `name` in `ViewNodeDef`" }
  ]
}
```

## 退出码

| 退出码 | 含义          |
|-----|-------------|
| `0` | 所有文件校验通过    |
| `1` | 一个或多个文件存在错误 |

## 校验层级

1. **RON 语法** — 括号匹配、逗号、引号、RON 扩展（`#![enable(implicit_some)]`）
2. **Schema 类型** — 字段名称、必需字段、类型不匹配、无效枚举变体

## 注意事项

- 使用 `ron` 0.12。为 `ron` 0.10 编写的文件如果使用了结构体语法 `(x: v, y: v, z: v)` 表示元组类型，可能会产生误报——请改用位置语法
  `(v, v, v)`。
- `#![enable(implicit_some)]` 扩展会自动生效。
