import { useState } from "react";
import { copyPassword, generatePassword } from "../lib/commands";
import {
  DEFAULT_PASSWORD_LENGTH,
  MAX_GENERATED_LENGTH,
  MIN_GENERATED_LENGTH,
  clampLength,
  formatGeneratorError,
} from "../lib/generator-form";
import type { GeneratorErrorCode } from "../types/commands";

const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const inputClass =
  "w-24 rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";

interface PasswordGeneratorProps {
  value: string;
  onGenerated: (value: string) => void;
  disabled?: boolean;
}

/** Generates a password into the parent's field. It keeps no password of its own. */
export function PasswordGenerator({
  value,
  onGenerated,
  disabled = false,
}: PasswordGeneratorProps) {
  const [length, setLength] = useState(String(DEFAULT_PASSWORD_LENGTH));
  const [symbols, setSymbols] = useState(true);
  const [error, setError] = useState<GeneratorErrorCode | null>(null);
  const [copied, setCopied] = useState(false);

  const generate = async () => {
    const clamped = clampLength(length);
    setLength(String(clamped));
    setCopied(false);
    try {
      const result = await generatePassword(clamped, symbols);
      if (result.error !== null || result.value === null) {
        setError(result.error ?? "generator_unavailable");
        return;
      }
      setError(null);
      onGenerated(result.value);
    } catch {
      setError("generator_unavailable");
    }
  };

  const copy = async () => {
    try {
      const result = await copyPassword(value);
      if (result.error !== null) {
        setCopied(false);
        setError(result.error);
        return;
      }
      setError(null);
      setCopied(true);
    } catch {
      setCopied(false);
      setError("generator_copy_failed");
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-end gap-3">
        <label className="flex flex-col gap-1">
          <span>Length</span>
          <input
            type="number"
            min={MIN_GENERATED_LENGTH}
            max={MAX_GENERATED_LENGTH}
            className={inputClass}
            value={length}
            disabled={disabled}
            onChange={(e) => setLength(e.target.value)}
          />
        </label>
        <label className="flex items-center gap-2 pb-2">
          <input
            type="checkbox"
            checked={symbols}
            disabled={disabled}
            onChange={(e) => setSymbols(e.target.checked)}
          />
          <span>Symbols</span>
        </label>
        <button type="button" className={buttonSecondary} disabled={disabled} onClick={generate}>
          Generate password
        </button>
        <button type="button" className={buttonSecondary} disabled={value === ""} onClick={copy}>
          Copy
        </button>
      </div>
      {error !== null ? (
        <p role="alert" className="m-0 text-sm text-warning">
          {formatGeneratorError(error)}
        </p>
      ) : null}
      {copied ? <p className="m-0 text-sm text-ink-muted">Copied to clipboard.</p> : null}
    </div>
  );
}
