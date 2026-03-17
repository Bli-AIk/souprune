# ===============================================
# 可覆盖变量（默认 souprune）
# 用法：just project=mygame build
# ===============================================
project := env_var_or_default("project", "souprune")

# ===============================================
# 默认任务：debug 构建
# ===============================================
default:
    cargo build -p {{project}} --features debug

# ===============================================
# 格式化
# ===============================================
fmt:
    cargo fmt --all

# ===============================================
# clippy
# ===============================================
clippy:
    cargo clippy --all-targets --all-features


# ===============================================
# clippy_local
# ===============================================
clippy_local:
    cargo clippy -p {{project}} --all-targets --all-features


# ===============================================
# 拼写检查
# ===============================================
typos:
    typos crates/{{project}}

# ===============================================
# Tokei 行数检查（主项目 + 所有子模块）
# ===============================================
tokei-check:
    @echo "=== Main project ==="
    @./tokei_check.sh
    @echo "=== bevy_alight_motion ==="
    @cd crates/bevy_alight_motion && bash tokei_check.sh
    @echo "=== bevy_ecs_typewriter ==="
    @cd crates/bevy_ecs_typewriter && bash tokei_check.sh
    @echo "=== bevy_fact_rule_event ==="
    @cd crates/bevy_fact_rule_event && bash tokei_check.sh
    @echo "=== bevy_mortar_bond ==="
    @cd crates/bevy_mortar_bond && bash tokei_check.sh
    @echo "=== bevy_workbench ==="
    @cd crates/bevy_workbench && bash tokei_check.sh
    @echo "=== flambe ==="
    @cd crates/flambe && bash tokei_check.sh
    @echo "=== mortar ==="
    @cd crates/mortar && bash tokei_check.sh

alias line := tokei-check

# ===============================================
# check
# ===============================================
check:
    cargo check -p {{project}} --all-targets --all-features

# ===============================================
# 综合检查
# ===============================================
full_check: clippy typos tokei-check
    echo "all checks completed"

# ===============================================
# clippy 自动修复
# ===============================================
fix:
    cargo clippy -p {{project}} --fix --allow-dirty --allow-staged --all-features

# ===============================================
# 普通构建（release 前）
# ===============================================
build:
    cargo build -p {{project}}

# ===============================================
# 普通运行（无 debug）
# ===============================================
run:
    cargo run -p {{project}}

# ===============================================
# 不安全 GPU 运行（禁用 Vulkan 验证层）
# ===============================================
unsafe_gpu:
    cargo run -p {{project}} --features unsafe_gpu

# ===============================================
# 不安全 GPU 开发运行（debug + 禁用验证层）
# ===============================================
unsafe_dev:
    cargo run -p {{project}} --features "unsafe_gpu,debug"

# ===============================================
# 测试
# ===============================================
test:
    cargo nextest run --workspace

# ===============================================
# 测试
# ===============================================
test_local:
    cargo test -p {{project}}

# ===============================================
# 开发运行（debug）
# ===============================================
dev:
    cargo run -p {{project}} --features debug

# ===============================================
# 清理
# ===============================================
clean:
    cargo clean

# ===============================================
# Tracy
# ===============================================
tracy:
    cargo run -p {{project}} --release --features trace_tracy

# ===============================================
# Bevy Debug Tracy (detailed Bevy function names in trace)
# ===============================================
bevy_debug_tracy:
    cargo run -p {{project}} --release --features debug_tracy

# ===============================================
# Souprune Debug Tracy (souprune debug feature + trace)
# ===============================================
soup_debug_tracy:
    cargo run -p {{project}} --release --features "trace_tracy,debug"

editor:
    cargo run -p souprune_editor

# ===============================================
# WASM Mod 构建（编译测试 mod 为 WASM 组件）
# ===============================================
wasm-build:
    cargo build -p souprune_mod_test --target wasm32-wasip2

# ===============================================
# WASM Mod 测试（用 mock host 加载运行）
# ===============================================
wasm-test: wasm-build
    cargo run -p souprune_mock_host -- target/wasm32-wasip2/debug/souprune_mod_test.wasm

# ===============================================
# 构建项目 Mod 为 WASM 组件
# Build a project mod as a WASM component
# Usage: just mod-build example_mod
# ===============================================
mod-build mod_name:
    cd projects/{{mod_name}}/code/mod_example && cargo build --target wasm32-wasip2 --release
    @echo "Built: projects/{{mod_name}}/code/mod_example/target/wasm32-wasip2/release/mod_example.wasm"

# ===============================================
# 构建并安装项目 Mod（复制 .wasm 到项目目录）
# Build and install a project mod
# Usage: just mod-install example_mod
# ===============================================
mod-install mod_name: (mod-build mod_name)
    cp projects/{{mod_name}}/code/mod_example/target/wasm32-wasip2/release/mod_example.wasm projects/{{mod_name}}/mod_example.wasm
    @echo "Installed: projects/{{mod_name}}/mod_example.wasm"