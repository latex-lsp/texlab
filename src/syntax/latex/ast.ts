import { Position, Range } from 'vscode-languageserver';
import { SyntaxNode } from '../syntaxNode';
import { LatexToken } from './lexer';

export enum LatexSyntaxKind {
  Document,
  Group,
  Command,
  Text,
}

export type LatexSyntaxNode =
  | LatexDocumentSyntax
  | LatexGroupSyntax
  | LatexCommandSyntax
  | LatexTextSyntax;

export class LatexDocumentSyntax extends SyntaxNode {
  public readonly kind: LatexSyntaxKind.Document;
  public readonly range: Range;

  constructor(public readonly children: LatexSyntaxNode[]) {
    super();
    this.kind = LatexSyntaxKind.Document;
    if (children.length > 0) {
      this.range = {
        start: children[0].start,
        end: children[children.length - 1].end,
      };
    } else {
      this.range = {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 0 },
      };
    }
  }
}

export class LatexGroupSyntax extends SyntaxNode {
  public readonly kind: LatexSyntaxKind.Group;
  public readonly range: Range;

  constructor(
    public readonly left: LatexToken,
    public readonly children: LatexSyntaxNode[],
    public readonly right: LatexToken | undefined,
  ) {
    super();
    this.kind = LatexSyntaxKind.Group;
    let end: Position;
    if (right !== undefined) {
      end = right.end;
    } else {
      if (children.length > 0) {
        end = children[children.length - 1].end;
      } else {
        end = left.end;
      }
    }
    this.range = {
      start: left.start,
      end,
    };
  }
}

export class LatexCommandSyntax extends SyntaxNode {
  public static is(node: LatexSyntaxNode): node is LatexCommandSyntax {
    return node.kind === LatexSyntaxKind.Command;
  }

  public readonly kind: LatexSyntaxKind.Command;
  public readonly range: Range;

  constructor(
    public readonly name: LatexToken,
    public readonly options: LatexGroupSyntax | undefined,
    public readonly args: LatexGroupSyntax[],
  ) {
    super();
    this.kind = LatexSyntaxKind.Command;
    let end: Position;
    if (args.length > 0) {
      end = args[args.length - 1].end;
    } else {
      if (options !== undefined) {
        end = options.end;
      } else {
        end = name.end;
      }
    }
    this.range = {
      start: name.start,
      end,
    };
  }

  public extractText(index: number): LatexTextSyntax | undefined {
    if (this.args.length > index && this.args[index].children.length === 1) {
      const child = this.args[index].children[0];
      return child.kind === LatexSyntaxKind.Text ? child : undefined;
    } else {
      return undefined;
    }
  }

  public extractWord(index: number): LatexToken | undefined {
    const text = this.extractText(index);
    return text === undefined || text.words.length !== 1
      ? undefined
      : text.words[0];
  }
}

export class LatexTextSyntax extends SyntaxNode {
  public static is(node: LatexSyntaxNode): node is LatexTextSyntax {
    return node.kind === LatexSyntaxKind.Text;
  }

  public readonly kind: LatexSyntaxKind.Text;
  public readonly range: Range;

  constructor(public readonly words: LatexToken[]) {
    super();
    this.kind = LatexSyntaxKind.Text;
    this.range = {
      start: words[0].start,
      end: words[words.length - 1].end,
    };
  }
}

export function descendants(root: LatexSyntaxNode) {
  const results: LatexSyntaxNode[] = [];
  function visit(node: LatexSyntaxNode) {
    results.push(node);
    switch (node.kind) {
      case LatexSyntaxKind.Document:
      case LatexSyntaxKind.Group:
        node.children.forEach(visit);
        break;
      case LatexSyntaxKind.Command:
        if (node.options !== undefined) {
          visit(node.options);
        }
        node.args.forEach(visit);
        break;
      case LatexSyntaxKind.Text:
        break;
    }
  }
  visit(root);
  return results;
}
