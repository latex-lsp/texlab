import { EOL } from 'os';
import {
  BibtexContentSyntax,
  BibtexDeclarationSyntax,
  BibtexFieldSyntax,
  BibtexSyntaxKind,
} from '../syntax/bibtex/ast';
import { BibtexToken } from '../syntax/bibtex/lexer';

export interface BibtexFormatterConfig {
  lineLength: number;
}

export class BibtexFormatter {
  private readonly indent: string;

  constructor(
    insertSpaces: boolean,
    private readonly tabSize: number,
    private readonly lineLength: number,
  ) {
    if (this.lineLength <= 0) {
      this.lineLength = Number.MAX_SAFE_INTEGER;
    }

    if (insertSpaces) {
      this.indent = '';
      for (let i = 0; i < tabSize; i++) {
        this.indent += ' ';
      }
    } else {
      this.indent = '\t';
    }
  }

  public format(
    node: BibtexDeclarationSyntax | BibtexFieldSyntax | BibtexContentSyntax,
    align: number = 0,
  ): string {
    let text = '';
    switch (node.kind) {
      case BibtexSyntaxKind.Preamble:
        text += node.type.text.toLowerCase();
        text += '{';
        if (node.content === undefined) {
          break;
        }
        text += this.format(node.content, text.length);
        text += '}';
        break;
      case BibtexSyntaxKind.String:
        text += node.type.text.toLowerCase();
        text += '{';
        if (node.name === undefined) {
          break;
        }
        text += node.name.text;
        text += ' = ';
        if (node.value === undefined) {
          break;
        }
        text += this.format(node.value, text.length);
        text += '}';
        break;
      case BibtexSyntaxKind.Entry:
        text += node.type.text.toLowerCase();
        text += '{';
        if (node.name === undefined) {
          break;
        }
        text += node.name.text;
        text += ',';
        text += EOL;
        node.fields.forEach(x => (text += this.format(x)));
        text += '}';
        break;
      case BibtexSyntaxKind.Field:
        text += this.indent;
        text += node.name.text.toLowerCase();
        text += ' = ';
        if (node.content === undefined) {
          break;
        }
        text += this.format(
          node.content,
          this.tabSize + node.name.text.length + 3,
        );
        text += ',';
        text += EOL;
        break;
      default:
        const tokens = getTokens(node);
        text += tokens[0].text;
        let length = align + tokens[0].length;
        for (let i = 1; i < tokens.length; i++) {
          const previous = tokens[i - 1];
          const current = tokens[i];

          const insertSpace = shouldInsertSpace(previous, current);
          const spaceLength = insertSpace ? 1 : 0;

          if (length + current.length + spaceLength > this.lineLength) {
            text += EOL;
            text += this.indent;
            for (let j = 0; j < align - this.tabSize + 1; j++) {
              text += ' ';
            }
            length = align;
          } else if (insertSpace) {
            text += ' ';
            length++;
          }
          text += current.text;
          length += current.length;
        }
        break;
    }
    return text;
  }
}

function shouldInsertSpace(
  previous: BibtexToken,
  current: BibtexToken,
): boolean {
  return (
    previous.line !== current.line ||
    previous.end.character < current.start.character
  );
}

function getTokens(content: BibtexContentSyntax): BibtexToken[] {
  const tokens: BibtexToken[] = [];
  function visit(node: BibtexContentSyntax) {
    switch (node.kind) {
      case BibtexSyntaxKind.Word:
        tokens.push(node.token);
        break;
      case BibtexSyntaxKind.Command:
        tokens.push(node.token);
        break;
      case BibtexSyntaxKind.QuotedContent:
      case BibtexSyntaxKind.BracedContent:
        tokens.push(node.left);
        node.children.forEach(x => visit(x));
        if (node.right !== undefined) {
          tokens.push(node.right);
        }
        break;
      case BibtexSyntaxKind.Concat:
        visit(node.left);
        tokens.push(node.operator);
        if (node.right !== undefined) {
          visit(node.right);
        }
        break;
    }
  }
  visit(content);
  return tokens;
}
