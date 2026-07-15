import { useState } from 'react';

export function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  function copy() {
    if (!navigator.clipboard) return;
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    });
  }

  return (
    <button
      type="button"
      className={`copy-btn${copied ? ' copied' : ''}`}
      onClick={copy}
      aria-label="Copy the install command"
    >
      {copied ? 'Copied' : 'Copy'}
    </button>
  );
}

/** The install-command chip with a copy affordance. */
export function InstallCommand({
  display,
  command,
  className,
}: {
  display: string;
  command: string;
  className?: string;
}) {
  return (
    <div className={`cmd${className ? ` ${className}` : ''}`}>
      <code>
        curl -fsSL <span className="tok">{display}</span> | bash
      </code>
      <CopyButton text={command} />
    </div>
  );
}
