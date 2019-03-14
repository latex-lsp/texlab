import { Position, Range } from 'vscode-languageserver';

export function contains(range: Range, position: Position): boolean {
  function leq(left: Position, right: Position) {
    return left.line === right.line
      ? left.character <= right.character
      : left.line <= right.line;
  }
  return leq(range.start, position) && leq(position, range.end);
}

export function containsExclusive(range: Range, position: Position): boolean {
  function lt(left: Position, right: Position) {
    return left.line === right.line
      ? left.character < right.character
      : left.line < right.line;
  }
  return lt(range.start, position) && lt(position, range.end);
}
