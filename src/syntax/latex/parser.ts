import { TokenBuffer } from '../token';
import {
  LatexCommandSyntax,
  LatexDocumentSyntax,
  LatexGroupSyntax,
  LatexSyntaxNode,
  LatexTextSyntax,
} from './ast';
import { LatexLexer, LatexToken, LatexTokenKind } from './lexer';

enum LatexScope {
  Document,
  Group,
  Options,
}

class LatexParser {
  constructor(private readonly tokens: TokenBuffer<LatexToken>) {}

  public document(): LatexDocumentSyntax {
    const children = this.content(LatexScope.Document);
    return new LatexDocumentSyntax(children);
  }

  private content(scope: LatexScope): LatexSyntaxNode[] {
    const children: LatexSyntaxNode[] = [];
    while (this.tokens.available) {
      switch (this.tokens.peek()!.kind) {
        case LatexTokenKind.Word:
        case LatexTokenKind.Math:
        case LatexTokenKind.BeginOptions:
          children.push(this.text(scope));
          break;
        case LatexTokenKind.Command:
          children.push(this.command());
          break;
        case LatexTokenKind.BeginGroup:
          children.push(this.group(LatexScope.Group));
          break;
        case LatexTokenKind.EndGroup:
          if (scope === LatexScope.Document) {
            this.tokens.next();
          } else {
            return children;
          }
          break;
        case LatexTokenKind.EndOptions:
          if (scope === LatexScope.Options) {
            return children;
          } else {
            children.push(this.text(scope));
          }
          break;
      }
    }
    return children;
  }

  private group(scope: LatexScope): LatexGroupSyntax {
    const left = this.tokens.next();
    const children = this.content(scope);
    const endKind =
      scope === LatexScope.Group
        ? LatexTokenKind.EndGroup
        : LatexTokenKind.EndOptions;

    const right = this.nextOfKind(endKind) ? this.tokens.next() : undefined;
    return new LatexGroupSyntax(left, children, right);
  }

  private command(): LatexCommandSyntax {
    const name = this.tokens.next();
    const options = this.nextOfKind(LatexTokenKind.BeginOptions)
      ? this.group(LatexScope.Options)
      : undefined;

    const args: LatexGroupSyntax[] = [];
    while (this.nextOfKind(LatexTokenKind.BeginGroup)) {
      args.push(this.group(LatexScope.Group));
    }

    return new LatexCommandSyntax(name, options, args);
  }

  private text(scope: LatexScope): LatexTextSyntax {
    const words: LatexToken[] = [];
    while (this.tokens.available) {
      const kind = this.tokens.peek()!.kind;
      const opts =
        kind === LatexTokenKind.EndOptions && scope !== LatexScope.Options;
      if (
        kind === LatexTokenKind.Word ||
        kind === LatexTokenKind.Math ||
        kind === LatexTokenKind.BeginOptions ||
        opts
      ) {
        words.push(this.tokens.next());
      } else {
        break;
      }
    }
    return new LatexTextSyntax(words);
  }

  private nextOfKind(kind: LatexTokenKind): boolean {
    const token = this.tokens.peek();
    return token !== undefined && token.kind === kind;
  }
}

export function parse(text: string): LatexDocumentSyntax {
  const lexer = new LatexLexer(text);
  const tokens = new TokenBuffer(lexer);
  const parser = new LatexParser(tokens);
  return parser.document();
}
