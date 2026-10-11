import { Outlet } from "react-router";
import { Sidebar } from "./sidebar";

export function AppShell() {
  return (
    <div className="grid h-full grid-cols-[248px_1fr] bg-canvas text-ink">
      <Sidebar />
      <main className="min-w-0 overflow-y-auto px-10 py-8">
        <Outlet />
      </main>
    </div>
  );
}
