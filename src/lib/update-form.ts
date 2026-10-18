import type {
  UpdateCheckResult,
  UpdateErrorCode,
  UpdatePrefsResult,
  UpdateView,
} from "../types/commands";

export function formatUpdateError(code: UpdateErrorCode): string {
  switch (code) {
    case "check_failed":
      return "Couldn't check for updates. Check your internet connection and try again.";
    case "install_failed":
      return "The update couldn't be installed. Try again, or download the installer from the GitHub Releases page.";
    case "nothing_pending":
      return "No update is ready. Check for updates again.";
    case "storage_unavailable":
      return "Settings couldn't be read. Restart DevShelf.";
  }
}

/**
 * Launch-time check. Failures are silent by design: offline use must not show an error.
 * Returns the available update, or null when disabled, failed, or up to date.
 */
export async function startupUpdateCheck(deps: {
  getPrefs: () => Promise<UpdatePrefsResult>;
  check: () => Promise<UpdateCheckResult>;
}): Promise<UpdateView | null> {
  const prefs = await deps.getPrefs();
  if (!prefs.auto_check) {
    return null;
  }
  try {
    const result = await deps.check();
    if (result.error) {
      return null;
    }
    return result.update;
  } catch {
    return null;
  }
}
