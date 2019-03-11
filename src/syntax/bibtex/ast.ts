import { Position, Range } from 'vscode-languageserver';
import { SyntaxNode } from '../syntaxNode';
import { BibtexToken } from './lexer';

export enum BibtexSyntaxKind {
  Document,
  Comment,
  Preamble,
  String,
  Entry,
  Field,
  Word,
  Command,
  QuotedContent,
  BracedContent,
  Concat,
}

export type BibtexSyntaxNode =
  | BibtexDocumentSyntax
  | BibtexCommentSyntax
  | BibtexDeclarationSyntax
  | BibtexFieldSyntax
  | BibtexContentSyntax;

export class BibtexDocumentSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Document;
  public readonly range: Range;

  constructor(public readonly children: BibtexDocumentItemSyntax[]) {
    super();
    this.kind = BibtexSyntaxKind.Document;
    this.range =
      children.length === 0
        ? { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } }
        : { start: children[0].start, end: children[children.length - 1].end };
  }
}

export type BibtexDocumentItemSyntax =
  | BibtexCommentSyntax
  | BibtexDeclarationSyntax;

export class BibtexCommentSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Comment;
  public readonly range: Range;

  constructor(public readonly token: BibtexToken) {
    super();
    this.kind = BibtexSyntaxKind.Comment;
    this.range = token.range;
  }
}

export type BibtexDeclarationSyntax =
  | BibtexPreambleSyntax
  | BibtexStringSyntax
  | BibtexEntrySyntax;

export class BibtexPreambleSyntax extends SyntaxNode {
  public kind: BibtexSyntaxKind.Preamble;
  public range: Range;

  constructor(
    public readonly type: BibtexToken,
    public readonly left: BibtexToken | undefined,
    public readonly content: BibtexContentSyntax | undefined,
    public readonly right: BibtexToken | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.Preamble;
    let end: Position;
    if (right !== undefined) {
      end = type.end;
    } else if (content !== undefined) {
      end = content.end;
    } else if (left !== undefined) {
      end = left.end;
    } else {
      end = type.end;
    }
    this.range = { start: type.start, end };
  }
}

export class BibtexStringSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.String;
  public readonly range: Range;

  constructor(
    public readonly type: BibtexToken,
    public readonly left: BibtexToken | undefined,
    public readonly name: BibtexToken | undefined,
    public readonly assign: BibtexToken | undefined,
    public readonly value: BibtexContentSyntax | undefined,
    public readonly right: BibtexToken | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.String;
    let end: Position;
    if (right !== undefined) {
      end = right.end;
    } else if (value !== undefined) {
      end = value.end;
    } else if (assign !== undefined) {
      end = assign.end;
    } else if (name !== undefined) {
      end = name.end;
    } else if (left !== undefined) {
      end = left.end;
    } else {
      end = type.end;
    }
    this.range = { start: type.start, end };
  }
}

export class BibtexEntrySyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Entry;
  public readonly range: Range;

  constructor(
    public readonly type: BibtexToken,
    public readonly left: BibtexToken | undefined,
    public readonly name: BibtexToken | undefined,
    public readonly comma: BibtexToken | undefined,
    public readonly fields: BibtexFieldSyntax[],
    public readonly right: BibtexToken | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.Entry;
    let end: Position;
    if (right !== undefined) {
      end = right.end;
    } else if (fields.length > 0) {
      end = fields[fields.length - 1].end;
    } else if (comma !== undefined) {
      end = comma.end;
    } else if (name !== undefined) {
      end = name.end;
    } else if (left !== undefined) {
      end = left.end;
    } else {
      end = type.end;
    }
    this.range = { start: type.start, end };
  }
}

export class BibtexFieldSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Field;
  public readonly range: Range;

  constructor(
    public readonly name: BibtexToken,
    public readonly assign: BibtexToken | undefined,
    public readonly content: BibtexContentSyntax | undefined,
    public readonly comma: BibtexToken | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.Field;
    let end: Position;
    if (comma !== undefined) {
      end = comma.end;
    } else if (content !== undefined) {
      end = content.end;
    } else if (assign !== undefined) {
      end = assign.end;
    } else {
      end = name.end;
    }
    this.range = { start: name.start, end };
  }
}

type BibtexContentSyntax =
  | BibtexWordSyntax
  | BibtexCommandSyntax
  | BibtexQuotedContentSyntax
  | BibtexBracedContentSyntax
  | BibtexConcatSyntax;

export class BibtexWordSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Word;
  public readonly range: Range;

  constructor(public readonly token: BibtexToken) {
    super();
    this.kind = BibtexSyntaxKind.Word;
    this.range = token.range;
  }
}

export class BibtexCommandSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Command;
  public readonly range: Range;

  constructor(public readonly token: BibtexToken) {
    super();
    this.kind = BibtexSyntaxKind.Command;
    this.range = token.range;
  }
}

export class BibtexQuotedContentSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.QuotedContent;
  public readonly range: Range;

  constructor(
    public readonly left: BibtexToken,
    public readonly children: BibtexContentSyntax[],
    public readonly right: BibtexToken | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.QuotedContent;
    let end: Position;
    if (right !== undefined) {
      end = right.end;
    } else if (children.length > 0) {
      end = children[children.length - 1].end;
    } else {
      end = left.end;
    }
    this.range = { start: left.start, end };
  }
}

export class BibtexBracedContentSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.BracedContent;
  public readonly range: Range;

  constructor(
    public readonly left: BibtexToken,
    public readonly children: BibtexContentSyntax[],
    public readonly right: BibtexToken | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.BracedContent;
    let end: Position;
    if (right !== undefined) {
      end = right.end;
    } else if (children.length > 0) {
      end = children[children.length - 1].end;
    } else {
      end = left.end;
    }
    this.range = { start: left.start, end };
  }
}

export class BibtexConcatSyntax extends SyntaxNode {
  public readonly kind: BibtexSyntaxKind.Concat;
  public readonly range: Range;

  constructor(
    public readonly left: BibtexContentSyntax,
    public readonly operator: BibtexToken,
    public readonly right: BibtexContentSyntax | undefined,
  ) {
    super();
    this.kind = BibtexSyntaxKind.Concat;
    const end = right === undefined ? operator.end : right.end;
    this.range = { start: left.start, end };
  }
}

export function descendants(root: BibtexSyntaxNode) {
  const results: BibtexSyntaxNode[] = [];
  function visit(node: BibtexSyntaxNode) {
    results.push(node);
    switch (node.kind) {
      case BibtexSyntaxKind.Document:
        node.children.forEach(visit);
        break;
      case BibtexSyntaxKind.Comment:
        break;
      case BibtexSyntaxKind.Preamble:
        if (node.content !== undefined) {
          visit(node.content);
        }
        break;
      case BibtexSyntaxKind.String:
        if (node.value !== undefined) {
          visit(node.value);
        }
        break;
      case BibtexSyntaxKind.Entry:
        node.fields.forEach(visit);
        break;
      case BibtexSyntaxKind.Field:
        if (node.content !== undefined) {
          visit(node.content);
        }
        break;
      case BibtexSyntaxKind.Word:
        break;
      case BibtexSyntaxKind.Command:
        break;
      case BibtexSyntaxKind.QuotedContent:
        node.children.forEach(visit);
        break;
      case BibtexSyntaxKind.BracedContent:
        node.children.forEach(visit);
        break;
      case BibtexSyntaxKind.Concat:
        visit(node.left);
        if (node.right !== undefined) {
          visit(node.right);
        }
        break;
    }
  }
  visit(root);
  return results;
}
