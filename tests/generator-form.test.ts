import { describe, expect, it } from "vitest";
import { clampLength, formatGeneratorError } from "../src/lib/generator-form";

describe("clampLength", () => {
  it("falls back to the default for non-numeric input", () => {
    expect(clampLength("")).toBe(20);
    expect(clampLength("abc")).toBe(20);
  });

  it("clamps to 8..128", () => {
    expect(clampLength("3")).toBe(8);
    expect(clampLength("500")).toBe(128);
  });

  it("floors fractional input and keeps valid values", () => {
    expect(clampLength("24.9")).toBe(24);
    expect(clampLength("20")).toBe(20);
  });
});

describe("formatGeneratorError", () => {
  it("names the length range for an invalid length", () => {
    expect(formatGeneratorError("generator_length_invalid")).toBe("Choose a length from 8 to 128.");
  });

  it("gives a non-empty message for every code", () => {
    for (const code of [
      "generator_length_invalid",
      "generator_unavailable",
      "generator_copy_failed",
    ] as const) {
      expect(formatGeneratorError(code)).not.toBe("");
    }
  });
});
