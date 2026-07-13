import {
  forwardRef,
  useId,
  type InputHTMLAttributes,
  type ReactNode,
  type SelectHTMLAttributes,
} from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { cn } from '../../lib/cn';

const CONTROL =
  'h-[34px] w-full rounded-sm border bg-surface-base px-2.5 text-sm text-text-primary placeholder:text-text-tertiary transition-colors focus:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent';

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
          className="label-mono text-[11px] font-medium text-text-secondary"
        >
          {label}
        </label>
        {optional ? (
          <span className="label-mono text-[10px] text-text-tertiary">
            Optional
          </span>
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

interface NumberFieldProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  'value' | 'onChange' | 'type'
> {
  invalid?: boolean;
  /** Current value as a string (mirrors the raw input value). */
  value: string | number;
  /** Emits the next value as a string on both typing and stepper presses. */
  onValueChange: (value: string) => void;
  min?: number;
  max?: number;
  step?: number;
}

/**
 * A number input with custom, token-themed stepper buttons. The native spin
 * buttons are hidden globally (see globals.css); these ChevronUp/ChevronDown
 * <button>s increment/decrement while respecting min/max/step. Keyboard arrows
 * on the input itself still work as usual.
 */
export const NumberField = forwardRef<HTMLInputElement, NumberFieldProps>(
  function NumberField(
    { invalid, value, onValueChange, min, max, step = 1, className, ...rest },
    ref,
  ) {
    const disabled = rest.disabled;

    function clamp(n: number): number {
      if (min !== undefined && n < min) return min;
      if (max !== undefined && n > max) return max;
      return n;
    }

    function nudge(direction: 1 | -1): void {
      const current = Number(value);
      const base = Number.isFinite(current) ? current : (min ?? 0);
      onValueChange(String(clamp(base + direction * step)));
    }

    const numeric = Number(value);
    const atMax =
      max !== undefined && Number.isFinite(numeric) && numeric >= max;
    const atMin =
      min !== undefined && Number.isFinite(numeric) && numeric <= min;

    return (
      <div className={cn('relative', className)}>
        <input
          ref={ref}
          type="number"
          inputMode="numeric"
          value={value}
          min={min}
          max={max}
          step={step}
          aria-invalid={invalid || undefined}
          onChange={(e) => onValueChange(e.target.value)}
          className={cn(
            CONTROL,
            'pr-9',
            invalid ? 'border-status-failed' : 'border-border-strong',
          )}
          {...rest}
        />
        <div className="absolute inset-y-0 right-0 flex w-8 flex-col overflow-hidden rounded-r-sm border-l border-border-subtle">
          <StepperButton
            label="Increase"
            onClick={() => nudge(1)}
            disabled={disabled || atMax}
          >
            <ChevronUp size={12} aria-hidden="true" />
          </StepperButton>
          <span className="h-px bg-border-subtle" aria-hidden="true" />
          <StepperButton
            label="Decrease"
            onClick={() => nudge(-1)}
            disabled={disabled || atMin}
          >
            <ChevronDown size={12} aria-hidden="true" />
          </StepperButton>
        </div>
      </div>
    );
  },
);

function StepperButton({
  label,
  onClick,
  disabled,
  children,
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className="flex flex-1 items-center justify-center bg-surface-raised text-text-secondary transition-colors hover:bg-surface-hover hover:text-accent focus-visible:z-10 focus-visible:text-accent focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-surface-raised disabled:hover:text-text-secondary"
    >
      {children}
    </button>
  );
}

/** Generate a stable field id from a base name. */
export function useFieldId(base: string): string {
  const id = useId();
  return `${base}-${id}`;
}
