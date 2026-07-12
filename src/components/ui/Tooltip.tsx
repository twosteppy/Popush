// Radix Tooltip wrapper (§20: use Radix primitives). Accessible, keyboard and
// screen-reader friendly out of the box.

import * as RadixTooltip from '@radix-ui/react-tooltip';
import type { ReactNode } from 'react';

interface TooltipProps {
  content: ReactNode;
  children: ReactNode;
  /** Render even when content is empty (no-op passthrough). */
  side?: 'top' | 'right' | 'bottom' | 'left';
}

export function Tooltip({ content, children, side = 'top' }: TooltipProps) {
  if (!content) return <>{children}</>;
  return (
    <RadixTooltip.Provider delayDuration={200}>
      <RadixTooltip.Root>
        <RadixTooltip.Trigger asChild>{children}</RadixTooltip.Trigger>
        <RadixTooltip.Portal>
          <RadixTooltip.Content
            side={side}
            sideOffset={6}
            className="z-50 max-w-xs rounded-md border border-border-strong bg-surface-overlay px-2 py-1 text-xs text-text-secondary shadow-lg"
          >
            {content}
            <RadixTooltip.Arrow className="fill-surface-overlay" />
          </RadixTooltip.Content>
        </RadixTooltip.Portal>
      </RadixTooltip.Root>
    </RadixTooltip.Provider>
  );
}
