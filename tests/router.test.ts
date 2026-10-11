import { createMemoryRouter, Navigate } from "react-router";
import { describe, expect, it } from "vitest";
import { routes } from "../src/routes";

function leafId(path: string): string | undefined {
  const router = createMemoryRouter(routes, { initialEntries: [path] });
  const matches = router.state.matches;
  const id = matches[matches.length - 1]?.route.id;
  router.dispose();
  return id;
}

function locationOf(path: string): string {
  const router = createMemoryRouter(routes, { initialEntries: [path] });
  const pathname = router.state.location.pathname;
  router.dispose();
  return pathname;
}

describe("routes", () => {
  it("matches / to the index route whose element redirects to /home", () => {
    const router = createMemoryRouter(routes, { initialEntries: ["/"] });
    const route = router.state.matches[router.state.matches.length - 1]?.route;
    router.dispose();
    expect(route?.id).toBe("root");
    const element = route?.element as { type: unknown; props: { to: string; replace?: boolean } };
    expect(element.type).toBe(Navigate);
    expect(element.props.to).toBe("/home");
    expect(element.props.replace).toBe(true);
  });

  it("resolves /projects/ to projects", () => {
    expect(leafId("/projects/")).toBe("projects");
  });

  it("resolves /projects?x=1 to projects", () => {
    expect(leafId("/projects?x=1")).toBe("projects");
    expect(locationOf("/projects?x=1")).toBe("/projects");
  });

  it("resolves unknown paths to not-found", () => {
    expect(leafId("/nope")).toBe("not-found");
  });
});
