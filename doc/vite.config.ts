import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import fs from 'fs';
import matter from 'gray-matter';

// Read the souprune version from Cargo.toml at build time
const readSoupruneVersion = (): string => {
  const cargoPath = path.resolve(__dirname, '..', 'crates', 'souprune', 'Cargo.toml');
  if (!fs.existsSync(cargoPath)) return '0.0.0';
  const content = fs.readFileSync(cargoPath, 'utf-8');
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);
  return match ? match[1] : '0.0.0';
};

// Parse front matter from markdown content, return { content, metadata }
const parseFrontMatter = (raw: string): { content: string; metadata?: Record<string, any> } => {
  if (!raw.startsWith('---')) return { content: raw };
  const { content, data } = matter(raw);
  if (Object.keys(data).length === 0) return { content: raw };
  return { content, metadata: data };
};

const docsPlugin = () => {
  const virtualModuleId = 'virtual:docs';
  const resolvedVirtualModuleId = '\0' + virtualModuleId;

  const generateDocs = () => {
    const docsDir = 'docs';
    const languages = ['en', 'zh-hans'];
    const soupruneVersion = readSoupruneVersion();
    
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
          const filePath = path.join(docsDir, lang, relativeItemPath);
          
          if (!fs.existsSync(filePath)) continue;

          const rawContent = fs.readFileSync(filePath, 'utf-8');
          const { content, metadata } = parseFrontMatter(rawContent);
          const titleMatch = content.match(/^#\s+(.*)/m);
          const title = titleMatch ? titleMatch[1] : label;
          const categorySlug = currentCategory.toLowerCase();

          let contentSerious = undefined;
          let metadataSerious = undefined;
          const seriousPath = filePath.replace(/\.md$/, '.serious.md');
          if (fs.existsSync(seriousPath)) {
            const rawSerious = fs.readFileSync(seriousPath, 'utf-8');
            const parsed = parseFrontMatter(rawSerious);
            contentSerious = parsed.content;
            metadataSerious = parsed.metadata;
          }

          allDocsData[lang].push({
            id,
            title,
            category: categorySlug,
            content,
            contentSerious,
            metadata,
            metadataSerious,
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
      export const SOUPRUNE_VERSION = ${JSON.stringify(soupruneVersion)};
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
