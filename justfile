# 可覆盖变量（默认 souprune）
# 用法：just project=mygame build
project := env_var_or_default("project", "souprune")

# 默认任务：debug 构建
default:
    cargo build -p {{project}} --features debug

# 格式化
fmt:
    cargo fmt --all

# clippy
clippy:
    cargo clippy --all-targets --all-features


# clippy_local
clippy_local:
    cargo clippy -p {{project}} --all-targets --all-features


# 拼写检查
typos:
    typos crates/{{project}}

# Tokei 行数检查（主项目 + 所有子模块）
tokei-check:
    @./scripts/tokei_check.sh --workspace

# 架构边界检查
arch-check:
    @bash ./scripts/check_core_boundaries.sh
    @bash ./scripts/check_editor_boundaries.sh

alias line := tokei-check

# check
check:
    cargo check -p {{project}} --all-targets --all-features

# 综合检查
full_check: clippy typos tokei-check arch-check
    echo "all checks completed"

# clippy 自动修复
fix:
    cargo clippy -p {{project}} --fix --allow-dirty --allow-staged --all-features

# 普通构建（release 前）
build:
    cargo build -p {{project}}

# 普通运行（动态链接加速）
run:
    cargo run -p {{project}} --features bevy/dynamic_linking

# 不安全 GPU 运行（禁用 Vulkan 验证层）
unsafe_gpu:
    cargo run -p {{project}} --features unsafe_gpu

# 不安全 GPU 开发运行（debug + 禁用验证层）
unsafe_dev:
    cargo run -p {{project}} --features "unsafe_gpu,debug"

# 测试
test:
    cargo nextest run --workspace

# 测试
test_local:
    cargo test -p {{project}}

# 开发运行（debug + 动态链接加速）
dev:
    cargo run -p {{project}} --features "debug,bevy/dynamic_linking"

# 清理
clean:
    cargo clean

# Release 构建运行（静态链接，最终性能）
release:
    cargo run -p {{project}} --release

# Tracy
tracy:
    cargo run -p {{project}} --release --features trace_tracy

# Bevy Debug Tracy (detailed Bevy function names in trace)
bevy_debug_tracy:
    cargo run -p {{project}} --release --features debug_tracy

# Souprune Debug Tracy (souprune debug feature + trace)
soup_debug_tracy:
    cargo run -p {{project}} --release --features "trace_tracy,debug"

editor:
    cargo run -p souprune_editor

# WASM Mod 构建（编译测试 mod 为 WASM 组件）
wasm-build:
    cargo build -p souprune_mod_test --target wasm32-wasip2

# WASM Mod 测试（用 mock host 加载运行）
wasm-test: wasm-build
    cargo run -p souprune_mock_host -- target/wasm32-wasip2/debug/souprune_mod_test.wasm

# 构建项目 Mod 为 WASM 组件
mod-build mod_name:
    cd projects/{{mod_name}}/code/mod_example && cargo build --target wasm32-wasip2 --release
    @echo "Built: projects/{{mod_name}}/code/mod_example/target/wasm32-wasip2/release/mod_example.wasm"

# 构建并安装项目 Mod（复制 .wasm 到项目目录）
mod-install mod_name: (mod-build mod_name)
    cp projects/{{mod_name}}/code/mod_example/target/wasm32-wasip2/release/mod_example.wasm projects/{{mod_name}}/mod_example.wasm
    @echo "Installed: projects/{{mod_name}}/mod_example.wasm"

# 打包发行版
# 统一脚本：scripts/pack.sh
# 支持别名：linux, windows, linux-arm，或任意 Rust target triple

# 打包到 linux
pack-linux:
    @bash scripts/pack.sh linux

# 打包到 windows
pack-windows:
    @bash scripts/pack.sh windows

# 打包到 {目标}
pack target:
    @bash scripts/pack.sh {{target}}
