import { describe, expect, it } from "vitest";
import { NAV_ITEMS, isNavActive, normalizePath } from "../src/lib/nav";

describe("NAV_ITEMS", () => {
  it("lists labels and paths in AC-02 order", () => {
    expect(NAV_ITEMS.map((i) => [i.label, i.path])).toEqual([
      ["Home", "/home"],
      ["Projects", "/projects"],
      ["Workspaces", "/workspaces"],
      ["Config Catalog", "/config"],
      ["Vault", "/vault"],
      ["Secrets", "/secrets"],
      ["Notes", "/notes"],
      ["Settings", "/settings"],
    ]);
  });
});

describe("normalizePath", () => {
  it("strips one trailing slash", () => {
    expect(normalizePath("/home/")).toBe("/home");
  });
  it("maps empty to root", () => {
    expect(normalizePath("")).toBe("/");
  });
});

describe("isNavActive", () => {
  it("matches equal paths", () => {
    expect(isNavActive("/projects", "/projects")).toBe(true);
  });
  it("ignores trailing slash", () => {
    expect(isNavActive("/projects/", "/projects")).toBe(true);
  });
  it("rejects different paths", () => {
    expect(isNavActive("/projects", "/home")).toBe(false);
  });
  it("does not prefix match", () => {
    expect(isNavActive("/projectsX", "/projects")).toBe(false);
  });
});
