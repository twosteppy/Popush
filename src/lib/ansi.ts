export function sanitizeTerminalOutput(input: string): string {
  let out = '';
  let i = 0;
  const n = input.length;
  while (i < n) {
    const code = input.charCodeAt(i);
    if (code === 0x1b) {
      const next = input[i + 1];
      if (next === '[') {
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
      i += 2;
      continue;
    }
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
