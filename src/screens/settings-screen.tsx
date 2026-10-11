import { useQuery } from "@tanstack/react-query";
import { EditorOpenTest } from "../components/editor-open-test";
import { EditorProfilesSection } from "../components/editor-profiles-section";
import { storageStatus } from "../lib/commands";

export function SettingsScreen() {
  const status = useQuery({ queryKey: ["storage-status"], queryFn: storageStatus });
  const data = status.data;

  return (
    <div>
      <h1 className="m-0 mb-8 text-3xl font-semibold">Settings</h1>
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
