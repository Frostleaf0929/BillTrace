// v2 图标库（prototype.html 移植）：单色线性 SVG，currentColor
export const ICONS: Record<string, string> = {
  overview: '<rect x="3.5" y="3.5" width="7" height="7" rx="2"/><rect x="13.5" y="3.5" width="7" height="7" rx="2"/><rect x="3.5" y="13.5" width="7" height="7" rx="2"/><rect x="13.5" y="13.5" width="7" height="7" rx="2"/>',
  list: '<path d="M9 6h11M9 12h11M9 18h11"/><circle cx="4.5" cy="6" r="1.3"/><circle cx="4.5" cy="12" r="1.3"/><circle cx="4.5" cy="18" r="1.3"/>',
  stats: '<path d="M5.5 20V11M12 20V4.5M18.5 20v-5.5"/>',
  budget: '<circle cx="12" cy="12" r="8.2"/><circle cx="12" cy="12" r="4.2"/><circle cx="12" cy="12" r=".8"/>',
  cats: '<path d="M3.5 7.2A2.2 2.2 0 0 1 5.7 5h3.6l2 2.2h7A2.2 2.2 0 0 1 20.5 9.4v7.4a2.2 2.2 0 0 1-2.2 2.2H5.7a2.2 2.2 0 0 1-2.2-2.2z"/>',
  gear: '<path d="M12 8.6a3.4 3.4 0 1 0 0 6.8 3.4 3.4 0 0 0 0-6.8z"/><path d="M19.4 13.9a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1.03 1.56V20a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1.11-1.56 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.56-1.03H4a2 2 0 1 1 0-4h.09A1.7 1.7 0 0 0 5.65 7.8a1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.87.34h.08a1.7 1.7 0 0 0 1.03-1.56V2a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1.03 1.56 1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87v.08a1.7 1.7 0 0 0 1.56 1.03H20a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.51.89z"/>',
  moon: '<path d="M20.6 13.2A8.4 8.4 0 1 1 10.8 3.4a6.6 6.6 0 0 0 9.8 9.8z"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2.8v2M12 19.2v2M2.8 12h2M19.2 12h2M5.2 5.2l1.5 1.5M17.3 17.3l1.5 1.5M18.8 5.2l-1.5 1.5M6.7 17.3l-1.5 1.5"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="M16 16l5 5"/>',
  cal: '<rect x="3.5" y="5" width="17" height="15.5" rx="2.5"/><path d="M8 3v4M16 3v4M3.5 10.5h17"/>',
  up: '<path d="M7 17L17 7M9 7h8v8"/>',
  down: '<path d="M17 7L7 17M15 17H7V9"/>',
  trash: '<path d="M4.5 7h15M9.5 7V5.2A1.2 1.2 0 0 1 10.7 4h2.6a1.2 1.2 0 0 1 1.2 1.2V7M6.5 7l.9 12a1.5 1.5 0 0 0 1.5 1.4h6.2a1.5 1.5 0 0 0 1.5-1.4l.9-12"/>',
  edit: '<path d="M4.5 19.5l3.8-.9L19.6 7.3a1.9 1.9 0 0 0-2.7-2.7L5.6 15.9z"/>',
  alert: '<path d="M12 9v4.5M12 16.8v.2"/><path d="M10.3 4.3L2.9 17.4a1.9 1.9 0 0 0 1.7 2.9h14.8a1.9 1.9 0 0 0 1.7-2.9L13.7 4.3a1.9 1.9 0 0 0-3.4 0z"/>',
  chevR: '<path d="M9.5 6l6 6-6 6"/>',
  arrow: '<path d="M5 12h13M13 6.5L18.5 12 13 17.5"/>',
  wallet: '<rect x="3" y="6.5" width="18" height="13" rx="3"/><path d="M3 10.5h18M16.5 15h.01"/>',
  exit: '<path d="M12 3.5v7.5"/><path d="M6.8 6.6a7.4 7.4 0 1 0 10.4 0"/>',
  dl: '<path d="M12 4v11M8 11l4 4 4-4"/><path d="M4.5 15.5v2.5a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2v-2.5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  check: '<path d="M5 12l5 5L20 7"/>',
  x: '<path d="M6 6l12 12M18 6L6 18"/>',
};

export function icon(name: string, size = 20): string {
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">${ICONS[name] ?? ''}</svg>`;
}

// 分类稳定图标：hash 从池子里挑（用户分类是动态数据，无法一一指定）
const CAT_ICON_POOL = ['cfood', 'ctrans', 'cshop', 'chome', 'cfun', 'cstudy', 'cmed', 'cgift'];
const CAT_ICONS: Record<string, string> = {
  cfood: '<path d="M4.5 12.5a7.5 7.5 0 0 0 15 0z"/><path d="M9 8.8V6.8M13 8.8V5.6M17 8.8V7.2"/>',
  ctrans: '<rect x="4.5" y="4.5" width="15" height="12.5" rx="2.5"/><path d="M4.5 10.5h15M8 20l.7-3M16 20l-.7-3"/><circle cx="8.6" cy="14" r=".7"/><circle cx="15.4" cy="14" r=".7"/>',
  cshop: '<path d="M6 8.5h12l-1 10.7a1.8 1.8 0 0 1-1.8 1.6H8.8A1.8 1.8 0 0 1 7 19.2z"/><path d="M9 10.5V7a3 3 0 0 1 6 0v3.5"/>',
  chome: '<path d="M4.5 11.5L12 4.5l7.5 7"/><path d="M6.5 10.5V19a1 1 0 0 0 1 1h9a1 1 0 0 0 1-1v-8.5"/><path d="M10 20v-5.5h4V20"/>',
  cfun: '<circle cx="12" cy="12" r="8.2"/><path d="M10.2 8.9l4.8 3.1-4.8 3.1z"/>',
  cstudy: '<path d="M12 6.3c-1.8-1.3-4.3-1.9-8-1.9v13.2c3.7 0 6.2.6 8 1.9 1.8-1.3 4.3-1.9 8-1.9V4.4c-3.7 0-6.2.6-8 1.9z"/><path d="M12 6.3v13.2"/>',
  cmed: '<rect x="4" y="4" width="16" height="16" rx="4.5"/><path d="M12 8.5v7M8.5 12h7"/>',
  cgift: '<rect x="4.5" y="11" width="15" height="8.5" rx="1.5"/><path d="M12 11v8.5M4.5 14.8h15"/><path d="M12 11c-3.8 0-4.8-1.5-4.4-2.8.4-1.5 2.6-1.3 3.4.2.6 1.1 1 2.6 1 2.6zm0 0c3.8 0 4.8-1.5 4.4-2.8-.4-1.5-2.6-1.3-3.4.2-.6 1.1-1 2.6-1 2.6z"/>',
};
export function catIcon(name: string, size = 15): string {
  if (!name) return icon('cats', size);
  let h = 0;
  for (const ch of name) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  const key = CAT_ICON_POOL[h % CAT_ICON_POOL.length];
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">${CAT_ICONS[key]}</svg>`;
}

// 分类稳定色（柔和去饱和板，与旧版 SOFT_PALETTE 同思路）
export const CAT_PALETTE = ['#877FC1', '#58B3C4', '#6FA98C', '#C9954E', '#C9797B', '#9A8FC9', '#7AA3B5', '#B08FC9', '#8FB37E', '#C9A06A', '#7D92CF', '#C98BA0'];
export function catColor(name: string): string {
  if (!name) return 'var(--zj-border)';
  let h = 0;
  for (const ch of name) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return CAT_PALETTE[h % CAT_PALETTE.length];
}
