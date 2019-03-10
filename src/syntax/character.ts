export function isWhiteSpace(c: string): boolean {
  return (
    c === ' ' ||
    c === '\f' ||
    c === '\n' ||
    c === '\r' ||
    c === '\t' ||
    c === '\v'
  );
}

export function isCommandChar(c: string): boolean {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c === '@';
}
