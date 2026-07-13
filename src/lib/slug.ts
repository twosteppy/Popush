export function slugify(input: string): string {
  return input
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .replace(/-{2,}/g, '-');
}

/** A slug guaranteed non-empty, falling back to a timestamped prefix. */
export function slugId(input: string, prefix: string): string {
  const base = slugify(input);
  return base || `${prefix}-${Date.now().toString(36)}`;
}
