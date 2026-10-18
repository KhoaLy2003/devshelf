export const APP_VERSION = "0.3.0";

export interface HelpGroup {
  heading: string;
  items: readonly string[];
}

export interface HelpSection {
  id: string;
  title: string;
  body: readonly string[];
  groups?: readonly HelpGroup[];
}

export const HELP_SECTIONS: readonly HelpSection[] = [
  {
    id: "start",
    title: "Start here",
    body: [
      "DevShelf is a single-user, local-first Windows app for organizing your projects, editors, workspaces, config references, private notes, and an encrypted secrets vault.",
      "There is no cloud, no sync, and no telemetry. Everything stays on this computer.",
      "Appearance: choose Dark or Light in Settings. Dark is the default.",
    ],
  },
  {
    id: "projects",
    title: "Projects",
    body: [
      "Register a project by choosing its folder. Pin the projects you use most so they show on Home.",
      "A project is the link between workspaces, config entries, notes, and secrets.",
    ],
  },
  {
    id: "editors",
    title: "Editor profiles",
    body: [
      "In Settings, add an editor by choosing its executable and the arguments to pass to it.",
      "You can then open a project or a file in the editor you choose.",
      "Shell syntax in arguments is rejected.",
    ],
  },
  {
    id: "workspaces",
    title: "Workspaces",
    body: [
      "Build a workspace from projects and paths, then restore it in one action.",
      "The restore result lists what was opened and what is missing. For each missing item you can choose Relink or Ignore.",
      "Tab layout and unsaved editor buffers are not restored.",
    ],
  },
  {
    id: "config",
    title: "Config catalog",
    body: [
      "Register config files for each project. Each entry has an environment (local, dev, staging, or production) and a kind.",
      "The status is Available, Missing, or Inaccessible. You can open an entry or relink it to a new path.",
      "Entries are added by hand. Automatic discovery is not in this version.",
    ],
  },
  {
    id: "notes",
    title: "Notes",
    body: [
      "Notes are private Markdown notes. You can link a note to a project.",
      "Notes autosave one second after you stop typing. The status shows Saved, Unsaved changes, or Couldn't save with a Retry button.",
      "Notes are stored outside your project folders. Note bodies are encrypted, and the encryption key lives in Windows Credential Manager.",
      "Preview is not in this version.",
    ],
  },
  {
    id: "vault",
    title: "Vault",
    body: [
      "On first use, create a master password of at least 12 characters.",
      "There is no recovery. If you forget the password, the vault cannot be opened.",
      "Unlock the vault with your password. Choose Lock now to lock it at any time.",
      "The vault also auto-locks after 1, 5, or 15 minutes with no mouse or keyboard input in the window, or never. The default is 5 minutes. A countdown appears in the last 60 seconds.",
      "The lock screen shows why the vault locked.",
    ],
  },
  {
    id: "secrets",
    title: "Secrets",
    body: [
      "Secrets are available only while the vault is unlocked.",
      "Each secret has a label, a type (API key, Password, Token, Connection string, or Other), an optional project, and an environment.",
      "Values are masked. Reveal shows one value at a time and masks it again after 30 seconds without interaction, or when the vault locks.",
      "Copy puts the value on the clipboard without showing it.",
      "A value can be up to 4096 bytes.",
    ],
  },
  {
    id: "home",
    title: "Home",
    body: [
      "Home shows your pinned projects and recent workspaces.",
      "Workspaces show their item counts and a warning when items are missing.",
      "Quick actions let you start common tasks from one place.",
    ],
  },
  {
    id: "privacy",
    title: "Privacy and security",
    body: [
      "DevShelf's only network use is checking GitHub Releases for updates, at startup (you can turn this off in Settings) or when you click Check for updates. No data about you or your projects is sent.",
      "All data stays in your Windows user data folder: %APPDATA%\\app.devshelf.desktop.",
      "Secrets and note bodies are encrypted.",
      "Plaintext secrets are sent to the screen only when you explicitly reveal or copy them.",
      "Project names, labels, environment names, and file paths are stored unencrypted in the local database.",
      "Export data (Settings) writes your notes and secret values as unencrypted CSV files; protect or delete them.",
    ],
  },
  {
    id: "troubleshooting",
    title: "Troubleshooting",
    body: [
      "A missing path shows as Missing with a reason. Use Relink to point it at the new location.",
      "If the vault locks unexpectedly, the inactivity timeout was reached. Change it in the Vault settings.",
      "If the Credential Manager entry for notes is deleted, existing notes cannot be opened and show Note unavailable.",
    ],
  },
  {
    id: "changelog",
    title: "Changelog",
    body: [`Version ${APP_VERSION}. Features shipped in this build:`],
    groups: [
      {
        heading: `Version ${APP_VERSION}`,
        items: [
          "F01 Foundation and navigation",
          "F02 Project registry",
          "F03 Editor profiles",
          "F04 Workspace builder",
          "F05 Workspace restore with missing-path report",
          "F06 Config catalog",
          "F07 Private project notes",
          "F08 Vault access control",
          "F09 Secret entries",
          "F10 Home overview",
          "F15 Project unregister (removes registration only; files are not touched)",
          "F16 Help",
          "Vault delete (requires the master password; removes all secrets)",
          "Updates: DevShelf checks GitHub Releases at startup (turn this off in Settings) or on demand, and installs only after you confirm",
        ],
      },
      {
        heading: "Not in this version",
        items: [
          "F11 Quick open: not in this version.",
          "F12 Encrypted backup and restore: not in this version.",
          "F13 Bounded config discovery: not in this version.",
          "F14 Clipboard auto-clear: not in this version.",
          "F15 Project maintenance (editing name, type and tags): not in this version.",
        ],
      },
    ],
  },
  {
    id: "about",
    title: "About",
    body: [`DevShelf version ${APP_VERSION}.`, "License: none. Telemetry: none."],
  },
];

export function getSection(id: string | null | undefined): HelpSection {
  const first = HELP_SECTIONS[0];
  if (first === undefined) throw new Error("HELP_SECTIONS is empty");
  return HELP_SECTIONS.find((s) => s.id === id) ?? first;
}
