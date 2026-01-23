export interface DocPage {
  id: string;
  title: string;
  content: string;
  contentSerious?: string;
  category: 'guide' | 'api' | 'examples';
}

export interface NavItem {
  id: string;
  label: string;
  category: string;
  font?: string;
}

export const ThemeColor = {
  NEON_GREEN: '#00ff00',
  PINK: '#ff00ff',
  YELLOW: '#ffff00',
  WHITE: '#ffffff',
  BLACK: '#000000',
  BLUE: '#00a2ff'
} as const;

export type ThemeColor = typeof ThemeColor[keyof typeof ThemeColor];
