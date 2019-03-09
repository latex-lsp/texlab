import { Position, Range } from 'vscode-languageserver';

export abstract class SyntaxNode {
  public abstract readonly range: Range;

  public get start(): Position {
    return this.range.start;
  }

  public get end(): Position {
    return this.range.end;
  }
}
