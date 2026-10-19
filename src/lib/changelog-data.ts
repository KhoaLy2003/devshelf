// Generated from release-notes/v*.md by scripts/gen-changelog.mjs. Do not edit by hand.

export interface ReleaseNotes {
  version: string;
  items: readonly string[];
}

export const CHANGELOG_RELEASES: readonly ReleaseNotes[] = [
  {
    version: "0.3.0",
    items: [
      "Project folder autofill: Register project fills name, type and tags from the folder's marker files, and never writes to the folder",
      "Password generator for the vault master password and secret values",
      "Export data: plaintext CSV export of all tables, with the master password and an acknowledgement required",
      "Landing page and GitHub Pages deploy",
    ],
  },
  {
    version: "0.2.0",
    items: [
      "Signed update check: DevShelf checks GitHub Releases at startup (turn this off in Settings) or on demand, and installs only after you confirm",
      "Home overview (F10)",
      "Project unregister: removes the registration only; files are not touched (F15)",
      "Help (F16)",
      "Vault delete: requires the master password and removes all secrets",
    ],
  },
  {
    version: "0.1.0",
    items: [
      "Foundation and navigation (F01)",
      "Project registry (F02)",
      "Editor profiles (F03)",
      "Workspace builder (F04)",
      "Workspace restore with missing-path report (F05)",
      "Config catalog (F06)",
      "Private project notes (F07)",
      "Vault access control (F08)",
      "Secret entries (F09)",
    ],
  },
];
