import type { GeneratorErrorCode } from "../types/commands";

export const DEFAULT_PASSWORD_LENGTH = 20;
export const MIN_GENERATED_LENGTH = 8;
export const MAX_GENERATED_LENGTH = 128;

/** Parses the length field. Non-numeric input falls back to the default; numbers are floored and clamped. */
export function clampLength(raw: string): number {
  const parsed = Number.parseFloat(raw);
  if (!Number.isFinite(parsed)) {
    return DEFAULT_PASSWORD_LENGTH;
  }
  return Math.min(MAX_GENERATED_LENGTH, Math.max(MIN_GENERATED_LENGTH, Math.floor(parsed)));
}

export function formatGeneratorError(code: GeneratorErrorCode): string {
  switch (code) {
    case "generator_length_invalid":
      return "Choose a length from 8 to 128.";
    case "generator_unavailable":
      return "Could not generate a password. Try again.";
    case "generator_copy_failed":
      return "Could not copy to the clipboard.";
  }
}
