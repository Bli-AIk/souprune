# Contributing to SoupRune 🥣

Thank you for your interest in contributing to SoupRune! Whether you're a game creator, artist, musician, translator, or
developer — there's a place for you here.

Every type of contribution makes SoupRune better! We don't rank contributions by "difficulty" — we organize them by
**what you want to do**.

**Not sure where to start?** Join our [Discord](https://discord.gg/5YXK5DRjPZ) and say hi!

| English | 简体中文                              |
|---------|-----------------------------------|
| English | [简体中文](./CONTRIBUTING_zh-hans.md) |

---

## 📋 Table of Contents

- [Code of Conduct](#-code-of-conduct)
- [Bug Reports & Suggestions](#-bug-reports--suggestions)
- [Path A: Community & Creation](#-path-a-community--creation)
- [Path B: Framework Core Development](#-path-b-framework-core-development)
- [Path C: Ecosystem & Toolchain](#-path-c-ecosystem--toolchain)
- [Development Workflow](#-development-workflow)
- [License & CLA](#-license--cla)

---

## 📜 Code of Conduct

Please read and follow our [Code of Conduct](./CODE_OF_CONDUCT.md) (based on Contributor Covenant 3.0). We are a
community that bridges the UTDR fangame world and the Rust/Bevy ecosystem — mutual respect across different backgrounds
is essential.

---

## 🐛 Bug Reports & Suggestions

A well-written bug report or feature suggestion is just as valuable as a code contribution. This is the most direct way
to improve SoupRune.

**Anyone can contribute this way — no programming required.**

### Reporting a Bug

Use the [Bug Report template](https://github.com/Bli-AIk/souprune/issues/new?template=bug-report.md) and include:

- SoupRune version and your operating system
- Clear steps to reproduce the issue
- What you expected vs. what actually happened
- Any error messages, logs, or screenshots

### Suggesting a Feature

Use the [Feature Request template](https://github.com/Bli-AIk/souprune/issues/new?template=feature-request.md) and
describe:

- The problem you're trying to solve or the workflow you want to improve, or simply talk about what new features you
  want the framework to have!
- Your proposed solution and any alternatives you've considered

### Suggesting a Refactor

Use the [Refactor Request template](https://github.com/Bli-AIk/souprune/issues/new?template=refactor-request.md) for
code structure improvements.

### Not Sure If It's a Bug?

Come chat on [Discord](https://discord.gg/5YXK5DRjPZ) first — we're happy to help figure it out!

---

## 🎮 Path A: Community & Creation

**For**: Game creators, artists, musicians, translators, Mod developers, documentation writers

**Core idea**: Build your own Mod with SoupRune, or help polish community-shared Mods for everyone.

### A1. Make Your Own Mod

SoupRune is built for creating Deltarune/Undertale-style fangames. Here's what you can do:

- **Mortar scripting** — Write dialogue and event logic using Mortar (SoupRune's built-in scripting language)
- **FRE rules** — Define game logic with data-driven rules using the FRE (Fact-Rule-Event) system
- **View layouts** — Design UI with RON (Rusty Object Notation) configuration files
- **Level design** — Create maps with the [Tiled](https://www.mapeditor.org/) map editor
- **WASM Mods** — Build Mods in any language that compiles to WebAssembly

📚 **Getting started**: See official documentation
or [example mods repository](https://github.com/Bli-AIk/souprune_example_mods).

### A2. Contribute to Community Mods

- Create Mod templates or prerequisite Mods for the community
- Contribute reusable game resource packs
- Write Mod development tutorials and best practices

### A3. Art & Audio

- Sprite, UI, and animation assets
- Music and sound effects
- Follow the asset naming conventions: `lowercase_with_underscores`
- Place assets in `projects/<mod_name>/assets/<category>`

### A4. Documentation & Translation

- Fix typos, improve wording, add missing explanations
- Translate documentation and README (we maintain English and Simplified Chinese)
- Help others on Discord — community support is a contribution too!

> 🎮 **No CLA required for Path A.** Your Mods and games created under `projects/` are entirely yours. SoupRune's
> architecture ensures that LGPL's copyleft does not reach your creations — you interact with the framework through
> well-defined interfaces (WIT, RON, Mortar sandbox). You're free to choose any license for your work.

---

## 🔧 Path B: Framework Core Development

**For**: Rust developers (Rustaceans)

**You'll need**: Rust language basics, familiarity with [Bevy](https://bevyengine.org/) and ECS (
Entity-Component-System) architecture

### Getting Started

1. **Set up your environment**: See [Quick Start](./readme.md#-quick-start) in the README
2. **Read the style guide**: [`doc/style.md`](./style.md) — this is mandatory before writing any code
3. **Understand the architecture**: [`doc/architecture.md`](./doc/docs/architecture.md) — how SoupRune is structured

### What You Can Work On

- `src/core/` — Core systems (animation, camera, collision, danmaku, input, view, FRE bridge)
- `src/app_state/` — Application states (AppSetup → Menu → Overworld → Battle)
- `src/extra/` — Extension utilities (Markdown, TOML, Mortar loader, debug tools)
- `crates/souprune_api` / `crates/souprune_sdk` — WASM host-side interfaces

### Finding Issues

Look for issues labeled [`good first issue`](https://github.com/Bli-AIk/souprune/labels/good%20first%20issue) or [
`help wanted`](https://github.com/Bli-AIk/souprune/labels/help%20wanted).

### Learning Resources

New to Rust or Bevy? Here are some recommended resources:

- [The Rust Programming Language](https://doc.rust-lang.org/book/) (official book)
- [Bevy Book](https://bevyengine.org/learn/book/introduction/) (official Bevy guide)
- [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/)

> ⚖️ **CLA required for Path B.** Submitting PRs to the core codebase (`crates/`) requires signing
> our [Contributor License Agreement](./CLA.md). See [License & CLA](#-license--cla) for details.

---

## 🧩 Path C: Ecosystem & Toolchain

**For**: Bevy ecosystem contributors, tooling developers, cross-language binding authors

### C1. Bevy Crate Family

The following independent Bevy crates are maintained by the souprune maintainers and integrated into SoupRune as git
submodules. Each crate has its own repository and issue tracker:

| Crate                    | Description                                                            | Repository                                                |
|--------------------------|------------------------------------------------------------------------|-----------------------------------------------------------|
| **bevy_mortar_bond**     | Mortar scripting language bindings for Bevy (dialogue & event systems) | [GitHub](https://github.com/Bli-AIk/bevy_mortar_bond)     |
| **bevy_fact_rule_event** | FRE data-driven rule engine                                            | [GitHub](https://github.com/Bli-AIk/bevy_fact_rule_event) |
| **bevy_ecs_typewriter**  | Typewriter text effect                                                 | [GitHub](https://github.com/Bli-AIk/bevy_ecs_typewriter)  |
| **bevy_alight_motion**   | Alight Motion animation import & SDF rendering                         | [GitHub](https://github.com/Bli-AIk/bevy_alight_motion)   |
| **bevy_bitmap_text**     | Bitmap font rendering                                                  | [GitHub](https://github.com/Bli-AIk/bevy_bitmap_text)     |

**Please submit Issues and PRs to the respective repositories**, not to the main SoupRune repo. Improvements to these
crates benefit both SoupRune and any other projects using them.

### C2. Editor (souprune_editor)

> ⚠️ **Experimental** — The editor is not currently a development priority, but it's a fascinating exploration
> direction.

- Built on Bevy + egui
- Current status: proof-of-concept stage
- Possible contributions: UI prototypes, tool panel experiments, visual debugging explorations
- Note: APIs may change frequently

### C3. SDK & Multi-language Bindings

- `souprune_sdk` — Rust WASM guest SDK
- C# (.NET AOT) bindings
- Haxe bindings
- **Want to add a new language?** Go, Python, Zig, and more are welcome!

### C4. CI / Build / Toolchain

- Build optimization
- CI pipeline improvements
- Developer experience tools (justfile recipes, scaffolding, etc.)

> ⚖️ **CLA required for Path C** (for crates maintained by Bli-AIk). See [License & CLA](#-license--cla) for details.

---

## 🔄 Development Workflow

### Fork & Branch

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Initialize submodules: `git submodule update --init --recursive`

### Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(battle): add new damage calculation system
fix(overworld): resolve NPC collision edge case
docs(readme): update quick start section
refactor(view): simplify layout parsing logic
chore(deps): bump bevy to 0.18
```

### Before Submitting a PR

Run the full check suite:

```bash
cargo fmt --all                                          # Format
cargo clippy --workspace --all-targets -D warnings       # Lint
cargo test --workspace                                   # Test
```

### PR Checklist

- [ ] Code compiles without warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] PR description explains what and why
- [ ] Related issue is linked (if applicable)
- [ ] Breaking changes are noted (if applicable)

---

## ⚖️ License & CLA

### Dual-Licensing Model

SoupRune uses a **dual-licensing** model:

| User Type         | License                | Description                                                                             |
|-------------------|------------------------|-----------------------------------------------------------------------------------------|
| Open source users | **LGPL-3.0**           | Free to use and modify the framework; modifications to the core must be open-sourced    |
| Commercial users  | **Commercial License** | Allows closed-source modifications to the core framework (e.g., console platform ports) |

This model has well-established precedents in the open source world (Qt, MySQL, and others).

### Who Needs to Sign the CLA?

**The boundary is architectural, not personal.**

```
┌──────────────────────────────────────────────────────┐
│  projects/ — Mods & Game Projects                    │
│  ❌ NO CLA required                                  │
│  Your game interacts with SoupRune through            │
│  WIT / RON / Mortar sandbox interfaces.               │
│  LGPL copyleft is blocked at the architecture          │
│  boundary. Choose any license for your work.          │
├──────────────────────────────────────────────────────┤
│  crates/ — Framework Core / Preset / SDK             │
│  ✅ CLA required                                     │
│  Any Rust code PR to crates/ requires signing         │
│  the Contributor License Agreement.                   │
└──────────────────────────────────────────────────────┘
```

### 📝 A Transparent Note on Code Ownership & Commercial Licensing

SoupRune is committed to being a long-term, stable, community-driven playground for the next generation of fangames. To
ensure the framework's healthy development, we require all developers submitting code to the core repository to sign a
CLA (Contributor License Agreement).

We want to be completely transparent about why and how this works:

#### 1. Why Sign a CLA?

For game creators using SoupRune (the Project / Mod layer) — you **100% own your game** and can freely distribute it. No
authorization needed.

But for the SoupRune core framework, we use the LGPL-3.0 license. This means that if a commercial team wants to use
SoupRune on a closed-source console platform (like Switch or PS5, where dynamic linking isn't feasible due to platform
constraints), they would need to purchase a "commercial closed-source license." The CLA grants the core maintenance team
the legal right to issue such licenses.

Let's be honest: this is currently more of a safeguard for a possible future than an immediate revenue stream. But we
need the legal foundation in place. After all, choosing an open-source, sustainable framework is better than being
locked into certain expensive commercial engines with uncertain futures.

#### 2. How Will Commercial License Revenue Be Used?

Since tracking the commercial value of individual code lines and distributing proportional shares would create
unmanageable overhead, we currently do not offer direct financial splits for individual PRs.

Signing the CLA means you agree to license your code rights to the project. If commercial license revenue is generated,
these funds will belong to the SoupRune core maintenance team (currently Bli-AIk) and will be used for:

- 💰 **Infrastructure costs**: Servers, domains, CI/CD, and other hard costs
- 🛠️ **Core maintenance**: Funding core maintainer development time, ensuring issues are addressed promptly and
  documentation stays current
- 🏆 **Community bounties** (future plan): When funding is sufficient, we'll establish bounties to directly reward
  developers who solve complex core issues

#### 3. What Does Your Contribution Mean?

Every line of code you contribute paves the way for fangame creators who can't afford expensive commercial engine
licenses. Thank you for your understanding and commitment!

#### What Signing the CLA Means

- ✅ You **retain copyright** of your contribution
- ✅ You grant SoupRune the right to use and **sublicense** your contribution
- ✅ You confirm you have the right to submit the code
- ✅ Your contribution remains available to all open source users under LGPL-3.0

#### What Signing the CLA Does NOT Mean

- ❌ Does not transfer your copyright
- ❌ Does not restrict you from using the same code in other projects
- ❌ Does not affect any of your rights in the open source community

📄 **Read the full CLA**: [CLA.md](./CLA.md)

---

*Thank you for contributing to SoupRune! Together, we're building something special for the fangame community. 🥣*
