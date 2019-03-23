import os from 'os';
import { BibtexSyntaxKind, BibtexSyntaxNode } from './ast';
import { BibtexToken } from './lexer';

class BibtexPrinter {
  private text: string;
  private line: number;
  private character: number;

  constructor() {
    this.text = '';
    this.line = 0;
    this.character = 0;
  }

  public visit(node: BibtexSyntaxNode | undefined) {
    if (node === undefined) {
      return;
    }

    switch (node.kind) {
      case BibtexSyntaxKind.Document:
        node.children.forEach(x => this.visit(x));
        break;
      case BibtexSyntaxKind.Comment:
        this.visitToken(node.token);
        break;
      case BibtexSyntaxKind.Preamble:
        this.visitToken(node.type);
        this.visitToken(node.left);
        this.visit(node.content);
        this.visitToken(node.right);
        break;
      case BibtexSyntaxKind.String:
        this.visitToken(node.type);
        this.visitToken(node.left);
        this.visitToken(node.name);
        this.visitToken(node.assign);
        this.visit(node.value);
        this.visitToken(node.right);
        break;
      case BibtexSyntaxKind.Entry:
        this.visitToken(node.type);
        this.visitToken(node.left);
        this.visitToken(node.name);
        this.visitToken(node.comma);
        node.fields.forEach(x => this.visit(x));
        this.visitToken(node.right);
        break;
      case BibtexSyntaxKind.Field:
        this.visitToken(node.name);
        this.visitToken(node.assign);
        this.visit(node.content);
        this.visitToken(node.comma);
        break;
      case BibtexSyntaxKind.Word:
      case BibtexSyntaxKind.Command:
        this.visitToken(node.token);
        break;
      case BibtexSyntaxKind.QuotedContent:
      case BibtexSyntaxKind.BracedContent:
        this.visitToken(node.left);
        node.children.forEach(x => this.visit(x));
        this.visitToken(node.right);
        break;
      case BibtexSyntaxKind.Concat:
        this.visit(node.left);
        this.visitToken(node.operator);
        this.visit(node.right);
    }
  }

  public visitToken(token: BibtexToken | undefined) {
    if (token === undefined) {
      return;
    }

    while (this.line < token.line) {
      this.text += os.EOL;
      this.line++;
      this.character = 0;
    }

    while (this.character < token.character) {
      this.text += ' ';
      this.character++;
    }

    this.text += token.text;
    this.character += token.text.length;
  }

  public toString(): string {
    return this.text;
  }
}

export function print(node: BibtexSyntaxNode): string {
  const printer = new BibtexPrinter();
  printer.visit(node);
  return printer.toString();
}
