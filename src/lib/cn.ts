// Tiny class-name composer. Wraps clsx so components can compose token classes
// conditionally without string-template noise. Presentation only (D14).

import clsx, { type ClassValue } from 'clsx';

export function cn(...inputs: ClassValue[]): string {
  return clsx(inputs);
}
