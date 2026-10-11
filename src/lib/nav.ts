export const NAV_ITEMS = [
  { path: "/home", label: "Home" },
  { path: "/projects", label: "Projects" },
  { path: "/workspaces", label: "Workspaces" },
  { path: "/config", label: "Config Catalog" },
  { path: "/vault", label: "Vault" },
  { path: "/notes", label: "Notes" },
  { path: "/settings", label: "Settings" },
] as const;

export function normalizePath(path: string): string {
  if (path === "") return "/";
  if (path.length > 1 && path.endsWith("/")) return path.slice(0, -1);
  return path;
}

export function isNavActive(currentPath: string, itemPath: string): boolean {
  return normalizePath(currentPath) === normalizePath(itemPath);
}
