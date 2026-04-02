<template>
  <div class="min-h-screen bg-black text-white font-vt323 relative overflow-hidden cyber-grid selection:bg-yellow-500 selection:text-black">
    
    <!-- Top Status Bar -->
    <header class="fixed top-0 left-0 right-0 h-16 bg-black border-b-4 border-white z-50 px-4 flex items-center justify-between shadow-[0_4px_0_rgba(0,0,0,0.5)]">
      <div class="flex items-center gap-4">
        <span class="text-xl text-yellow-400 tracking-widest">SOUPRUNE_DOCS</span>
        <div class="hidden md:flex gap-4 text-xs md:text-sm text-gray-400 font-pixel">
          <span class="flex items-center gap-2"><span class="text-white">LV</span> {{ day }}</span>
          <span class="flex items-center gap-2"><span class="text-white">HP</span> {{ time }}</span>
          <span class="text-yellow-300">G {{ milliseconds }}</span>
        </div>
      </div>
      <div class="flex items-center gap-4 md:gap-6">
        <!-- Serious Mode Toggle -->
        <button 
          @click="isSerious = !isSerious" 
          class="transition-colors"
          :class="isSerious ? 'text-yellow-300 hover:text-white' : 'text-white hover:text-yellow-300'"
          :title="isSerious ? 'Switch to Lively Mode' : 'Switch to Serious Mode'"
        >
          <Briefcase :size="20" />
        </button>

        <!-- Language Switcher -->
        <button 
          @click="toggleLang" 
          class="font-pixel text-yellow-300 hover:text-white border-2 border-yellow-300 hover:border-white px-2 py-1 text-xs transition-colors"
        >
          {{ currentLang === 'en' ? 'ZH' : 'EN' }}
        </button>

        <!-- Social Icons -->
        <div class="flex items-center gap-4">
          <a href="https://github.com/Bli-AIk/souprune/" target="_blank" rel="noopener noreferrer" class="text-white hover:text-yellow-300 transition-colors">
            <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 fill-current"><title>GitHub</title><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>
          </a>
          <a href="https://discord.gg/5YXK5DRjPZ" target="_blank" rel="noopener noreferrer" class="text-white hover:text-yellow-300 transition-colors">
            <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 fill-current"><title>Discord</title><path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/></svg>
          </a>
        </div>

        <!-- Mobile Menu Button -->
        <button @click="menuOpen = !menuOpen" class="md:hidden text-white hover:text-yellow-300 transition-colors">
          <X v-if="menuOpen" :size="24" />
          <Menu v-else :size="24" />
        </button>
      </div>
    </header>

    <div class="pt-20 pb-8 px-2 md:px-8 max-w-[1800px] mx-auto h-[calc(100vh)] flex flex-col md:flex-row gap-6 relative z-10">
      
      <!-- Left Column: The "Menu" -->
      <nav 
        :class="[
          'fixed md:static inset-0 top-16 bg-black/95 md:bg-transparent z-40',
          'flex flex-col gap-6 w-full md:w-80 shrink-0 transition-transform duration-300 overflow-hidden',
          menuOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'
        ]"
      >
        <Transition name="nav-mode-switch" mode="out-in">
        <div class="p-4 md:p-0 overflow-y-auto h-full" :key="isSerious ? 'serious' : 'lively'">
          <!-- User Card -->
          <div class="border-4 border-white bg-black p-4 mb-6 shadow-[8px_8px_0px_0px_rgba(255,255,255,0.2)]">
            <div class="flex items-center gap-4 mb-2">
              <div class="w-12 h-12 bg-green-500 border-2 border-white"></div>
              <div>
                <div class="text-lg text-yellow-300">RALSEI</div>
                <div class="font-pixel text-sm text-gray-300">Mage / Doc Writer</div>
              </div>
            </div>
            <div class="w-full h-4 bg-red-900 border border-white mt-2 relative">
              <div 
                class="absolute top-0 left-0 h-full bg-yellow-400 transition-all duration-300 ease-in-out" 
                :style="{ width: scrollProgress + '%' }"
              ></div>
            </div>
          </div>

          <!-- Navigation Groups -->
          <div class="space-y-6">
            <NavGroup 
              v-for="(items, category) in groupedNav"
              :key="category"
              :title="category.toUpperCase()" 
              :icon="getIcon(category as string)" 
              :items="items" 
              :activeId="activeId" 
              @select="handleNavSelect" 
            />
          </div>
          

        </div>
        </Transition>
      </nav>

      <!-- Right Column: The "Content Box" -->
      <main 
        class="flex-1 min-w-0 relative h-full flex flex-col overflow-hidden"
        @touchstart="onTouchStart"
        @touchmove="onTouchMove"
        @touchend="onTouchEnd"
      >
        <Transition
          :name="transitionName"
          mode="out-in"
        >
          <div 
            v-if="activeDoc"
            :key="activeDoc.id + (isSerious ? '-serious' : '')"
            class="flex-1 flex flex-col h-full pb-16 md:pb-0"
          >
            <div class="hud-box flex-1 flex flex-col relative overflow-hidden">
              
              <!-- Scrollable Area -->
              <div 
                ref="contentScrollContainer"
                class="p-4 md:p-12 overflow-y-auto flex-1 custom-scrollbar relative"
              >
                <!-- Version Mismatch Warning (top of page) -->
                <div v-if="versionStatus === 'stale' && activeMetadata" class="mb-6 border-2 border-yellow-700 bg-yellow-900/20 px-4 py-3 text-sm font-pixel text-yellow-400">
                  <template v-if="currentLang === 'zh-hans'">
                    此文档基于 <span class="text-red-400">v{{ activeMetadata.version }}</span> 编写，当前框架版本为 <span class="text-green-400">v{{ SOUPRUNE_VERSION }}</span>。内容可能已过时。
                    [<span class="text-red-400">v{{ activeMetadata.version }}</span> → <span class="text-green-400">v{{ SOUPRUNE_VERSION }}</span>]
                  </template>
                  <template v-else>
                    This doc was written for <span class="text-red-400">v{{ activeMetadata.version }}</span>, but the current framework version is <span class="text-green-400">v{{ SOUPRUNE_VERSION }}</span>. Content may be outdated.
                    [<span class="text-red-400">v{{ activeMetadata.version }}</span> → <span class="text-green-400">v{{ SOUPRUNE_VERSION }}</span>]
                  </template>
                </div>
                <div v-if="versionStatus === 'ahead' && activeMetadata" class="mb-6 border-2 border-cyan-700 bg-cyan-900/20 px-4 py-3 text-sm font-pixel text-cyan-400">
                  <template v-if="currentLang === 'zh-hans'">
                    此文档基于 <span class="text-green-400">v{{ activeMetadata.version }}</span> 编写，领先于当前框架版本 <span class="text-red-400">v{{ SOUPRUNE_VERSION }}</span>。部分内容可能尚未实现。
                    [<span class="text-red-400">v{{ SOUPRUNE_VERSION }}</span> → <span class="text-green-400">v{{ activeMetadata.version }}</span>]
                  </template>
                  <template v-else>
                    This doc targets <span class="text-green-400">v{{ activeMetadata.version }}</span>, ahead of the current framework <span class="text-red-400">v{{ SOUPRUNE_VERSION }}</span>. Some features may not be implemented yet.
                    [<span class="text-red-400">v{{ SOUPRUNE_VERSION }}</span> → <span class="text-green-400">v{{ activeMetadata.version }}</span>]
                  </template>
                </div>
                <MarkdownRenderer :content="(isSerious && activeDoc?.contentSerious) ? activeDoc.contentSerious : (activeDoc?.content || '')" @navigate-doc="handleDocLink" />
                
                <!-- Document Metadata (bottom of page) -->
                <div v-if="activeMetadata" class="mt-12 border-2 border-gray-700 bg-gray-900/50 px-4 py-3 text-sm font-pixel">
                  <div class="flex flex-wrap items-center gap-x-6 gap-y-2 text-gray-400">
                    <span v-if="activeMetadata.author" class="flex items-center gap-1">
                      <span class="text-gray-500">Author</span>
                      <span class="text-white">{{ activeMetadata.author }}</span>
                    </span>
                    <span v-if="activeMetadata.version" class="flex items-center gap-1">
                      <span class="text-gray-500">Doc</span>
                      <span :class="versionStatus === 'stale' ? 'text-red-400' : versionStatus === 'ahead' ? 'text-green-400' : 'text-white'">v{{ activeMetadata.version }}</span>
                    </span>
                    <span v-if="activeMetadata.version" class="flex items-center gap-1">
                      <span class="text-gray-500">Framework</span>
                      <span :class="versionStatus === 'stale' ? 'text-green-400' : versionStatus === 'ahead' ? 'text-red-400' : 'text-white'">v{{ SOUPRUNE_VERSION }}</span>
                    </span>
                    <span v-if="activeMetadata.date" class="flex items-center gap-1">
                      <span class="text-gray-500">Date</span>
                      <span class="text-white">{{ activeMetadata.date }}</span>
                    </span>
                    <div v-if="activeMetadata.tags?.length" class="flex items-center gap-1 flex-wrap">
                      <span v-for="tag in activeMetadata.tags" :key="tag" class="bg-gray-800 text-yellow-300 px-2 py-0.5 text-xs border border-gray-600">{{ tag }}</span>
                    </div>
                  </div>
                </div>

                <!-- Page Footer with Navigation Hints -->
                <div class="mt-16 pt-8 border-t-2 border-dashed border-gray-700 flex justify-between text-gray-500 text-xl items-center">
                  <button 
                    @click="navigate('prev')" 
                    class="hover:text-white flex items-center gap-2 transition-colors md:hidden"
                    :disabled="flatNavOrder.length === 0 || activeId === flatNavOrder[0].id"
                  >
                    <ChevronLeft /> PREV
                  </button>
                  
                  <span class="hidden md:inline">PAGE_{{ activeDoc?.id.toUpperCase() || 'UNKNOWN' }}</span>
                  <span class="hidden md:inline">(PRESS Z TO PROCEED)</span>

                  <button 
                    @click="navigate('next')" 
                    class="hover:text-white flex items-center gap-2 transition-colors md:hidden"
                    :disabled="flatNavOrder.length === 0 || activeId === flatNavOrder[flatNavOrder.length - 1].id"
                  >
                    NEXT <ChevronRight />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </Transition>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue';
import { Menu, X, Shield, ChevronLeft, ChevronRight, Utensils, Flame, Map, Sparkles, FlaskConical, Scroll, Briefcase } from 'lucide-vue-next';
import { NAV_ITEMS as ALL_NAV_ITEMS, DOCS_DATA as ALL_DOCS_DATA, SOUPRUNE_VERSION } from 'virtual:docs';
import MarkdownRenderer from './components/MarkdownRenderer.vue';
import NavGroup from './components/NavGroup.vue';
import { DocPage, DocMetadata, NavItem } from './types';
import { SERIOUS_TITLES } from './titles';

// Cast the imported data to Record<string, ...>
const navItemsMap = ALL_NAV_ITEMS as Record<string, NavItem[]>;
const docsDataMap = ALL_DOCS_DATA as Record<string, DocPage[]>;

const currentLang = ref('en');
const isSerious = ref(false);
const activeId = ref<string>('intro');
const menuOpen = ref(false);
const direction = ref(0); // -1 for prev, 1 for next
const transitionName = ref('slide-left');
let suppressHashUpdate = false;

// Parse URL hash: #/lang/docId or #/lang/docId/serious
const parseHash = (): { lang?: string; id?: string; serious?: boolean } => {
  const hash = window.location.hash.replace(/^#\/?/, '');
  if (!hash) return {};
  const parts = hash.split('/');
  if (parts.length >= 3 && parts[parts.length - 1] === 'serious') {
    return { lang: parts[0], id: parts.slice(1, -1).join('/'), serious: true };
  }
  if (parts.length >= 2) {
    return { lang: parts[0], id: parts.slice(1).join('/') };
  }
  return { id: parts[0] };
};

// Update URL hash from current state
const updateHash = () => {
  if (suppressHashUpdate) return;
  let newHash = `#/${currentLang.value}/${activeId.value}`;
  if (isSerious.value) newHash += '/serious';
  if (window.location.hash !== newHash) {
    history.replaceState(null, '', newHash);
  }
};

// Computed data based on language
const currentDocsData = computed(() => docsDataMap[currentLang.value] || []);
const currentNavItems = computed(() => {
  const items = navItemsMap[currentLang.value] || [];
  if (!isSerious.value) return items;

  // If serious mode, map labels and categories
  return items.map(item => ({
    ...item,
    label: SERIOUS_TITLES[currentLang.value]?.[item.id] || item.label,
    category: SERIOUS_TITLES[currentLang.value]?.[item.category] || item.category
  }));
});

// Active doc based on ID and current data
const activeDoc = computed(() => currentDocsData.value.find(d => d.id === activeId.value));

// Active metadata (switches between normal and serious mode metadata)
const activeMetadata = computed((): DocMetadata | undefined => {
  if (!activeDoc.value) return undefined;
  if (isSerious.value && activeDoc.value.metadataSerious) {
    return activeDoc.value.metadataSerious as DocMetadata;
  }
  return activeDoc.value.metadata as DocMetadata | undefined;
});

// Version comparison status
const versionStatus = computed((): 'match' | 'stale' | 'ahead' | null => {
  const meta = activeMetadata.value;
  if (!meta?.version) return null;
  
  const docParts = meta.version.split('.').map(Number);
  const curParts = SOUPRUNE_VERSION.split('.').map(Number);
  
  for (let i = 0; i < Math.max(docParts.length, curParts.length); i++) {
    const d = docParts[i] || 0;
    const c = curParts[i] || 0;
    if (d < c) return 'stale';
    if (d > c) return 'ahead';
  }
  return 'match';
});

// Toggle Language
const toggleLang = () => {
  currentLang.value = currentLang.value === 'en' ? 'zh-hans' : 'en';
};

// Time-related state
const day = ref('');
const time = ref('');
const milliseconds = ref('');
let timeInterval: any;
let animationFrameId: any;
const scrollProgress = ref(0); // This will become global progress

const updateMilliseconds = () => {
  milliseconds.value = String(new Date().getMilliseconds()).padStart(3, '0');
  animationFrameId = requestAnimationFrame(updateMilliseconds);
};

// No updateScrollProgress function anymore

onMounted(() => {
  // Initialize from URL hash
  const { lang, id, serious } = parseHash();
  if (lang && (lang === 'en' || lang === 'zh-hans')) {
    currentLang.value = lang;
  }
  if (id) {
    const exists = (docsDataMap[currentLang.value] || []).some(d => d.id === id);
    if (exists) activeId.value = id;
  }
  if (serious) {
    isSerious.value = true;
  }
  updateHash();

  // Listen for browser back/forward
  const onHashChange = () => {
    const parsed = parseHash();
    suppressHashUpdate = true;
    if (parsed.lang && (parsed.lang === 'en' || parsed.lang === 'zh-hans')) {
      currentLang.value = parsed.lang;
    }
    if (parsed.id) {
      const exists = (docsDataMap[currentLang.value] || []).some(d => d.id === parsed.id);
      if (exists) activeId.value = parsed.id!;
    }
    isSerious.value = !!parsed.serious;
    suppressHashUpdate = false;
  };
  window.addEventListener('hashchange', onHashChange);
  window.addEventListener('popstate', onHashChange);

  timeInterval = setInterval(() => {
    const now = new Date();
    day.value = String(now.getDate()).padStart(2, '0');
    time.value = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
  }, 1000);
  updateMilliseconds();
  // Initial global progress calculation
  updateGlobalProgress();
});

onUnmounted(() => {
  clearInterval(timeInterval);
  cancelAnimationFrame(animationFrameId);
});

// Swipe handling state
const touchStartX = ref<number | null>(null);
const touchEndX = ref<number | null>(null);
const MIN_SWIPE_DISTANCE = 50;

// Content scroll container ref
const contentScrollContainer = ref<HTMLElement | null>(null);

// Group items dynamically
const groupedNav = computed(() => {
  const groups: Record<string, NavItem[]> = {};
  
  // Get all unique categories from the items
  const categories = new Set<string>();
  currentNavItems.value.forEach(item => {
    categories.add(item.category);
    if (!groups[item.category]) groups[item.category] = [];
    groups[item.category].push(item);
  });
  
  // Sort categories: "Part X" first, then others (Appendix)
  const sortedCategories = Array.from(categories).sort((a, b) => {
    const getWeight = (str: string) => {
      const s = str.toLowerCase();
      if (s.includes('table of contents') || s.includes('目录') || s.includes('official documentation') || s.includes('官方文档')) return -1;
      if (s.includes('mise') || s.includes('备菜') || s.includes('environment setup') || s.includes('getting started') || s.includes('环境配置')) return 0;
      if (s.includes('spicy') || s.includes('主菜') || s.includes('battle system') || s.includes('战斗系统')) return 1;
      if (s.includes('plating') || s.includes('摆盘') || s.includes('world & narrative') || s.includes('世界场景')) return 2;
      if (s.includes('soul') || s.includes('甜点') || s.includes('visuals & audio') || s.includes('视听')) return 3;
      if (s.includes('molecular') || s.includes('分子') || s.includes('advanced scripting') || s.includes('高级脚本')) return 4;
      if (s.includes('appendix') || s.includes('附录')) return 99;
      return 50; // Unknown
    };

    return getWeight(a) - getWeight(b);
  });

  // Reconstruct object with sorted keys for v-for iteration
  const sortedGroups: Record<string, NavItem[]> = {};
  sortedCategories.forEach(key => {
    sortedGroups[key] = groups[key];
  });
  
  return sortedGroups;
});

const getIcon = (category: string) => {
  const cat = category.toLowerCase();
  if (cat.includes('table of contents') || cat.includes('目录') || cat.includes('official documentation') || cat.includes('官方文档')) return Shield;
  if (cat.includes('mise') || cat.includes('备菜') || cat.includes('environment setup') || cat.includes('getting started') || cat.includes('环境配置')) return Utensils;
  if (cat.includes('spicy') || cat.includes('主菜') || cat.includes('battle system') || cat.includes('战斗系统')) return Flame;
  if (cat.includes('plating') || cat.includes('摆盘') || cat.includes('world & narrative') || cat.includes('世界场景')) return Map;
  if (cat.includes('soul') || cat.includes('甜点') || cat.includes('visuals & audio') || cat.includes('视听')) return Sparkles;
  if (cat.includes('molecular') || cat.includes('分子') || cat.includes('advanced scripting') || cat.includes('高级脚本')) return FlaskConical;
  if (cat.includes('appendix') || cat.includes('附录')) return Scroll;
  return Shield;
};

// Flatten navigation for next/prev logic
const flatNavOrder = computed(() => currentNavItems.value);

const updateGlobalProgress = () => {
  const currentIndex = flatNavOrder.value.findIndex(item => item.id === activeId.value);
  if (currentIndex !== -1 && flatNavOrder.value.length > 0) {
    // Calculate progress as (current_article_index + 1) / total_articles * 100
    scrollProgress.value = ((currentIndex + 1) / flatNavOrder.value.length) * 100;
  } else {
    scrollProgress.value = 0;
  }
};

watch(activeId, async () => {
  await nextTick();
  if (contentScrollContainer.value) {
    contentScrollContainer.value.scrollTop = 0;
  }
  updateGlobalProgress();
  updateHash();
});

// Also watch language change to ensure we don't get stuck on invalid ID if sets differ
watch(currentLang, () => {
    // If the current activeId doesn't exist in the new language, reset to first item
    const exists = currentDocsData.value.some(d => d.id === activeId.value);
    if (!exists && currentDocsData.value.length > 0) {
        activeId.value = currentDocsData.value[0].id;
    }
    updateGlobalProgress();
    updateHash();
});

watch(isSerious, () => {
  transitionName.value = 'mode-switch';
  updateHash();
});

const buildHash = (lang: string, id: string) => {
  let h = `#/${lang}/${id}`;
  if (isSerious.value) h += '/serious';
  return h;
};

const navigate = (dir: 'next' | 'prev') => {
  const currentIndex = flatNavOrder.value.findIndex(item => item.id === activeId.value);
  if (currentIndex === -1) return;

  let nextIndex = dir === 'next' ? currentIndex + 1 : currentIndex - 1;

  // Clamp for documentation
  if (nextIndex >= 0 && nextIndex < flatNavOrder.value.length) {
    direction.value = dir === 'next' ? 1 : -1;
    transitionName.value = dir === 'next' ? 'slide-left' : 'slide-right';
    const newId = flatNavOrder.value[nextIndex].id;
    history.pushState(null, '', buildHash(currentLang.value, newId));
    suppressHashUpdate = true;
    activeId.value = newId;
    suppressHashUpdate = false;
  }
};

const handleNavSelect = (id: string) => {
  const currentIndex = flatNavOrder.value.findIndex(item => item.id === activeId.value);
  const nextIndex = flatNavOrder.value.findIndex(item => item.id === id);
  
  direction.value = nextIndex > currentIndex ? 1 : -1;
  transitionName.value = nextIndex > currentIndex ? 'slide-left' : 'slide-right';
  history.pushState(null, '', buildHash(currentLang.value, id));
  suppressHashUpdate = true;
  activeId.value = id;
  suppressHashUpdate = false;
  menuOpen.value = false;
};

const handleDocLink = (docId: string) => {
  const exists = currentDocsData.value.some(d => d.id === docId);
  if (exists) {
    handleNavSelect(docId);
  }
};

const onTouchStart = (e: TouchEvent) => {
  touchStartX.value = e.targetTouches[0].clientX;
  touchEndX.value = null;
};

const onTouchMove = (e: TouchEvent) => {
  touchEndX.value = e.targetTouches[0].clientX;
};

const onTouchEnd = () => {
  if (touchStartX.value === null || touchEndX.value === null) return;
  
  const distance = touchStartX.value - touchEndX.value;
  const isLeftSwipe = distance > MIN_SWIPE_DISTANCE;
  const isRightSwipe = distance < -MIN_SWIPE_DISTANCE;

  if (isLeftSwipe) {
    navigate('next');
  } else if (isRightSwipe) {
    navigate('prev');
  }
  
  // Reset
  touchStartX.value = null;
  touchEndX.value = null;
};
</script>

<style>
/* Slide transitions */
.slide-left-enter-active,
.slide-left-leave-active,
.slide-right-enter-active,
.slide-right-leave-active {
  transition: all 0.3s ease;
}

.slide-left-enter-from {
  opacity: 0;
  transform: translateX(50px) scale(0.95);
}

.slide-left-leave-to {
  opacity: 0;
  transform: translateX(-50px) scale(0.95);
}

.slide-right-enter-from {
  opacity: 0;
  transform: translateX(-50px) scale(0.95);
}

.slide-right-leave-to {
  opacity: 0;
  transform: translateX(50px) scale(0.95);
}

/* Nav mode switch transition (sidebar: fly out/in beyond viewport) */
.nav-mode-switch-enter-active,
.nav-mode-switch-leave-active {
  transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.nav-mode-switch-leave-to {
  opacity: 0;
  transform: translateY(100vh);
}

.nav-mode-switch-enter-from {
  opacity: 0;
  transform: translateY(-100vh);
}

/* Mode switch transition — serious mode toggle (content: fly up out, fly down in) */
.mode-switch-enter-active,
.mode-switch-leave-active {
  transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.mode-switch-leave-to {
  opacity: 0;
  transform: translateY(-100vh);
}

.mode-switch-enter-from {
  opacity: 0;
  transform: translateY(100vh);
}

@keyframes spin-slow {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin-slow {
  animation: spin-slow 4s linear infinite;
}
</style>