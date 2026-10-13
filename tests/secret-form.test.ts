import { describe, expect, it } from "vitest";
import {
  maskedValue,
  revealTimerExpired,
  secretProjectLabel,
  validateSecretForm,
  type SecretFormInput,
} from "../src/lib/secret-form";

const valid: SecretFormInput = {
  label: "Stripe key",
  secret_type: "api_key",
  environment: "dev",
  value: "sk_test_123",
};

describe("validateSecretForm", () => {
  it("accepts a complete form", () => {
    expect(validateSecretForm(valid, false)).toBeNull();
  });

  it("requires a label, ignoring surrounding spaces", () => {
    expect(validateSecretForm({ ...valid, label: "" }, false)).toBe("secrets_label_required");
    expect(validateSecretForm({ ...valid, label: "   " }, false)).toBe("secrets_label_required");
  });

  it("limits the trimmed label to 120 characters", () => {
    expect(validateSecretForm({ ...valid, label: "a".repeat(120) }, false)).toBeNull();
    expect(validateSecretForm({ ...valid, label: ` ${"a".repeat(120)} ` }, false)).toBeNull();
    expect(validateSecretForm({ ...valid, label: "a".repeat(121) }, false)).toBe(
      "secrets_label_too_long",
    );
  });

  it("requires a type", () => {
    expect(validateSecretForm({ ...valid, secret_type: "" }, false)).toBe("secrets_type_required");
  });

  it("requires an environment", () => {
    expect(validateSecretForm({ ...valid, environment: "" }, false)).toBe(
      "secrets_environment_required",
    );
  });

  it("requires a value when creating", () => {
    expect(validateSecretForm({ ...valid, value: "" }, false)).toBe("secrets_value_required");
  });

  it("allows a blank value when editing (keeps the stored value)", () => {
    expect(validateSecretForm({ ...valid, value: "" }, true)).toBeNull();
  });

  it("limits the value to 4096 bytes", () => {
    expect(validateSecretForm({ ...valid, value: "a".repeat(4096) }, false)).toBeNull();
    expect(validateSecretForm({ ...valid, value: "a".repeat(4097) }, false)).toBe(
      "secrets_value_too_long",
    );
    // 2049 two-byte characters = 4098 bytes while only 2049 characters long.
    expect(validateSecretForm({ ...valid, value: "é".repeat(2049) }, false)).toBe(
      "secrets_value_too_long",
    );
    expect(validateSecretForm({ ...valid, value: "é".repeat(2048) }, false)).toBeNull();
  });

  it("limits an edited value to 4096 bytes too", () => {
    expect(validateSecretForm({ ...valid, value: "a".repeat(4097) }, true)).toBe(
      "secrets_value_too_long",
    );
  });

  it("reports the first error in field order", () => {
    const empty: SecretFormInput = { label: "", secret_type: "", environment: "", value: "" };
    expect(validateSecretForm(empty, false)).toBe("secrets_label_required");
    expect(validateSecretForm({ ...empty, label: "x" }, false)).toBe("secrets_type_required");
    expect(validateSecretForm({ ...empty, label: "x", secret_type: "token" }, false)).toBe(
      "secrets_environment_required",
    );
    expect(
      validateSecretForm(
        { ...empty, label: "x", secret_type: "token", environment: "local" },
        false,
      ),
    ).toBe("secrets_value_required");
  });
});

describe("maskedValue", () => {
  it("returns the fixed mask", () => {
    expect(maskedValue()).toBe("••••••••");
    expect(maskedValue()).toBe(maskedValue());
  });
});

describe("revealTimerExpired", () => {
  it("is false before 30 seconds", () => {
    expect(revealTimerExpired(1000, 1000)).toBe(false);
    expect(revealTimerExpired(0, 29999)).toBe(false);
  });

  it("is true at and after 30 seconds", () => {
    expect(revealTimerExpired(0, 30000)).toBe(true);
    expect(revealTimerExpired(0, 30001)).toBe(true);
    expect(revealTimerExpired(5000, 35000)).toBe(true);
  });
});

describe("secretProjectLabel", () => {
  const names = new Map([[1, "Alpha"]]);
  it("resolves names, no project, and removed projects", () => {
    expect(secretProjectLabel({ project_id: 1, project_missing: false }, names)).toBe("Alpha");
    expect(secretProjectLabel({ project_id: null, project_missing: false }, names)).toBe(
      "No project",
    );
    expect(secretProjectLabel({ project_id: 9, project_missing: true }, names)).toBe(
      "Project removed",
    );
    expect(secretProjectLabel({ project_id: 9, project_missing: false }, names)).toBe(
      "Project removed",
    );
  });
});
