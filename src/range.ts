import { Position, Range } from 'vscode-languageserver';

export function contains(range: Range, position: Position) {
  function leq(left: Position, right: Position) {
    return left.line === right.line
      ? left.character <= right.character
      : left.line <= right.line;
  }
  return leq(range.start, position) && leq(position, range.end);
}
