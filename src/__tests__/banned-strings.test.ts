// The strings "Deploy failed" and "Something went wrong" are banned anywhere in
// user-facing copy. This test recursively reads every .ts/.tsx file under src/
// and fails if either appears. This test file itself is excluded (it
// necessarily names the banned strings).

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const SRC_DIR = resolve(__dirname, '..');
const SELF = resolve(__filename);
const BANNED = ['Deploy failed', 'Something went wrong'];

// Backend-owned generated types are not frontend user-facing strings; the file
// legitimately references the banned phrase in a doc comment.
const EXCLUDED = new Set([resolve(SRC_DIR, 'types', 'generated.ts')]);

function collectFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      out.push(...collectFiles(full));
    } else if (/\.(ts|tsx)$/.test(entry)) {
      out.push(full);
    }
  }
  return out;
}

describe('banned user-facing strings', () => {
  const files = collectFiles(SRC_DIR).filter(
    (f) => resolve(f) !== SELF && !EXCLUDED.has(resolve(f)),
  );

  it('scans a non-trivial number of source files', () => {
    expect(files.length).toBeGreaterThan(5);
  });

  for (const phrase of BANNED) {
    it(`no source file contains "${phrase}"`, () => {
      const offenders = files.filter((f) =>
        readFileSync(f, 'utf8').includes(phrase),
      );
      expect(offenders).toEqual([]);
    });
  }
});
