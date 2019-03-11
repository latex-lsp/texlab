import { TokenBuffer } from '../token';
import {
  BibtexBracedContentSyntax,
  BibtexCommandSyntax,
  BibtexCommentSyntax,
  BibtexConcatSyntax,
  BibtexContentSyntax,
  BibtexDocumentItemSyntax,
  BibtexDocumentSyntax,
  BibtexEntrySyntax,
  BibtexFieldSyntax,
  BibtexPreambleSyntax,
  BibtexQuotedContentSyntax,
  BibtexStringSyntax,
  BibtexWordSyntax,
} from './ast';
import { BibtexLexer, BibtexToken, BibtexTokenKind } from './lexer';

class BibtexParser {
  constructor(private readonly tokens: TokenBuffer<BibtexToken>) {}

  public document(): BibtexDocumentSyntax {
    const children: BibtexDocumentItemSyntax[] = [];
    while (this.tokens.available) {
      switch (this.tokens.peek()!.kind) {
        case BibtexTokenKind.PreambleType:
          children.push(this.preamble());
          break;
        case BibtexTokenKind.StringType:
          children.push(this.string());
          break;
        case BibtexTokenKind.EntryType:
          children.push(this.entry());
          break;
        default:
          const token = this.tokens.next();
          children.push(new BibtexCommentSyntax(token));
          break;
      }
    }
    return new BibtexDocumentSyntax(children);
  }

  private preamble(): BibtexPreambleSyntax {
    const type = this.tokens.next();
    const left = this.expect(
      BibtexTokenKind.BeginBrace,
      BibtexTokenKind.BeginParen,
    );
    if (left === undefined) {
      return new BibtexPreambleSyntax(type, undefined, undefined, undefined);
    }

    if (!this.canMatchContent()) {
      return new BibtexPreambleSyntax(type, left, undefined, undefined);
    }
    const content = this.content();

    const right = this.expect(
      BibtexTokenKind.EndBrace,
      BibtexTokenKind.EndParen,
    );
    return new BibtexPreambleSyntax(type, left, content, right);
  }

  private string(): BibtexStringSyntax {
    const type = this.tokens.next();

    const left = this.expect(
      BibtexTokenKind.BeginBrace,
      BibtexTokenKind.BeginParen,
    );
    if (left === undefined) {
      return new BibtexStringSyntax(
        type,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
      );
    }

    const name = this.expect(BibtexTokenKind.Word);
    if (name === undefined) {
      return new BibtexStringSyntax(
        type,
        left,
        undefined,
        undefined,
        undefined,
        undefined,
      );
    }

    const assign = this.expect(BibtexTokenKind.Assign);
    if (assign === undefined) {
      return new BibtexStringSyntax(
        type,
        left,
        name,
        undefined,
        undefined,
        undefined,
      );
    }

    if (!this.canMatchContent()) {
      return new BibtexStringSyntax(
        type,
        left,
        name,
        assign,
        undefined,
        undefined,
      );
    }
    const value = this.content();

    const right = this.expect(
      BibtexTokenKind.EndBrace,
      BibtexTokenKind.EndParen,
    );
    return new BibtexStringSyntax(type, left, name, assign, value, right);
  }

  private entry(): BibtexEntrySyntax {
    const type = this.tokens.next();

    const left = this.expect(
      BibtexTokenKind.BeginBrace,
      BibtexTokenKind.BeginParen,
    );
    if (left === undefined) {
      return new BibtexEntrySyntax(
        type,
        undefined,
        undefined,
        undefined,
        [],
        undefined,
      );
    }

    const name = this.expect(BibtexTokenKind.Word);
    if (name === undefined) {
      return new BibtexEntrySyntax(
        type,
        left,
        undefined,
        undefined,
        [],
        undefined,
      );
    }

    const comma = this.expect(BibtexTokenKind.Comma);
    if (comma === undefined) {
      return new BibtexEntrySyntax(type, left, name, undefined, [], undefined);
    }

    const fields: BibtexFieldSyntax[] = [];
    while (this.nextOfKind(BibtexTokenKind.Word)) {
      fields.push(this.field());
    }

    const right = this.expect(
      BibtexTokenKind.EndBrace,
      BibtexTokenKind.EndParen,
    );

    return new BibtexEntrySyntax(type, left, name, comma, fields, right);
  }

  private field(): BibtexFieldSyntax {
    const name = this.tokens.next();
    const assign = this.expect(BibtexTokenKind.Assign);
    if (assign === undefined) {
      return new BibtexFieldSyntax(name, undefined, undefined, undefined);
    }

    if (!this.canMatchContent()) {
      return new BibtexFieldSyntax(name, assign, undefined, undefined);
    }
    const content = this.content();

    const comma = this.expect(BibtexTokenKind.Comma);
    return new BibtexFieldSyntax(name, assign, content, comma);
  }

  private content(): BibtexContentSyntax {
    let left: BibtexContentSyntax;
    const token = this.tokens.next();
    switch (token.kind) {
      case BibtexTokenKind.PreambleType:
      case BibtexTokenKind.StringType:
      case BibtexTokenKind.EntryType:
      case BibtexTokenKind.Word:
      case BibtexTokenKind.Assign:
      case BibtexTokenKind.Comma:
      case BibtexTokenKind.BeginParen:
      case BibtexTokenKind.EndParen:
        left = new BibtexWordSyntax(token);
        break;
      case BibtexTokenKind.Command:
        left = new BibtexCommandSyntax(token);
        break;
      case BibtexTokenKind.Quote: {
        const children: BibtexContentSyntax[] = [];
        while (this.canMatchContent()) {
          const kind = this.tokens.peek()!.kind;
          if (kind === BibtexTokenKind.Quote) {
            break;
          }
          children.push(this.content());
        }
        const right = this.expect(BibtexTokenKind.Quote);
        left = new BibtexQuotedContentSyntax(token, children, right);
        break;
      }
      case BibtexTokenKind.BeginBrace: {
        const children: BibtexContentSyntax[] = [];
        while (this.canMatchContent()) {
          children.push(this.content());
        }
        const right = this.expect(BibtexTokenKind.EndBrace);
        left = new BibtexBracedContentSyntax(token, children, right);
        break;
      }
      default:
        throw Error('Unexpected token type: ' + token.kind);
    }
    const operator = this.expect(BibtexTokenKind.Concat);
    if (operator === undefined) {
      return left;
    } else {
      const right = this.canMatchContent() ? this.content() : undefined;
      return new BibtexConcatSyntax(left, operator, right);
    }
  }

  private canMatchContent(): boolean {
    const token = this.tokens.peek();
    if (token === undefined) {
      return false;
    }

    switch (token.kind) {
      case BibtexTokenKind.PreambleType:
      case BibtexTokenKind.StringType:
      case BibtexTokenKind.EntryType:
      case BibtexTokenKind.Word:
      case BibtexTokenKind.Command:
      case BibtexTokenKind.Assign:
      case BibtexTokenKind.Comma:
        return true;
      case BibtexTokenKind.Concat:
        return false;
      case BibtexTokenKind.Quote:
        return true;
      case BibtexTokenKind.BeginBrace:
        return true;
      case BibtexTokenKind.EndBrace:
        return false;
      case BibtexTokenKind.BeginParen:
      case BibtexTokenKind.EndParen:
        return true;
    }
  }

  private expect(...kinds: BibtexTokenKind[]): BibtexToken | undefined {
    const token = this.tokens.peek();
    return token !== undefined && kinds.some(x => x === token.kind)
      ? this.tokens.next()
      : undefined;
  }

  private nextOfKind(kind: BibtexTokenKind): boolean {
    const token = this.tokens.peek();
    return token !== undefined && token.kind === kind;
  }
}

export function parse(text: string): BibtexDocumentSyntax {
  const lexer = new BibtexLexer(text);
  const tokens = new TokenBuffer(lexer);
  const parser = new BibtexParser(tokens);
  return parser.document();
}
