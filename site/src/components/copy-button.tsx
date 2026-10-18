import { useEffect, useState } from "react";
import { Check, Copy } from "@phosphor-icons/react";

type CopyButtonProps = {
  text: string;
};

export function CopyButton({ text }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const id = window.setTimeout(() => setCopied(false), 2000);
    return () => window.clearTimeout(id);
  }, [copied]);

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch {
      // Clipboard unavailable or denied: keep the label as "Copy".
    }
  }

  return (
    <button
      type="button"
      onClick={copy}
      className="inline-flex items-center gap-2 whitespace-nowrap rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-xs font-semibold text-ink transition hover:-translate-y-px active:scale-[0.98]"
    >
      {copied ? (
        <Check size={16} weight="regular" aria-hidden="true" />
      ) : (
        <Copy size={16} weight="regular" aria-hidden="true" />
      )}
      {copied ? "Copied" : "Copy"}
    </button>
  );
}
