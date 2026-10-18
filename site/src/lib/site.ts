export const REPO_URL = "https://github.com/KhoaLy2003/devshelf";
export const RELEASES_URL = `${REPO_URL}/releases/latest`;
export const CHANGELOG_URL = `${REPO_URL}/blob/main/CHANGELOG.md`;

export function shot(name: string): string {
  return `${import.meta.env.BASE_URL}screenshots/${name}.png`;
}

export function asset(name: string): string {
  return `${import.meta.env.BASE_URL}${name}`;
}
