// Remote command / git output is attacker-influenceable: a compromised server or
// repo can emit arbitrary bytes. xterm.js does not execute code, but raw control
// sequences (cursor motion, carriage returns, OSC/DCS strings) let hostile output
// overwrite earlier lines to forge a "success" line or hide a real command in a
// deployment tool's log. This scanner keeps colour (SGR) sequences and printable
// text and drops everything else that could rewrite the display.
export function sanitizeTerminalOutput(input: string): string {
  let out = '';
  let i = 0;
  const n = input.length;
  while (i < n) {
    const code = input.charCodeAt(i);
    if (code === 0x1b) {
      const next = input[i + 1];
      if (next === '[') {
        // CSI: ESC [ params/intermediates (0x20-0x3f) then a final byte. Keep
        // only SGR (final 'm', i.e. colour/style); drop cursor motion, erase, etc.
        let j = i + 2;
        while (
          j < n &&
          input.charCodeAt(j) >= 0x20 &&
          input.charCodeAt(j) <= 0x3f
        )
          j++;
        if (j < n) {
          if (input[j] === 'm') out += input.slice(i, j + 1);
          i = j + 1;
        } else {
          i = n; // incomplete sequence at end of chunk; drop it
        }
        continue;
      }
      if (
        next === ']' ||
        next === 'P' ||
        next === 'X' ||
        next === '^' ||
        next === '_'
      ) {
        // OSC/DCS/APC/PM/SOS strings: consume up to BEL or ST (ESC \).
        let j = i + 2;
        while (j < n) {
          if (input.charCodeAt(j) === 0x07) {
            j++;
            break;
          }
          if (input.charCodeAt(j) === 0x1b && input[j + 1] === '\\') {
            j += 2;
            break;
          }
          j++;
        }
        i = j;
        continue;
      }
      // Any other ESC-initiated sequence: drop the ESC and its following byte.
      i += 2;
      continue;
    }
    // Keep tab, newline, carriage return, and printable characters; drop other
    // C0/C1 control bytes and DEL.
    const keep =
      code === 0x09 ||
      code === 0x0a ||
      code === 0x0d ||
      (code >= 0x20 && code !== 0x7f && !(code >= 0x80 && code <= 0x9f));
    if (keep) out += input[i];
    i++;
  }
  return out;
}
