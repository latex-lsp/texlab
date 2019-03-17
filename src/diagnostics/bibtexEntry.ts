import {
  Diagnostic,
  DiagnosticSeverity,
  Position,
  Range,
} from 'vscode-languageserver';
import { Language } from '../language';
import { BibtexContentSyntax, BibtexSyntaxKind } from '../syntax/bibtex/ast';
import { DiagnosticsProvider } from './provider';

export const BibtexEntryDiagnosticsProvider: DiagnosticsProvider = {
  execute: async ({ document }) => {
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    const diagnostics: Diagnostic[] = [];
    function add(code: ErrorCode, position: Position) {
      const range = { start: position, end: position };
      diagnostics.push(createDiagnostic(code, range));
    }

    for (const entry of document.tree.entries) {
      if (entry.left === undefined) {
        add(ErrorCode.MissingBeginBrace, entry.type.end);
        continue;
      }

      if (entry.name === undefined) {
        add(ErrorCode.MissingEntryName, entry.left.end);
        continue;
      }

      if (entry.comma === undefined) {
        add(ErrorCode.MissingComma, entry.name.end);
        continue;
      }

      for (let i = 0; i < entry.fields.length; i++) {
        const field = entry.fields[i];
        if (field.assign === undefined) {
          add(ErrorCode.MissingAssign, field.name.end);
          continue;
        }

        if (field.content === undefined) {
          add(ErrorCode.MissingContent, field.assign.end);
          continue;
        }

        diagnostics.push(...analyzeContent(field.content));

        if (i !== entry.fields.length - 1 && field.comma === undefined) {
          add(ErrorCode.MissingComma, field.content.end);
          continue;
        }
      }

      if (entry.right === undefined) {
        const position =
          entry.fields.length > 0
            ? entry.fields[entry.fields.length - 1].end
            : entry.comma.end;
        add(ErrorCode.MissingEndBrace, position);
        continue;
      }
    }

    return diagnostics;
  },
};

function analyzeContent(content: BibtexContentSyntax): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  function visit(node: BibtexContentSyntax) {
    const range = { start: node.end, end: node.end };
    switch (node.kind) {
      case BibtexSyntaxKind.QuotedContent:
        node.children.forEach(visit);
        if (node.right === undefined) {
          diagnostics.push(createDiagnostic(ErrorCode.MissingQuote, range));
        }
        break;
      case BibtexSyntaxKind.BracedContent:
        node.children.forEach(visit);
        if (node.right === undefined) {
          diagnostics.push(createDiagnostic(ErrorCode.MissingEndBrace, range));
        }
        break;
      case BibtexSyntaxKind.Concat:
        visit(node.left);
        if (node.right === undefined) {
          diagnostics.push(createDiagnostic(ErrorCode.MissingContent, range));
        }
        break;
    }
  }
  visit(content);
  return diagnostics;
}

export enum ErrorCode {
  MissingBeginBrace,
  MissingEntryName,
  MissingComma,
  MissingEndBrace,
  MissingAssign,
  MissingContent,
  MissingQuote,
}

export function createDiagnostic(code: ErrorCode, range: Range): Diagnostic {
  let message: string;
  switch (code) {
    case ErrorCode.MissingBeginBrace:
      message = 'Expecting a curly bracket: "{"';
      break;
    case ErrorCode.MissingEntryName:
      message = 'Expecting an entry name';
      break;
    case ErrorCode.MissingComma:
      message = 'Expecting a comma: ","';
      break;
    case ErrorCode.MissingEndBrace:
      message = 'Expecting a curly bracket: "}"';
      break;
    case ErrorCode.MissingAssign:
      message = 'Expecting an equals sign: "="';
      break;
    case ErrorCode.MissingContent:
      message = 'Expecting content';
      break;
    case ErrorCode.MissingQuote:
      message = "Expecting a quote: '\"'";
      break;
    default:
      message = 'Unknown error';
      break;
  }

  return {
    source: 'bibtex',
    range,
    message,
    severity: DiagnosticSeverity.Error,
  };
}
