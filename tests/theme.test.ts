import { describe, expect, it } from "vitest";
import { THEME_STORAGE_KEY, readTheme, writeTheme } from "../src/lib/theme";

function fakeStorage(initial: Record<string, string> = {}) {
  const data = new Map(Object.entries(initial));
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => {
      data.set(key, value);
    },
  };
}

describe("readTheme", () => {
  it("defaults to dark when missing", () => {
    expect(readTheme(fakeStorage())).toBe("dark");
  });
  it("defaults to dark for an invalid value", () => {
    expect(readTheme(fakeStorage({ [THEME_STORAGE_KEY]: "purple" }))).toBe("dark");
  });
  it("returns light when stored", () => {
    expect(readTheme(fakeStorage({ [THEME_STORAGE_KEY]: "light" }))).toBe("light");
  });
});

describe("writeTheme", () => {
  it("writes storage and sets the dataset attribute", () => {
    const storage = fakeStorage();
    const root: { dataset: { theme?: string } } = { dataset: {} };
    writeTheme(storage, root, "light");
    expect(storage.getItem(THEME_STORAGE_KEY)).toBe("light");
    expect(root.dataset.theme).toBe("light");
    writeTheme(storage, root, "dark");
    expect(storage.getItem(THEME_STORAGE_KEY)).toBe("dark");
    expect(root.dataset.theme).toBe("dark");
  });
});
