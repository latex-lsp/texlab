import { Position } from 'vscode-languageserver';
import { isCommandChar, isWhiteSpace } from '../character';
import { CharStream } from '../charStream';
import { Token, TokenSource } from '../token';

export enum BibtexTokenKind {
  PreambleType,
  StringType,
  EntryType,
  Word,
  Command,
  Assign,
  Comma,
  Concat,
  Quote,
  BeginBrace,
  EndBrace,
  BeginParen,
  EndParen,
}

export class BibtexToken extends Token {
  constructor(
    start: Position,
    text: string,
    public readonly kind: BibtexTokenKind,
  ) {
    super(start, text);
  }
}

export class BibtexLexer implements TokenSource<BibtexToken> {
  private readonly stream: CharStream;

  constructor(text: string) {
    this.stream = new CharStream(text);
  }

  public next(): BibtexToken | undefined {
    while (this.stream.available) {
      const c = this.stream.peek();
      switch (c) {
        case '@':
          return this.type();
        case '=':
          return this.singleCharacter(BibtexTokenKind.Assign);
        case ',':
          return this.singleCharacter(BibtexTokenKind.Comma);
        case '#':
          return this.singleCharacter(BibtexTokenKind.Concat);
        case '"':
          return this.singleCharacter(BibtexTokenKind.Quote);
        case '{':
          return this.singleCharacter(BibtexTokenKind.BeginBrace);
        case '}':
          return this.singleCharacter(BibtexTokenKind.EndBrace);
        case '(':
          return this.singleCharacter(BibtexTokenKind.BeginParen);
        case ')':
          return this.singleCharacter(BibtexTokenKind.EndParen);
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

  private type(): BibtexToken {
    function isTypeChar(c: string): boolean {
      return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
    }

    const startPosition = this.stream.position;
    const startIndex = this.stream.index;
    do {
      this.stream.next();
    } while (this.stream.available && isTypeChar(this.stream.peek()));
    const text = this.stream.text.substring(startIndex, this.stream.index);
    let kind: BibtexTokenKind;
    switch (text.toLowerCase()) {
      case '@preamble':
        kind = BibtexTokenKind.PreambleType;
        break;
      case '@string':
        kind = BibtexTokenKind.StringType;
        break;
      default:
        kind = BibtexTokenKind.EntryType;
        break;
    }
    return new BibtexToken(startPosition, text, kind);
  }

  private singleCharacter(kind: BibtexTokenKind): BibtexToken {
    const startPosition = this.stream.position;
    this.stream.next();
    const text = this.stream.text.substring(
      this.stream.index - 1,
      this.stream.index,
    );
    return new BibtexToken(startPosition, text, kind);
  }

  private command(): BibtexToken {
    const startPosition = this.stream.position;
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
    return new BibtexToken(startPosition, text, BibtexTokenKind.Command);
  }

  private word(): BibtexToken {
    function isWordChar(c: string): boolean {
      return (
        !isWhiteSpace(c) &&
        c !== '@' &&
        c !== '=' &&
        c !== ',' &&
        c !== '#' &&
        c !== '"' &&
        c !== '{' &&
        c !== '}' &&
        c !== '(' &&
        c !== ')'
      );
    }

    const startPosition = this.stream.position;
    const startIndex = this.stream.index;
    do {
      this.stream.next();
    } while (this.stream.available && isWordChar(this.stream.peek()));

    const text = this.stream.text.substring(startIndex, this.stream.index);
    return new BibtexToken(startPosition, text, BibtexTokenKind.Word);
  }
}
