import { describe, expect, it } from 'vitest';
import { sanitizeTerminalOutput } from '../ansi';

describe('sanitizeTerminalOutput', () => {
  it('keeps plain text, tabs, and newlines', () => {
    expect(sanitizeTerminalOutput('build ok\tdone\n')).toBe('build ok\tdone\n');
  });

  it('preserves SGR colour sequences', () => {
    const colored = '\x1b[32mgreen\x1b[0m';
    expect(sanitizeTerminalOutput(colored)).toBe(colored);
  });

  it('strips cursor-motion sequences used to overwrite lines', () => {
    // Move-up + carriage-return + erase-line: the classic log-spoofing tools.
    const spoof = 'real command\x1b[1A\x1b[2Kfake success';
    const out = sanitizeTerminalOutput(spoof);
    expect(out).toBe('real commandfake success');
    expect(out).not.toContain('\x1b');
  });

  it('strips OSC strings (e.g. window-title / hyperlink)', () => {
    const osc = '\x1b]0;pwned\x07visible';
    expect(sanitizeTerminalOutput(osc)).toBe('visible');
  });

  it('drops other control and DEL bytes but keeps CR/LF', () => {
    expect(sanitizeTerminalOutput('a\x00b\x07c\x7fd\r\n')).toBe('abcd\r\n');
  });
});
