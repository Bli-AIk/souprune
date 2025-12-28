import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import fs from 'fs';

const docsPlugin = () => {
  const virtualModuleId = 'virtual:docs';
  const resolvedVirtualModuleId = '\0' + virtualModuleId;

  const generateDocs = () => {
    const summaryPath = 'docs/SUMMARY.md';
    if (!fs.existsSync(summaryPath)) {
      return `export const DOCS_DATA = []; export const NAV_ITEMS = [];`;
    }
    const summaryContent = fs.readFileSync(summaryPath, 'utf-8');
    const lines = summaryContent.split('\n');

    const docsData = [];
    const navItems = [];
    let currentCategory = '';

    const categoryRegex = /^#\s+(.*)/;
    const itemRegex = /-\s+\[(.*?)\]\((.*?)\)/;

    for (const line of lines) {
      const categoryMatch = line.match(categoryRegex);
      if (categoryMatch) {
        currentCategory = categoryMatch[1].trim();
        continue;
      }

      const itemMatch = line.match(itemRegex);
      if (itemMatch && currentCategory) {
        const label = itemMatch[1].trim();
        const itemPath = itemMatch[2].trim();
        
        const id = path.basename(itemPath, '.md');
        const filePath = path.join('docs', itemPath);
        
        if (!fs.existsSync(filePath)) continue;

        const content = fs.readFileSync(filePath, 'utf-8');
        const titleMatch = content.match(/^#\s+(.*)/m);
        const title = titleMatch ? titleMatch[1] : label;
        const categorySlug = currentCategory.toLowerCase();

        docsData.push({
          id,
          title,
          category: categorySlug,
          content,
        });

        navItems.push({
          id,
          label,
          category: categorySlug,
          font: categorySlug === 'guide' ? 'dtm-sans' : undefined,
        });
      }
    }

    return `
      export const DOCS_DATA = ${JSON.stringify(docsData)};
      export const NAV_ITEMS = ${JSON.stringify(navItems)};
    `;
  };

  return {
    name: 'docs-plugin',
    resolveId(id) {
      if (id === virtualModuleId) {
        return resolvedVirtualModuleId;
      }
    },
    load(id) {
      if (id === resolvedVirtualModuleId) {
        return generateDocs();
      }
    },
    configureServer(server) {
      const docsDir = path.resolve(__dirname, 'docs');
      server.watcher.add(docsDir);

      const reload = () => {
        server.ws.send({
          type: 'full-reload',
          path: '*'
        });
      };

      server.watcher.on('add', (file) => {
        if (file.startsWith(docsDir)) {
          reload();
        }
      });
      server.watcher.on('change', (file) => {
        if (file.startsWith(docsDir)) {
          reload();
        }
      });
      server.watcher.on('unlink', (file) => {
        if (file.startsWith(docsDir)) {
          reload();
        }
      });
    }
  };
}

export default defineConfig({
  plugins: [vue(), docsPlugin()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
