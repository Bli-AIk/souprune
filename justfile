# 默认任务：构建 souprune（带 debug 特性）
default:
    cargo build --package souprune --bin souprune --features debug

# 格式化
fmt:
    cargo fmt --all

# 仅运行 clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# 拼写检查
typos:
    typos

# 代码统计检查（超过 1000 行报错）
tokei-check:
    @# 检查每个 Rust 文件，如果代码行数（不含注释和空行）超过 1000 行则报错
    @result=$(tokei --output json --files | jq -r '.Rust.reports[]? | select(.stats.code > 1000) | "Error: \(.name) has \(.stats.code) lines of code"') && \
    if [ -n "$result" ]; then \
        echo "$result"; \
        exit 1; \
    else \
        echo "Tokei OK: All Rust files are under 1000 lines of code."; \
    fi

# 综合检查（clippy + typos + tokei）
check:
    just clippy
    just typos
    just tokei-check

# 自动修复（clippy fix + typos -w）
fix:
    cargo clippy --fix --allow-dirty --all-features
    typos -w

# 构建（无 debug feature）
build:
    cargo build --package souprune --bin souprune

# 运行（无 debug feature）
run:
    cargo run --package souprune --bin souprune

# 开发运行（debug）
dev:
    cargo run --package souprune --bin souprune --features debug

# 清理
clean:
    cargo clean