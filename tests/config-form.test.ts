import { describe, expect, it } from "vitest";
import {
  activeChips,
  applyFilters,
  environmentLabel,
  formatConfigError,
  kindFromFileName,
  kindLabel,
  statusReason,
  type ConfigFilters,
} from "../src/lib/config-form";
import {
  CONFIG_ENVIRONMENTS,
  CONFIG_KINDS,
  type ConfigCode,
  type ConfigEntryView,
} from "../src/types/commands";

describe("kindFromFileName", () => {
  it.each([
    ["docker-compose.yml", "docker_compose"],
    ["docker-compose.yaml", "docker_compose"],
    ["docker-compose.override.yml", "docker_compose"],
    ["app.yaml", "yaml"],
    ["application-staging.yml", "yaml"],
    ["app.properties", "properties"],
    [".env.staging", "dotenv"],
    [".env", "dotenv"],
    ["config.json", "json"],
    ["Caddyfile", "caddy_nginx"],
    ["nginx.conf", "caddy_nginx"],
    ["unknown.txt", null],
  ])("%s -> %s", (name, expected) => {
    expect(kindFromFileName(name)).toBe(expected);
  });

  it("is case-insensitive", () => {
    expect(kindFromFileName("APP.YML")).toBe("yaml");
    expect(kindFromFileName("CADDYFILE")).toBe("caddy_nginx");
  });
});

function entry(over: Partial<ConfigEntryView> & { id: number }): ConfigEntryView {
  return {
    project_id: 1,
    project_name: "Alpha",
    path: "F:\\work\\a.yml",
    name: "a.yml",
    environment: "local",
    kind: "yaml",
    status: "available",
    ...over,
  };
}

const NONE: ConfigFilters = { project: "all", environment: "all", kind: "all", status: "all" };

describe("applyFilters", () => {
  const entries = [
    entry({ id: 1 }),
    entry({ id: 2, environment: "staging", kind: "json", status: "missing" }),
    entry({ id: 3, project_name: "Beta", environment: "staging", status: "missing" }),
    entry({ id: 4, project_name: null, environment: "dev", kind: "dotenv" }),
  ];

  it("returns everything when every filter is all", () => {
    expect(applyFilters(entries, NONE).map((e) => e.id)).toEqual([1, 2, 3, 4]);
  });

  it("treats empty strings as no filter", () => {
    expect(
      applyFilters(entries, { project: "", environment: "", kind: "", status: "" }).length,
    ).toBe(4);
  });

  it("filters by project name", () => {
    expect(applyFilters(entries, { ...NONE, project: "Beta" }).map((e) => e.id)).toEqual([3]);
  });

  it("combines filters with AND", () => {
    const out = applyFilters(entries, { ...NONE, environment: "staging", status: "missing" });
    expect(out.map((e) => e.id)).toEqual([2, 3]);
    const narrower = applyFilters(entries, {
      project: "Alpha",
      environment: "staging",
      kind: "json",
      status: "missing",
    });
    expect(narrower.map((e) => e.id)).toEqual([2]);
  });

  it("returns empty when nothing matches", () => {
    expect(applyFilters(entries, { ...NONE, project: "Alpha", environment: "dev" })).toEqual([]);
  });

  it("the missing status filter also matches a folder at the path", () => {
    const withFolder = [entry({ id: 1, status: "folder_at_path" }), entry({ id: 2 })];
    expect(applyFilters(withFolder, { ...NONE, status: "missing" }).map((e) => e.id)).toEqual([1]);
  });

  it("does not mutate the input", () => {
    const copy = [...entries];
    applyFilters(entries, { ...NONE, kind: "json" });
    expect(entries).toEqual(copy);
  });
});

describe("activeChips", () => {
  it("is empty when no filter is active", () => {
    expect(activeChips(NONE)).toEqual([]);
    expect(activeChips({ project: "", environment: "", kind: "", status: "" })).toEqual([]);
  });

  it("lists active filters in project, environment, kind, status order", () => {
    expect(
      activeChips({
        project: "Alpha",
        environment: "staging",
        kind: "docker_compose",
        status: "missing",
      }),
    ).toEqual([
      { key: "project", label: "Project: Alpha" },
      { key: "environment", label: "Environment: staging" },
      { key: "kind", label: "Kind: Docker Compose" },
      { key: "status", label: "Status: Missing" },
    ]);
  });

  it("includes only the active ones", () => {
    expect(activeChips({ ...NONE, environment: "production" })).toEqual([
      { key: "environment", label: "Environment: production" },
    ]);
  });

  it("labels each status", () => {
    expect(activeChips({ ...NONE, status: "available" })[0]?.label).toBe("Status: Available");
    expect(activeChips({ ...NONE, status: "folder_at_path" })[0]?.label).toBe("Status: Missing");
    expect(activeChips({ ...NONE, status: "inaccessible" })[0]?.label).toBe("Status: Inaccessible");
  });
});

describe("statusReason", () => {
  it("available", () => {
    expect(statusReason("available")).toBe("The file is in place.");
  });
  it("missing", () => {
    expect(statusReason("missing")).toBe("The file was not found at this path.");
  });
  it("folder_at_path", () => {
    expect(statusReason("folder_at_path")).toBe("A folder is at this path, not a file.");
  });
  it("inaccessible", () => {
    expect(statusReason("inaccessible")).toBe(
      "The file can't be reached. The drive may not be ready or access is denied.",
    );
  });
});

describe("formatConfigError", () => {
  it("path_empty", () => {
    expect(formatConfigError("path_empty", null, "")).toBe("Enter a file path.");
  });
  it("path_not_found uses the detail path", () => {
    expect(formatConfigError("path_not_found", "F:\\work\\a.yml", "a.yml")).toBe(
      "`F:\\work\\a.yml` was not found.",
    );
  });
  it("path_not_found with unreachable detail gives the inaccessible sentence", () => {
    expect(formatConfigError("path_not_found", "unreachable", "a.yml")).toBe(
      "The file can't be reached. The drive may not be ready or access is denied.",
    );
  });
  it("path_is_folder", () => {
    expect(formatConfigError("path_is_folder", null, "")).toBe("This is a folder. Pick a file.");
  });
  it("path_relative", () => {
    expect(formatConfigError("path_relative", null, "")).toBe(
      "Enter an absolute path, for example `F:\\work\\application.yml`.",
    );
  });
  it("path_too_long", () => {
    expect(formatConfigError("path_too_long", null, "")).toBe("This path is too long for Windows.");
  });
  it("duplicate_path", () => {
    expect(formatConfigError("duplicate_path", "F:\\work\\a.yml", "a.yml")).toBe(
      "`F:\\work\\a.yml` is already in this project's config list.",
    );
  });
  it("project_not_found", () => {
    expect(formatConfigError("project_not_found", null, "")).toBe(
      "That project no longer exists. Choose another project.",
    );
  });
  it("environment_invalid", () => {
    expect(formatConfigError("environment_invalid", null, "")).toBe(
      "Pick an environment from the list.",
    );
  });
  it("kind_invalid", () => {
    expect(formatConfigError("kind_invalid", null, "")).toBe("Pick a kind from the list.");
  });
  it("config_not_found", () => {
    expect(formatConfigError("config_not_found", null, "")).toBe(
      "This config file is no longer in the catalog.",
    );
  });
  it("storage_unavailable", () => {
    expect(formatConfigError("storage_unavailable", null, "")).toBe(
      "DevShelf cannot reach its data folder. Restart DevShelf and try again.",
    );
  });

  it("covers all eleven codes with non-empty copy", () => {
    const codes: ConfigCode[] = [
      "config_not_found",
      "path_empty",
      "path_not_found",
      "path_is_folder",
      "path_relative",
      "path_too_long",
      "duplicate_path",
      "project_not_found",
      "environment_invalid",
      "kind_invalid",
      "storage_unavailable",
    ];
    for (const code of codes) {
      expect(formatConfigError(code, "x", "x").length).toBeGreaterThan(0);
    }
  });
});

describe("labels", () => {
  it("environmentLabel", () => {
    expect(CONFIG_ENVIRONMENTS.map(environmentLabel)).toEqual([
      "Local",
      "Dev",
      "Staging",
      "Production",
    ]);
  });

  it("kindLabel", () => {
    expect(CONFIG_KINDS.map(kindLabel)).toEqual([
      "YAML",
      "Properties",
      ".env",
      "Docker Compose",
      "Caddy/Nginx",
      "JSON",
    ]);
  });

  it("constants match the Rust lists", () => {
    expect([...CONFIG_ENVIRONMENTS]).toEqual(["local", "dev", "staging", "production"]);
    expect([...CONFIG_KINDS]).toEqual([
      "yaml",
      "properties",
      "dotenv",
      "docker_compose",
      "caddy_nginx",
      "json",
    ]);
  });
});
