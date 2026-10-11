import { EmptyState } from "../components/empty-state";

export function VaultScreen() {
  return (
    <div>
      <h1 className="m-0 mb-8 text-3xl font-semibold">Vault</h1>
      <EmptyState heading="No secrets yet" />
    </div>
  );
}
