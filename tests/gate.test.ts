import { describe, expect, it } from "vitest";
import { resolveGate } from "../src/lib/gate";

describe("resolveGate", () => {
  it("returns loading while pending", () => {
    expect(resolveGate({ isPending: true, isError: false })).toEqual({ kind: "loading" });
  });

  it("returns storage_unavailable with empty folder on query error", () => {
    const result = resolveGate({ isPending: false, isError: true });
    expect(result).toEqual({ kind: "error", code: "storage_unavailable", folder: "" });
    expect(Object.keys(result).sort()).toEqual(["code", "folder", "kind"]);
  });

  it("carries schema_too_new code and folder", () => {
    expect(
      resolveGate({
        isPending: false,
        isError: false,
        data: { ok: false, folder: "C:\\data", error_code: "schema_too_new" },
      }),
    ).toEqual({ kind: "error", code: "schema_too_new", folder: "C:\\data" });
  });

  it("falls back to storage_unavailable when the code is null", () => {
    expect(
      resolveGate({
        isPending: false,
        isError: false,
        data: { ok: false, folder: "C:\\data", error_code: null },
      }),
    ).toEqual({ kind: "error", code: "storage_unavailable", folder: "C:\\data" });
  });

  it("returns ready when ok", () => {
    expect(
      resolveGate({
        isPending: false,
        isError: false,
        data: { ok: true, folder: "C:\\data", error_code: null },
      }),
    ).toEqual({ kind: "ready" });
  });
});
