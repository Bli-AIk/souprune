# souprune-lint — JetBrains IDE 集成指南

在 JetBrains IDE（RustRover、IntelliJ IDEA、CLion 等）中将 `souprune-lint` 设置为 File Watcher，实现保存时自动校验 RON 文件。

## 前置条件

1. **File Watchers 插件** — 通过 *Settings → Plugins → Marketplace* 搜索 "File Watchers" 安装。
2. **构建 souprune-lint** — 在项目根目录运行 `cargo build -p souprune-lint --release`。

## File Watcher 配置

打开 *Settings → Tools → File Watchers*，点击 **+**，选择 **Custom**。

### Watcher 设置

| 字段                          | 值                                               |
|-----------------------------|-------------------------------------------------|
| **Name**                    | `souprune-lint`                                 |
| **File type**               | `RON`（如果没有 RON 类型，选择 `Any`）                     |
| **Scope**                   | `Project Files`（或限定到 `projects/` 的自定义 Scope）    |
| **Program**                 | `$ProjectFileDir$/target/release/souprune-lint` |
| **Arguments**               | `check --format jetbrains $FilePath$`           |
| **Output paths to refresh** | *（留空）*                                          |
| **Working directory**       | `$ProjectFileDir$`                              |

### 高级选项

| 选项                                                | 推荐设置       |
|---------------------------------------------------|------------|
| **Auto-save edited files to trigger the watcher** | ✅ 启用       |
| **Trigger the watcher on external changes**       | ❌ 禁用       |
| **Trigger watcher regardless of syntax errors**   | ✅ 启用       |
| **Create output file from stdout**                | ❌ 禁用       |
| **Show console**                                  | `On error` |

### 输出过滤器

添加输出过滤器以在编辑器中内联显示错误：

```
$FILE_PATH$:$LINE$:$COLUMN$: error: $MESSAGE$
```

此过滤器匹配 `jetbrains` 格式的输出：

```
path/to/file.view.ron:42:5: error: unexpected field `widht`
```

## 自定义 Scope（可选）

限制 Watcher 仅对 `projects/` 下的 RON 文件生效：

1. 打开 *Settings → Appearance & Behavior → Scopes*。
2. 点击 **+** 新建 Scope。
3. 命名为 `SoupRune RON Files`。
4. 设置模式：`file[souprune]:projects//*.ron`
5. 在 File Watcher 配置中选择此 Scope。

## 故障排除

### Watcher 未触发

- 确认 `.ron` 文件类型已被识别。前往 *Settings → Editor → File Types*，检查 `*.ron` 是否已列在某个已知类型下，或手动添加。
- 如果使用 "RON" 文件类型，确保 File Watchers 插件能识别它。

### 找不到二进制文件

- 确认二进制文件存在：`ls target/release/souprune-lint`
- 如需重新构建：`cargo build -p souprune-lint --release`

### 误报

- `souprune-lint` 使用 `ron` 0.12。部分为 `ron` 0.10 编写的 RON 文件可能使用了已弃用的语法（如对元组类型使用结构体字段语法）。请将
  RON 文件更新为位置元组语法。
