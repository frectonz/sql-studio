"use client";

import { useState } from "react";

export function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  return (
    <button
      onClick={() => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      }}
      className="tw:text-xs tw:tracking-widest tw:transition-colors tw:duration-100"
      style={{ color: copied ? "var(--ss-primary)" : "var(--ss-muted-dim)" }}
    >
      {copied ? "COPIED" : "COPY"}
    </button>
  );
}
