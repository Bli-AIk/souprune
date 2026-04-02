/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module 'virtual:docs' {
  export interface DocItem {
    id: string;
    title: string;
    category: string;
    content: string;
    contentSerious?: string;
    metadata?: Record<string, any>;
    metadataSerious?: Record<string, any>;
  }

  export interface NavItem {
    id: string;
    label: string;
    category: string;
    font?: string;
  }

  export const DOCS_DATA: Record<string, DocItem[]>;
  export const NAV_ITEMS: Record<string, NavItem[]>;
  export const SOUPRUNE_VERSION: string;
}