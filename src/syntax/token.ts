import { Position, Range } from 'vscode-languageserver';

export abstract class Token {
  constructor(public readonly start: Position, public readonly text: string) {}

  public get line(): number {
    return this.start.line;
  }

  public get character(): number {
    return this.start.character;
  }

  public get length(): number {
    return this.text.length;
  }

  public get end(): Position {
    return {
      line: this.line,
      character: this.character + this.length,
    };
  }

  public get range(): Range {
    return {
      start: this.start,
      end: this.end,
    };
  }
}

export interface TokenSource<T> {
  next(): T | undefined;
}

export class TokenBuffer<T> {
  private readonly buffer: T[];

  constructor(private source: TokenSource<T>) {
    this.buffer = [];
  }

  public get available(): boolean {
    return this.peek() !== undefined;
  }

  public peek(lookAhead: number = 0): T | undefined {
    while (this.buffer.length < lookAhead + 1) {
      const token = this.source.next();
      if (token === undefined) {
        return undefined;
      }
      this.buffer.push(token);
    }
    return this.buffer[lookAhead];
  }

  public next(): T {
    const token = this.peek();
    this.buffer.splice(0, 1);
    return token!;
  }
}
