<template>
  <div class="border-2 border-gray-700 bg-black/80">
    <div class="bg-gray-800 text-gray-200 p-2 flex items-center gap-2 font-speechbubble text-xs tracking-wider border-b-2 border-gray-700">
      <component :is="icon" :size="18" />
      {{ title }}
    </div>
    <ul class="p-2 space-y-1">
      <li v-for="item in items" :key="item.id">
        <button
          @click="$emit('select', item.id)"
          :class="[
            'w-full text-left text-2xl py-2 px-2 flex items-center group transition-all',
            activeId === item.id 
              ? 'text-yellow-300 bg-white/10' 
              : 'text-gray-400 hover:text-white hover:pl-4'
          ]"
        >
          <SoulCursor v-if="activeId === item.id" />
          <span v-else class="w-8"></span>
          <span :class="[
            activeId === item.id ? 'drop-shadow-[0_0_5px_rgba(255,255,0,0.5)]' : '',
            item.font === 'dtm-sans' ? 'font-ganon' : 'font-vt323'
          ]">
            {{ item.label }}
          </span>
        </button>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { NavItem } from '../types';
import SoulCursor from './SoulCursor.vue';

defineProps<{
  title: string;
  icon: any;
  items: NavItem[];
  activeId: string;
}>();

defineEmits<{
  select: [id: string];
}>();
</script>
