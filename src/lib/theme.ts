// Appearance preference (Dark / Light).
//
// Stored in the webview's localStorage under "devshelf.theme". This is a
// non-sensitive UI preference only; never store secrets or user data here.

export type Theme = "dark" | "light";

export const THEME_STORAGE_KEY = "devshelf.theme";

export interface ThemeStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export interface ThemeRoot {
  dataset: { theme?: string | undefined };
}

export function readTheme(storage: Pick<ThemeStorage, "getItem">): Theme {
  let value: string | null;
  try {
    value = storage.getItem(THEME_STORAGE_KEY);
  } catch {
    return "dark";
  }
  return value === "light" ? "light" : "dark";
}

export function writeTheme(storage: ThemeStorage, root: ThemeRoot, theme: Theme): void {
  root.dataset.theme = theme;
  try {
    storage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // Storage unavailable: the theme still applies for this session.
  }
}

export function getTheme(): Theme {
  return readTheme(window.localStorage);
}

export function setTheme(theme: Theme): void {
  writeTheme(window.localStorage, document.documentElement, theme);
}

export function applyStoredTheme(): void {
  document.documentElement.dataset.theme = getTheme();
}
