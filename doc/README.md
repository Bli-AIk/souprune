# DELTA ENGINE Documentation - Vue.js 版本

这是从 React 版本翻译而来的 Vue.js 文档项目。

## 🎮 功能特点

- ✨ 复古像素风格界面（DELTARUNE/UNDERTALE 风格）
- 📖 Markdown 文档渲染
- 🎯 响应式设计（支持移动端滑动切换）
- ⚡ Vue 3 + TypeScript + Vite
- 🎨 Tailwind CSS 样式

## 🚀 快速开始

### 安装依赖

```bash
npm install
```

### 运行开发服务器

```bash
npm run dev
```

### 构建生产版本

```bash
npm run build
```

### 预览生产构建

```bash
npm run preview
```

## 📁 项目结构

```
src/
├── components/
│   ├── MarkdownRenderer.vue  # Markdown 渲染组件
│   ├── NavGroup.vue          # 导航组件
│   └── SoulCursor.vue        # 灵魂光标动画
├── App.vue                   # 主应用组件
├── main.ts                   # 应用入口
├── style.css                 # 全局样式
├── types.ts                  # TypeScript 类型定义
└── constants.ts              # 常量和文档数据
```

## 🎨 主要技术栈

- **Vue 3** - 渐进式 JavaScript 框架
- **TypeScript** - JavaScript 的超集
- **Vite** - 下一代前端构建工具
- **Tailwind CSS** - 实用优先的 CSS 框架
- **Lucide Vue** - 图标库
- **Markdown-it** - Markdown 解析器

## 📝 与 React 版本的主要差异

1. **组件系统**: 使用 Vue 3 Composition API (`<script setup>`)
2. **状态管理**: 使用 Vue 的 `ref` 和 `reactive` 替代 React 的 `useState`
3. **动画**: 使用 Vue 的 `<Transition>` 组件替代 Framer Motion
4. **Markdown**: 使用 `markdown-it` 替代 `react-markdown`
5. **图标**: 使用 `lucide-vue-next` 替代 `lucide-react`

## 🎯 特色功能

- **打字机效果**: H1 标题带有打字机动画效果
- **页面切换动画**: 平滑的页面转场效果
- **移动端手势**: 支持左右滑动切换页面
- **灵魂光标**: 选中项带有心形光标动画
- **复古滚动条**: 自定义像素风格滚动条

## 📄 License

MIT
