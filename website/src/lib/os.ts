export type OS = 'windows' | 'mac' | 'linux' | 'other';

/** Best-effort client OS sniff, used only to pick which download to feature. */
export function detectOS(): OS {
  if (typeof navigator === 'undefined') return 'other';
  const s = `${navigator.userAgent} ${navigator.platform ?? ''}`.toLowerCase();
  if (s.includes('win')) return 'windows';
  if (s.includes('mac') || s.includes('iphone') || s.includes('ipad')) return 'mac';
  if (s.includes('linux') || s.includes('android') || s.includes('x11')) return 'linux';
  return 'other';
}
