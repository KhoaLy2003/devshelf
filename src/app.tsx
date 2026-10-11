import { useEffect, useState } from "react";
import { appInfo } from "./lib/commands";
import type { AppInfo } from "./types/commands";

export function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    appInfo().then(setInfo, () => setFailed(true));
  }, []);

  return (
    <main>
      <h1>Hello, DevShelf</h1>
      <p data-testid="app-info">
        {failed ? "app_info failed" : info ? `${info.name} v${info.version}` : "loading..."}
      </p>
    </main>
  );
}
