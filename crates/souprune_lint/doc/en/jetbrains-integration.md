# souprune-lint — JetBrains IDE Integration

Set up `souprune-lint` as a File Watcher in JetBrains IDEs (RustRover, IntelliJ IDEA, CLion, etc.) to get real-time RON
file validation on save.

## Prerequisites

1. **File Watchers plugin** — Install via *Settings → Plugins → Marketplace*, search "File Watchers".
2. **Build souprune-lint** — Run `cargo build -p souprune-lint --release` in the project root.

## File Watcher Configuration

Open *Settings → Tools → File Watchers*, click **+**, select **Custom**.

### Watcher Settings

| Field                       | Value                                                      |
|-----------------------------|------------------------------------------------------------|
| **Name**                    | `souprune-lint`                                            |
| **File type**               | `RON` (or `Any` if RON is not listed)                      |
| **Scope**                   | `Project Files` (or a custom scope limited to `projects/`) |
| **Program**                 | `$ProjectFileDir$/target/release/souprune-lint`            |
| **Arguments**               | `check --format jetbrains $FilePath$`                      |
| **Output paths to refresh** | *(leave empty)*                                            |
| **Working directory**       | `$ProjectFileDir$`                                         |

### Advanced Options

| Option                                            | Recommended |
|---------------------------------------------------|-------------|
| **Auto-save edited files to trigger the watcher** | ✅ Enabled   |
| **Trigger the watcher on external changes**       | ❌ Disabled  |
| **Trigger watcher regardless of syntax errors**   | ✅ Enabled   |
| **Create output file from stdout**                | ❌ Disabled  |
| **Show console**                                  | `On error`  |

### Output Filter

To display errors inline in the editor, add an output filter:

```
$FILE_PATH$:$LINE$:$COLUMN$: error: $MESSAGE$
```

This matches the `jetbrains` output format:

```
path/to/file.view.ron:42:5: error: unexpected field `widht`
```

### Showing Errors as Errors (not Warnings)

By default, the "File Watcher Problems" inspection severity is **Warning**. To display lint errors as red errors:

1. Open *Settings → Editor → Inspections*
2. Search for **File Watcher Problems** (under *Other*)
3. Change **Severity** from `Warning` to **`Error`**
4. Click **Apply**

## Custom Scope (Optional)

To limit the watcher to RON files under `projects/`:

1. Open *Settings → Appearance & Behavior → Scopes*.
2. Click **+** to add a new scope.
3. Name it `SoupRune RON Files`.
4. Set pattern: `file[souprune]:projects//*.ron`
5. Select this scope in the File Watcher configuration.

## Troubleshooting

### Watcher not triggering

- Ensure the `.ron` file type is recognized. Go to *Settings → Editor → File Types* and check that `*.ron` is listed
  under a known type, or add it manually.
- If using "RON" file type, make sure the File Watchers plugin recognizes it.

### Binary not found

- Verify the binary exists: `ls target/release/souprune-lint`
- Rebuild if needed: `cargo build -p souprune-lint --release`

### False positives

- `souprune-lint` uses `ron` 0.12. Some RON files written for `ron` 0.10 may use deprecated syntax (e.g., struct-style
  fields for tuple types). Update the RON files to use positional tuple syntax.
