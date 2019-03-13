import { Position } from 'vscode-languageserver';
import { isCommandChar, isWhiteSpace } from '../character';
import { CharStream } from '../charStream';
import { Token, TokenSource } from '../token';

export enum LatexTokenKind {
  Command,
  Word,
  Math,
  BeginGroup,
  EndGroup,
  BeginOptions,
  EndOptions,
}

export class LatexToken extends Token {
  constructor(
    start: Position,
    text: string,
    public readonly kind: LatexTokenKind,
  ) {
    super(start, text);
  }
}

export class LatexLexer implements TokenSource<LatexToken> {
  private readonly stream: CharStream;

  constructor(text: string) {
    this.stream = new CharStream(text);
  }

  public next(): LatexToken | undefined {
    while (this.stream.available) {
      const c = this.stream.peek();
      switch (c) {
        case '%':
          this.stream.next();
          this.stream.skipRestOfLine();
          break;
        case '{':
          return this.delimiter(LatexTokenKind.BeginGroup);
        case '}':
          return this.delimiter(LatexTokenKind.EndGroup);
        case '[':
          return this.delimiter(LatexTokenKind.BeginOptions);
        case ']':
          return this.delimiter(LatexTokenKind.EndOptions);
        case '$':
          return this.math();
        case '\\':
          return this.command();
        default:
          if (isWhiteSpace(c)) {
            this.stream.next();
          } else {
            return this.word();
          }
      }
    }
  }

  private delimiter(kind: LatexTokenKind): LatexToken {
    const startPosition = this.stream.position;
    this.stream.next();
    const text = this.stream.text.substring(
      this.stream.index - 1,
      this.stream.index,
    );
    return new LatexToken(startPosition, text, kind);
  }

  private math(): LatexToken {
    const startPosition = this.stream.position;
    const startIndex = this.stream.index;
    this.stream.next();
    if (this.stream.available && this.stream.peek() === '$') {
      this.stream.next();
    }
    const text = this.stream.text.substring(startIndex, this.stream.index);
    return new LatexToken(startPosition, text, LatexTokenKind.Math);
  }

  private command(): LatexToken {
    const startPostion = this.stream.position;
    const startIndex = this.stream.index;
    this.stream.next();
    let escape = true;
    while (this.stream.available && isCommandChar(this.stream.peek())) {
      this.stream.next();
      escape = false;
    }

    if (
      this.stream.available &&
      this.stream.peek() !== '\r' &&
      this.stream.peek() !== '\n' &&
      (escape || this.stream.peek() === '*')
    ) {
      this.stream.next();
    }

    const text = this.stream.text.substring(startIndex, this.stream.index);
    return new LatexToken(startPostion, text, LatexTokenKind.Command);
  }

  private word(): LatexToken {
    function isWordChar(c: string): boolean {
      return (
        !isWhiteSpace(c) &&
        c !== '%' &&
        c !== '{' &&
        c !== '}' &&
        c !== '[' &&
        c !== ']' &&
        c !== '\\' &&
        c !== '$'
      );
    }

    const startPosition = this.stream.position;
    const startIndex = this.stream.index;
    do {
      this.stream.next();
    } while (this.stream.available && isWordChar(this.stream.peek()));

    const text = this.stream.text.substring(startIndex, this.stream.index);
    return new LatexToken(startPosition, text, LatexTokenKind.Word);
  }
}
