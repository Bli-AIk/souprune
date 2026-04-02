<template>
  <div class="markdown-container">
    <div v-html="renderedContent" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js';
import miHighlight from 'markdown-it-highlightjs';

const props = defineProps<{
  content: string;
}>();

// RON (Rust Object Notation) uses Rust-like syntax
hljs.registerAliases('ron', { languageName: 'rust' });

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
}).use(miHighlight, { hljs, inline: true });

// 自定义渲染规则
md.renderer.rules.heading_open = (tokens, idx) => {
  const level = tokens[idx].tag;
  
  if (level === 'h1') {
    return `<div class="mb-8"><h1 class="font-vt323 text-2xl md:text-3xl text-yellow-300 uppercase tracking-widest drop-shadow-[2px_2px_0_rgba(0,0,0,1)] min-h-[3rem] typewriter">`;
  } else if (level === 'h2') {
    return `<div class="flex items-center gap-4 mt-10 mb-6">
      <img src="/images/spr_heartsmall_0.png" alt="Small Heart" class="w-4 h-4 object-contain image-pixelated relative top-0.5" />
      <h2 class="font-speechbubble text-lg text-pink-400 uppercase">`;
  } else if (level === 'h3') {
    return `<h3 class="font-speechbubble text-base mt-6 mb-3 text-green-400">`;
  }
  return `<${level} class="font-speechbubble text-sm mt-4 mb-2 text-gray-400 uppercase">`;
};

md.renderer.rules.heading_close = (tokens, idx) => {
  const level = tokens[idx].tag;
  if (level === 'h1') {
    return `</h1><div class="h-0.5 bg-gray-700 w-full mt-4"></div></div>`;
  }
  if (level === 'h2') {
    return `</h2></div>`;
  }
  return `</${level}>`;
};

md.renderer.rules.paragraph_open = () => {
  return '<p class="text-2xl leading-loose mb-6 text-white drop-shadow-md font-vt323">';
};

md.renderer.rules.bullet_list_open = () => {
  return '<ul class="custom-bullet-list list-none ml-2 mb-6 space-y-3">';
};

md.renderer.rules.ordered_list_open = () => {
  return '<ol class="list-decimal ml-6 mb-6 space-y-3 text-xl text-gray-300">';
};

md.renderer.rules.list_item_open = () => {
  return '<li class="text-xl relative pl-8">';
};

// Removed custom fence rule as markdown-it-highlightjs handles it

// Custom fence rule to intercept "dialogue" blocks
const defaultFence = md.renderer.rules.fence;

md.renderer.rules.fence = (tokens, idx, options, env, self) => {
  const token = tokens[idx];
  const info = token.info ? md.utils.unescapeAll(token.info).trim() : '';

  if (info === 'dialogue') {
    const lines = token.content.trim().split('\n');
    let imageSrc = '/images/faces/toriel.png'; // default fallback
    let charName = 'toriel';
    let fontClass = 'font-vt323'; // default font
    
    let contentStartIndex = 0;

    // Phase 1: Parse header tags (consume lines starting with <...>)
    // This allows arbitrary order like:
    // <path:/images/faces/sans.png>
    // <font:comic>
    while (contentStartIndex < lines.length) {
      const line = lines[contentStartIndex].trim();
      const pathMatch = line.match(/^<path:(.*?)>$/i);
      const fontMatch = line.match(/^<font:(.*?)>$/i);
      
      if (pathMatch) {
        imageSrc = pathMatch[1].trim();
        contentStartIndex++;
      } else if (fontMatch) {
        const fontName = fontMatch[1].trim().toLowerCase();
        if (fontName === 'comic' || fontName === 'sans') {
          fontClass = 'font-comic tracking-wide';
        } else if (fontName === 'papyrus') {
          fontClass = 'font-papyrus tracking-widest'; // Papyrus usually needs more spacing
        } else {
          fontClass = `font-${fontName}`;
        }
        contentStartIndex++;
      } else {
        // Not a tag, stop parsing headers
        break;
      }
    }

    // Phase 2: Legacy/Simple format fallback (only if no tags were found at the very top)
    // Checks the first line of content for "Name:" or "image:" patterns
    if (contentStartIndex === 0 && lines.length > 0) {
      const firstLine = lines[0].trim();
      if (firstLine.toLowerCase().startsWith('image:')) {
        imageSrc = firstLine.substring(6).trim();
        contentStartIndex = 1;
      } else if (firstLine.endsWith(':')) {
        // Format: "Toriel:"
        charName = firstLine.slice(0, -1).trim().toLowerCase();
        imageSrc = `/images/faces/${charName}.png`;
        contentStartIndex = 1;
      } else if (firstLine.includes(':') && !firstLine.startsWith('*')) {
        // Format: "Toriel: Hello"
        const splitIdx = firstLine.indexOf(':');
        charName = firstLine.slice(0, splitIdx).trim().toLowerCase();
        imageSrc = `/images/faces/${charName}.png`;
        
        // Don't skip the line, just strip the name prefix from it
        lines[0] = firstLine.slice(splitIdx + 1).trim();
        // contentStartIndex remains 0
      }
    }

    // Slice the actual content
    const textLines = lines.slice(contentStartIndex);

    // Process text lines
    const renderedText = textLines.map(line => {
      const trimmed = line.trim();
      return md.renderInline(trimmed);
    }).join('<br/>');

    return `
      <div class="ut-box border-4 border-white bg-black p-6 my-8 flex items-start gap-9 w-full max-w-4xl">
        <div class="shrink-0 w-[100px] flex justify-center">
          <img 
            src="${imageSrc}" 
            alt="${charName}"
            class="w-full h-auto object-contain image-pixelated"
            onerror="this.src='/images/faces/toriel.png'; this.style.opacity='0.5'" 
          />
        </div>
        <div class="flex-1 ${fontClass} text-2xl text-white pt-0 -mt-4 leading-relaxed">
          ${renderedText}
        </div>
      </div>
    `;
  }

  // Fallback to default fence renderer (likely highlight.js)
  return defaultFence ? defaultFence(tokens, idx, options, env, self) : `<pre><code class="hljs">${md.utils.escapeHtml(token.content)}</code></pre>`;
};

md.renderer.rules.code_inline = (tokens, idx) => {
  const content = tokens[idx].content;
  return `<code class="bg-[#002200] px-2 py-0.5 text-xl border border-green-800 rounded-sm font-vt323 tracking-wider">${md.utils.escapeHtml(content)}</code>`;
};

md.renderer.rules.blockquote_open = () => {
  return '<div class="my-8 p-4 border-2 border-white bg-blue-900/30"><blockquote class="italic text-xl text-blue-100 flex-1">';
};

md.renderer.rules.blockquote_close = () => {
  return '</blockquote></div>';
};

md.renderer.rules.image = (tokens, idx) => {
  const token = tokens[idx];
  const src = token.attrGet('src');
  const alt = token.content;
  return `<img src="${src}" alt="${alt}" class="border-4 border-white my-6 w-full max-w-lg mx-auto image-pixelated" />`;
};

md.renderer.rules.table_open = (_tokens, _idx, _options, _env, _self) => {
  return '<div class="overflow-x-auto my-8 border-2 border-gray-600 shadow-[4px_4px_0px_0px_rgba(255,255,255,0.2)]"><table class="w-full border-collapse bg-black text-left">';
};

md.renderer.rules.table_close = (_tokens, _idx, _options, _env, _self) => {
  return '</table></div>';
};

md.renderer.rules.thead_open = () => {
  return '<thead class="bg-gray-900 border-b-2 border-white">';
};

md.renderer.rules.th_open = () => {
  return '<th class="p-4 text-lg font-speechbubble text-yellow-300 tracking-wider border-r border-gray-700 last:border-r-0">';
};

md.renderer.rules.td_open = () => {
  return '<td class="border-t border-gray-800 p-3 text-xl text-gray-300 border-r border-gray-800 last:border-r-0">';
};

md.renderer.rules.link_open = (tokens, idx) => {
  const href = tokens[idx].attrGet('href');
  return `<a href="${href}" class="text-yellow-300 hover:text-white hover:underline decoration-2 underline-offset-4 transition-colors cursor-pointer">`;
};

const renderedContent = computed(() => {
  return md.render(props.content);
});
</script>

<style scoped>
.typewriter::after {
  content: '';
  display: inline-block;
  width: 1rem;
  height: 2rem;
  background-color: white;
  margin-left: 0.5rem;
  animation: blink 0.8s infinite;
  vertical-align: middle;
}

@keyframes blink {
  0%, 100% {
    opacity: 0;
  }
  50% {
    opacity: 1;
  }
}
</style>
