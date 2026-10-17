import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { EditorOpenTest } from "../components/editor-open-test";
import { EditorProfilesSection } from "../components/editor-profiles-section";
import { storageStatus } from "../lib/commands";
import { getTheme, setTheme, type Theme } from "../lib/theme";

const THEME_OPTIONS: { value: Theme; label: string }[] = [
  { value: "dark", label: "Dark" },
  { value: "light", label: "Light" },
];

function AppearanceSection() {
  const [theme, setThemeState] = useState<Theme>(getTheme);

  function choose(next: Theme) {
    setThemeState(next);
    setTheme(next);
  }

  return (
    <section className="mb-6 rounded-2xl border border-hairline bg-surface p-5">
      <fieldset className="m-0 border-0 p-0">
        <legend className="mb-3 p-0 text-xl font-semibold">Appearance</legend>
        <div className="flex gap-6">
          {THEME_OPTIONS.map((option) => (
            <label key={option.value} className="flex cursor-pointer items-center gap-2">
              <input
                type="radio"
                name="theme"
                value={option.value}
                checked={theme === option.value}
                onChange={() => choose(option.value)}
                className="accent-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
              />
              {option.label}
            </label>
          ))}
        </div>
      </fieldset>
    </section>
  );
}

export function SettingsScreen() {
  const status = useQuery({ queryKey: ["storage-status"], queryFn: storageStatus });
  const data = status.data;

  return (
    <div>
      <h1 className="m-0 mb-8 text-3xl font-semibold">Settings</h1>
      <AppearanceSection />
      <EditorProfilesSection />
      <EditorOpenTest />
      {data?.ok ? (
        <section className="mb-6 rounded-2xl border border-hairline bg-surface p-5">
          <p className="m-0 mb-1 font-medium">Storage: OK</p>
          <p className="m-0 break-all font-mono text-sm text-ink-muted">{data.folder}</p>
        </section>
      ) : null}
      <p className="m-0 max-w-[60ch] text-ink-muted">
        DevShelf stores all data on this computer. Nothing is sent over the network. Project names,
        labels, environment names, and file paths are stored unencrypted in the local database.
      </p>
    </div>
  );
}
