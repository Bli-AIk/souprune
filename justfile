# 可覆盖变量（默认 souprune）
# 用法：just project=mygame build
project := env_var_or_default("project", "souprune")
# Target mod for content building (reads from projects/config.toml by default)
# 用法：just mod=epictale run
mod := env_var_or_default("mod", `sed -n 's/^mod_name[[:space:]]*=[[:space:]]*"\(.*\)".*/\1/p' projects/config.toml`)
workspace_root := justfile_directory()

# 默认任务：debug 构建
default: prepare-assets
    cargo build -p {{project}} --features debug

# 构建运行主程序前所需的内置 WASM 与 mod RON 内容
prepare-assets: builtin-build runtime-build-deps content-build-deps

# release 构建运行主程序前所需的内置 WASM 与 mod RON 内容
prepare-assets-release: builtin-build-release runtime-build-deps-release content-build-deps-release

# 格式化
fmt:
    cargo fmt --all

# 格式化所有 mod 的代码
fmt-mods:
    @for toml in $(find projects -name Cargo.toml); do \
        cargo fmt --manifest-path $toml; \
    done
    @echo "Formatted all mods"

# 构建所有 mod 的 runtime 和 content 主线
build-mods: builtin-build
    @for mod_dir in $(find projects -mindepth 1 -maxdepth 1 -type d | sort); do \
        mod_name=$(basename "$mod_dir"); \
        just project-build "$mod_name" || exit $?; \
    done
    @echo "Built all mods"

# 构建目标 mod 及其依赖 mod 的 content guest，并直接生成正式内容文件
content-build-deps:
    @for mod_name in $(CARGO_TARGET_DIR={{workspace_root}}/target/cauld-ron-deps cargo run -p souprune_cauld_ron --features deps-cli --bin cauld-ron-deps -- {{mod}}); do \
        just content-build "$mod_name" || exit $?; \
    done
    @echo "Built content for {{mod}} and its dependencies"

# 构建目标 mod 及其依赖 mod 的 runtime WASM
runtime-build-deps:
    @for mod_name in $(CARGO_TARGET_DIR={{workspace_root}}/target/cauld-ron-deps cargo run -p souprune_cauld_ron --features deps-cli --bin cauld-ron-deps -- {{mod}}); do \
        just runtime-build "$mod_name" || exit $?; \
    done
    @echo "Built runtime for {{mod}} and its dependencies"

# release 构建目标 mod 及其依赖 mod 的 runtime WASM
runtime-build-deps-release:
    @for mod_name in $(CARGO_TARGET_DIR={{workspace_root}}/target/cauld-ron-deps cargo run -p souprune_cauld_ron --features deps-cli --bin cauld-ron-deps -- {{mod}}); do \
        just runtime-build-release "$mod_name" || exit $?; \
    done
    @echo "Built release runtime for {{mod}} and its dependencies"

# release 构建目标 mod 及其依赖 mod 的 content guest，并直接生成正式内容文件
content-build-deps-release:
    @for mod_name in $(CARGO_TARGET_DIR={{workspace_root}}/target/cauld-ron-deps cargo run -p souprune_cauld_ron --features deps-cli --bin cauld-ron-deps -- {{mod}}); do \
        just content-build-release "$mod_name" || exit $?; \
    done
    @echo "Built release content for {{mod}} and its dependencies"

alias generate-mods := build-mods

# clippy
clippy:
    cargo clippy --all-targets --all-features --no-deps -- -D warnings


# clippy_local
clippy_local:
    cargo clippy -p {{project}} --all-targets --all-features --no-deps -- -D warnings

# 对所有 mod crate 运行 clippy
clippy-mods:
    @for toml in $(find projects -mindepth 2 -maxdepth 3 -name Cargo.toml | sort); do \
        crate_id=$(echo "$toml" | sed 's#^projects/##; s#/Cargo.toml$##; s#/#-#g'); \
        cargo clippy --manifest-path "$toml" --target wasm32-wasip2 --target-dir "{{workspace_root}}/target/mod-clippy/$crate_id" --all-targets --all-features --no-deps -- -D warnings || exit $?; \
    done
    @echo "Clippy passed for all mods"


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
build: prepare-assets
    cargo build -p {{project}}

# 普通运行（动态链接加速）
run: prepare-assets
    cargo run -p {{project}} --features bevy/dynamic_linking

# 不安全 GPU 运行（禁用 Vulkan 验证层）
unsafe_gpu: prepare-assets
    cargo run -p {{project}} --features unsafe_gpu

# 不安全 GPU 开发运行（debug + 禁用验证层）
unsafe_dev: prepare-assets
    cargo run -p {{project}} --features "unsafe_gpu,debug"

# 测试
test:
    cargo nextest run --workspace

# 测试
test_local:
    cargo test -p {{project}}

# 运行所有 mod crate 的测试
test-mods:
    @for toml in $(find projects -mindepth 2 -maxdepth 3 -name Cargo.toml | sort); do \
        crate_id=$(echo "$toml" | sed 's#^projects/##; s#/Cargo.toml$##; s#/#-#g'); \
        cargo nextest run --manifest-path "$toml" --target-dir "{{workspace_root}}/target/mod-tests/$crate_id" || exit $?; \
    done
    @echo "Tested all mods"

# 开发运行（debug + 动态链接加速）
dev: prepare-assets
    cargo run -p {{project}} --features "debug,bevy/dynamic_linking"

# 清理
clean:
    cargo clean

# 彻底清理：根工作区、独立子模块、mod 残留 target 与 .build
clean-all:
    cargo clean
    @find crates projects -type d -name target -prune -exec rm -rf {} +
    @find projects -depth -path '*/.build/*' ! -name 'cauld-ron-output-manifest.toml' -exec rm -rf {} +
    @echo "Cleaned workspace, nested crate targets, and mod build artifacts (preserved Cauld-ron manifests)"

# Release 构建运行（静态链接，最终性能）
release: prepare-assets-release
    cargo run -p {{project}} --release

# Tracy
tracy: prepare-assets-release
    cargo run -p {{project}} --release --features trace_tracy

# Bevy Debug Tracy (detailed Bevy function names in trace)
bevy_debug_tracy: prepare-assets-release
    cargo run -p {{project}} --release --features debug_tracy

# Souprune Debug Tracy (souprune debug feature + trace)
soup_debug_tracy: prepare-assets-release
    cargo run -p {{project}} --release --features "trace_tracy,debug"

editor:
    cargo run --manifest-path crates/souprune_editor/Cargo.toml

# WASM Mod 构建（编译测试 mod 为 WASM 组件）
wasm-build:
    cargo build -p souprune_mod_test --target wasm32-wasip2

# WASM Mod 测试（用 mock host 加载运行）
wasm-test: wasm-build
    cargo run -p souprune_mock_host -- target/wasm32-wasip2/debug/souprune_mod_test.wasm

# 构建内置弹幕 WASM，并安装到 assets/builtins/
builtin-build:
    mkdir -p assets/builtins
    CARGO_TARGET_DIR={{workspace_root}}/target/builtins cargo build --manifest-path crates/souprune_builtins/Cargo.toml --target wasm32-wasip2
    cp {{workspace_root}}/target/builtins/wasm32-wasip2/debug/souprune_builtins.wasm assets/builtins/souprune_builtins.wasm
    @echo "Built builtin WASM: assets/builtins/souprune_builtins.wasm"

# release 构建内置弹幕 WASM，并安装到 assets/builtins/
builtin-build-release:
    mkdir -p assets/builtins
    CARGO_TARGET_DIR={{workspace_root}}/target/builtins cargo build --manifest-path crates/souprune_builtins/Cargo.toml --target wasm32-wasip2 --release
    cp {{workspace_root}}/target/builtins/wasm32-wasip2/release/souprune_builtins.wasm assets/builtins/souprune_builtins.wasm
    @echo "Built builtin WASM: assets/builtins/souprune_builtins.wasm"

# 构建项目 runtime WASM 组件并安装到 .build/runtime.wasm
runtime-build mod_name:
    CARGO_TARGET_DIR={{workspace_root}}/target/runtime-wasm/{{mod_name}} cargo build --manifest-path projects/{{mod_name}}/runtime/Cargo.toml --target wasm32-wasip2 --locked
    mkdir -p projects/{{mod_name}}/.build
    cp {{workspace_root}}/target/runtime-wasm/{{mod_name}}/wasm32-wasip2/debug/runtime.wasm projects/{{mod_name}}/.build/runtime.wasm
    @echo "Built runtime: projects/{{mod_name}}/.build/runtime.wasm"

# 构建所有 mod 的 runtime WASM
runtime-build-mods:
    @for mod_dir in $(find projects -mindepth 1 -maxdepth 1 -type d | sort); do \
        mod_name=$(basename "$mod_dir"); \
        just runtime-build "$mod_name" || exit $?; \
    done
    @echo "Built runtime for all mods"

# release 构建项目 runtime WASM 组件并安装到 .build/runtime.wasm
runtime-build-release mod_name:
    CARGO_TARGET_DIR={{workspace_root}}/target/runtime-wasm/{{mod_name}} cargo build --manifest-path projects/{{mod_name}}/runtime/Cargo.toml --target wasm32-wasip2 --release --locked
    mkdir -p projects/{{mod_name}}/.build
    cp {{workspace_root}}/target/runtime-wasm/{{mod_name}}/wasm32-wasip2/release/runtime.wasm projects/{{mod_name}}/.build/runtime.wasm
    @echo "Built runtime: projects/{{mod_name}}/.build/runtime.wasm"

# 构建项目 content guest，安装到 .build/content.wasm，并直接生成正式内容文件
content-build mod_name:
    CARGO_TARGET_DIR={{workspace_root}}/target/content-wasm/{{mod_name}} cargo build --manifest-path projects/{{mod_name}}/content/Cargo.toml --target wasm32-wasip2 --locked
    mkdir -p projects/{{mod_name}}/.build
    cp {{workspace_root}}/target/content-wasm/{{mod_name}}/wasm32-wasip2/debug/content.wasm projects/{{mod_name}}/.build/content.wasm
    CARGO_TARGET_DIR={{workspace_root}}/target/cauld-ron-cli cargo run -p cauld-ron -- build projects/{{mod_name}}/.build/content.wasm --output projects/{{mod_name}}
    @echo "Built content: projects/{{mod_name}}/.build/content.wasm"

# 构建所有 mod 的 content guest，并直接生成正式内容文件
content-build-mods:
    @for mod_dir in $(find projects -mindepth 1 -maxdepth 1 -type d | sort); do \
        mod_name=$(basename "$mod_dir"); \
        just content-build "$mod_name" || exit $?; \
    done
    @echo "Built content for all mods"

# release 构建项目 content guest，安装到 .build/content.wasm，并直接生成正式内容文件
content-build-release mod_name:
    CARGO_TARGET_DIR={{workspace_root}}/target/content-wasm/{{mod_name}} cargo build --manifest-path projects/{{mod_name}}/content/Cargo.toml --target wasm32-wasip2 --release --locked
    mkdir -p projects/{{mod_name}}/.build
    cp {{workspace_root}}/target/content-wasm/{{mod_name}}/wasm32-wasip2/release/content.wasm projects/{{mod_name}}/.build/content.wasm
    CARGO_TARGET_DIR={{workspace_root}}/target/cauld-ron-cli cargo run -p cauld-ron -- build projects/{{mod_name}}/.build/content.wasm --output projects/{{mod_name}}
    @echo "Built content: projects/{{mod_name}}/.build/content.wasm"

# 构建项目的 runtime 和 content 两条主线
project-build mod_name: (runtime-build mod_name) (content-build mod_name)
    @echo "Built project: {{mod_name}}"

# release 构建项目的 runtime 和 content 两条主线
project-build-release mod_name: (runtime-build-release mod_name) (content-build-release mod_name)
    @echo "Built project: {{mod_name}}"

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
