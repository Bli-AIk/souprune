# GitHub Workflows for souprune

此项目包含以下GitHub Actions工作流：

## 工作流说明

### 1. **CI工作流** (`.github/workflows/ci.yml`)

- **触发条件**: 推送到 `main`/`develop` 分支和创建PR时
- **功能**:
  - 在多个Rust版本上测试 (stable, beta, nightly)
  - 代码格式检查 (`cargo fmt`)
  - Clippy代码静态分析
  - 构建和测试
  - 为Linux、Windows、macOS构建二进制文件
  - 安全审计检查

### 2. **Coverage工作流** (`.github/workflows/coverage.yml`)

- **触发条件**: 推送到 `main` 分支和创建PR时
- **功能**:
  - 生成代码覆盖率报告
  - 上传到Codecov (可选)

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

## Issue模板

项目包含三种Issue模板：
- **Bug报告**: 用于报告bug和意外行为
- **功能请求**: 用于建议新功能或改进
- **重构请求**: 用于建议代码结构改进

## Dependabot配置

- 自动监控Rust依赖和GitHub Actions更新
- 每周创建更新PR

## 使用act验证工作流

可以使用 [act](https://github.com/nektos/act) 本地验证工作流：

```bash
# 验证CI工作流
act -n -j test

# 验证Coverage工作流
act -n -j coverage

# 验证依赖更新工作流
act -n -j update-dependencies
```

## 配置说明

### 必需的Secrets (用于Release工作流)

- `CRATES_TOKEN`: 用于发布到crates.io (可选)

### 必需的依赖 (Ubuntu)

CI工作流会自动安装以下Bevy依赖：
- pkg-config
- libx11-dev
- libasound2-dev
- libudev-dev
- libxcb-shape0-dev
- libxcb-xfixes0-dev