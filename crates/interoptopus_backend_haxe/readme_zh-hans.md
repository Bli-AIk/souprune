# interoptopus_backend_haxe

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中 - 实验性

**interoptopus_backend_haxe** — Interoptopus FFI 绑定生成器的 Haxe (hxcpp) 后端。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`interoptopus_backend_haxe` 是 [Interoptopus](https://github.com/ralfbiedert/interoptopus) FFI 绑定工具的实验性 Haxe 代码生成后端。  
它解决了在 Rust 和 Haxe (hxcpp) 之间手动编写 FFI 绑定的问题，让用户能够从 Rust 库自动生成类型安全的 Haxe 包装代码。

使用 `interoptopus_backend_haxe`，你只需要用 Interoptopus 属性标注你的 Rust 代码，工具就会生成对应的 Haxe 绑定。  
未来还计划支持回调处理和复杂类型映射等高级功能。

## 功能

* 从 Rust FFI 自动生成 Haxe (hxcpp) 绑定
* 与 Interoptopus 工作流集成
* 类型安全的代码生成
* （计划中）回调支持
* （计划中）复杂类型转换
* （计划中）文档生成

## 使用方法

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [build-dependencies]
   interoptopus = "0.15.0-alpha.24"
   interoptopus_backend_haxe = "0.0.1"
   ```

3. **基本使用**（在 build.rs 或绑定生成器中）：

   ```rust
   use interoptopus::Interop;
   use interoptopus_backend_haxe::Generator;

   // 定义 FFI 接口
   // ...（参见 Interoptopus 文档）

   // 生成 Haxe 绑定
   let inventory = my_inventory();
   let generator = Generator::new();
   generator.write_to_file(&inventory, "generated_bindings.hx").unwrap();
   ```

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本 | 描述                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [interoptopus](https://crates.io/crates/interoptopus) | 0.15.0-alpha.24   | FFI 绑定框架 |
| [interoptopus_backend_utils](https://crates.io/crates/interoptopus_backend_utils) | 0.15.0-alpha.24   | 共享后端工具 |
| [derive_builder](https://crates.io/crates/derive_builder) | 0.20.2   | Builder 模式宏 |
| [heck](https://crates.io/crates/heck) | 0.5   | 大小写转换工具 |

## 警告

⚠️ **这是一个实验性的早期版本。**

- API 不稳定，可能会发生重大变化
- 测试覆盖率有限
- 不推荐用于生产环境
- 未来版本预计会有破坏性更改

如有问题或改进建议，欢迎反馈！

## 贡献指南

欢迎贡献！
无论你想修复错误、添加功能或改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目可依据以下任意一种许可证进行分发：

* Apache License 2.0（[LICENSE-APACHE](LICENSE-APACHE) 或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)）
* MIT License（[LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)）

可任选其一。
