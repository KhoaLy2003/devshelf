import { useState } from "react";
import { Moon, Sun } from "@phosphor-icons/react";
import { applyTheme, readTheme, type Theme } from "../lib/theme";

export function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>(readTheme);
  const next: Theme = theme === "dark" ? "light" : "dark";
  const Icon = theme === "dark" ? Sun : Moon;

  function toggle() {
    applyTheme(next);
    setTheme(next);
  }

  return (
    <button
      type="button"
      onClick={toggle}
      aria-label={`Switch to ${next} theme`}
      className="inline-flex size-10 items-center justify-center rounded-lg border border-hairline bg-surface text-ink transition hover:-translate-y-px active:scale-[0.98]"
    >
      <Icon size={20} weight="regular" aria-hidden="true" />
    </button>
  );
}
