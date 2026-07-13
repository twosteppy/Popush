// URL safety helpers. User-configured URLs (a site's live URL, its health-check
// URL) must never be a `javascript:`, `data:`, or `file:` URL: those would run
// in, or exfiltrate from, the app webview. Only http(s) is allowed, and such
// links are opened in the system browser, never navigated to in-app.

/** True when `value` parses as an absolute http(s) URL. */
export function isSafeHttpUrl(value: string | null | undefined): boolean {
  if (!value) return false;
  try {
    const u = new URL(value);
    return u.protocol === 'http:' || u.protocol === 'https:';
  } catch {
    return false;
  }
}
