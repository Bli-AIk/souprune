# GitHub Workflows for souprune

此项目包含以下GitHub Actions工作流，所有工作流已修复并通过act验证。

## 工作流说明

### 1. **CI工作流** (`.github/workflows/ci.yml`)

- **触发条件**: 推送到 `main`/`develop` 分支和创建PR时
- **功能**:
  - 代码格式检查 (`cargo fmt`)
  - Clippy代码静态分析
  - 构建和测试
  - 为Linux、Windows、macOS构建二进制文件
  - 安全审计检查

### 2. **Coverage工作流** (`.github/workflows/coverage.yml`)

- **触发条件**: 推送到 `main` 分支和创建PR时
- **功能**:
  - 生成代码覆盖率报告
  - 上传到Codecov

### 3. **Release工作流** (`.github/workflows/release.yml`)

- **触发条件**: 推送标签 (v*)
- **功能**:
  - 自动创建GitHub Release
  - 构建并上传多平台二进制文件
  - 发布到crates.io (需要配置 `CRATES_TOKEN`)

### 4. **依赖更新工作流** (`.github/workflows/update-deps.yml`)

- **触发条件**: 每周一自动运行 或 手动触发
- **功能**:
  - 更新依赖版本
  - 自动创建PR如果有更新

## 修复内容 (2024-10-16)

### ✅ 已修复的问题:

1. **环境变量错误**: 修复了matrix中无法使用`${{ env.APP_NAME }}`的问题
2. **Wayland依赖错误**: 移除了Ubuntu 24.04中不存在的包，保留了实际存在的依赖
3. **废弃的Actions**: 更新了过时的GitHub Actions到最新版本
4. **项目名称**: 将所有引用从`my-bevy-game`更新为`souprune`

### 📦 **系统依赖 (Ubuntu 24.04兼容)**

所有工作流会自动安装以下Bevy依赖：
- `libasound2-dev` - 音频支持
- `libudev-dev` - 设备管理  
- `libwayland-dev` - Wayland显示服务器
- `libxkbcommon-dev` - 键盘处理
- `libxrandr-dev` - 显示管理
- `libx11-dev` - X11支持
- `libxi-dev` - 输入扩展
- `libxinerama-dev` - 多头显示
- `libxcursor-dev` - 光标支持
- `libgl1-mesa-dev` - OpenGL支持
- `pkg-config` - 包配置

### ⚠️ **跨平台编译说明**

- GitHub Actions在对应平台的runner上构建（ubuntu-latest、windows-latest、macos-latest）
- 从Linux本地交叉编译到其他平台需要特定的工具链
- 本地开发建议使用 `cargo build --release` 构建本机目标

## 使用act验证工作流

可以使用 [act](https://github.com/nektos/act) 本地验证工作流：

```bash
# 验证所有工作流
act -l

# 验证CI测试任务
act push -j test -n

# 验证构建任务  
act push -j build -n

# 验证Coverage工作流
act push -W .github/workflows/coverage.yml -n
```

## 配置说明

### 必需的Secrets (用于Release工作流)

- `CRATES_TOKEN`: 用于发布到crates.io (可选)

### Actions版本

- `actions/checkout@v4`: 仓库检出
- `dtolnay/rust-toolchain@stable`: Rust安装
- `actions/cache@v4`: 依赖缓存
- `softprops/action-gh-release@v1`: GitHub发布
- `rustsec/audit-check@v1`: 安全审计
- `codecov/codecov-action@v5`: 覆盖率报告
- `peter-evans/create-pull-request@v7`: 自动PR

所有Actions都锁定到特定版本以确保安全性和可重现性。