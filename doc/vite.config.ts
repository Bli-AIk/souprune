import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import fs from 'fs';

const docsPlugin = () => {
  const virtualModuleId = 'virtual:docs';
  const resolvedVirtualModuleId = '\0' + virtualModuleId;

  const generateDocs = () => {
    const docsDir = 'docs';
    const languages = ['en', 'zh-hans']; // Fixed list or fs.readdirSync(docsDir) if strictly structured
    
    const allDocsData: Record<string, any[]> = {};
    const allNavItems: Record<string, any[]> = {};

    for (const lang of languages) {
      allDocsData[lang] = [];
      allNavItems[lang] = [];

      const summaryPath = path.join(docsDir, lang, 'SUMMARY.md');
      if (!fs.existsSync(summaryPath)) continue;

      const summaryContent = fs.readFileSync(summaryPath, 'utf-8');
      const lines = summaryContent.split('\n');
      
      let currentCategory = '';
      const categoryRegex = /^#{1,2}\s+(.*)/;
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
          const relativeItemPath = itemMatch[2].trim();
          
          const id = path.basename(relativeItemPath, '.md');
          // File path is now docs/<lang>/<relative_path>
          const filePath = path.join(docsDir, lang, relativeItemPath);
          
          if (!fs.existsSync(filePath)) continue;

          const content = fs.readFileSync(filePath, 'utf-8');
          const titleMatch = content.match(/^#\s+(.*)/m);
          const title = titleMatch ? titleMatch[1] : label;
          const categorySlug = currentCategory.toLowerCase(); // Note: categories might need translation mapping if SUMMARY changes

          allDocsData[lang].push({
            id,
            title,
            category: categorySlug,
            content,
          });

          allNavItems[lang].push({
            id,
            label,
            category: categorySlug,
            font: categorySlug === 'guide' ? 'dtm-sans' : undefined,
          });
        }
      }
    }

    return `
      export const DOCS_DATA = ${JSON.stringify(allDocsData)};
      export const NAV_ITEMS = ${JSON.stringify(allNavItems)};
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
