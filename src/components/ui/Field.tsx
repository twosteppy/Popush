// Form primitives (§20): labels sit ABOVE inputs (never placeholder-as-label),
// controls are 34px tall with visible focus rings, and validation errors get a
// text message tied to the input via aria-describedby.
//
// D14: presentation only. These render values and emit change intents upward.

import {
  forwardRef,
  useId,
  type InputHTMLAttributes,
  type ReactNode,
  type SelectHTMLAttributes,
} from 'react';
import { cn } from '../../lib/cn';

const CONTROL =
  'h-[34px] w-full rounded-md border bg-surface-base px-2.5 text-sm text-text-primary placeholder:text-text-tertiary transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent';

interface FieldProps {
  label: string;
  htmlFor: string;
  /** Optional helper text shown beneath the label. */
  hint?: ReactNode;
  /** Validation error; when present the control is styled and described by it. */
  error?: string | null;
  /** Marks the field visually optional. */
  optional?: boolean;
  children: ReactNode;
}

/** A labelled form row: label (above) + control + optional hint/error. */
export function Field({
  label,
  htmlFor,
  hint,
  error,
  optional,
  children,
}: FieldProps) {
  const hintId = `${htmlFor}-hint`;
  const errId = `${htmlFor}-error`;
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-baseline justify-between gap-2">
        <label
          htmlFor={htmlFor}
          className="text-xs font-medium text-text-secondary"
        >
          {label}
        </label>
        {optional ? (
          <span className="text-[11px] text-text-tertiary">Optional</span>
        ) : null}
      </div>
      {children}
      {error ? (
        <p id={errId} className="text-xs text-status-failed">
          {error}
        </p>
      ) : hint ? (
        <p id={hintId} className="text-xs text-text-tertiary">
          {hint}
        </p>
      ) : null}
    </div>
  );
}

interface TextInputProps extends InputHTMLAttributes<HTMLInputElement> {
  invalid?: boolean;
}

export const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  function TextInput({ invalid, className, ...rest }, ref) {
    return (
      <input
        ref={ref}
        aria-invalid={invalid || undefined}
        className={cn(
          CONTROL,
          invalid ? 'border-status-failed' : 'border-border-strong',
          className,
        )}
        {...rest}
      />
    );
  },
);

interface SelectInputProps extends SelectHTMLAttributes<HTMLSelectElement> {
  invalid?: boolean;
}

export const SelectInput = forwardRef<HTMLSelectElement, SelectInputProps>(
  function SelectInput({ invalid, className, children, ...rest }, ref) {
    return (
      <select
        ref={ref}
        aria-invalid={invalid || undefined}
        className={cn(
          CONTROL,
          'appearance-none pr-8',
          invalid ? 'border-status-failed' : 'border-border-strong',
          className,
        )}
        {...rest}
      >
        {children}
      </select>
    );
  },
);

/** Generate a stable field id from a base name. */
export function useFieldId(base: string): string {
  const id = useId();
  return `${base}-${id}`;
}
