import { FoldingRange, FoldingRangeKind } from 'vscode-languageserver';
import { Language } from '../language';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import { FoldingProvider } from './provider';

export const BibtexDeclarationFoldingProvider: FoldingProvider = {
  execute: async ({ document }) => {
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    const foldings: FoldingRange[] = [];
    document.tree.root.children.forEach(declaration => {
      switch (declaration.kind) {
        case BibtexSyntaxKind.Preamble:
        case BibtexSyntaxKind.String:
        case BibtexSyntaxKind.Entry:
          const { type, right } = declaration;
          if (right === undefined) {
            break;
          }
          foldings.push({
            startLine: type.line,
            startCharacter: type.character,
            endLine: right.line,
            endCharacter: right.character,
            kind: FoldingRangeKind.Region,
          });
          break;
      }
    });
    return foldings;
  },
};
