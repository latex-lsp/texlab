import { FoldingRange, FoldingRangeKind } from 'vscode-languageserver';
import { Language } from '../language';
import { LatexSection } from '../syntax/latex/analysis';
import { FoldingProvider } from './provider';

export const LatexSectionFoldingProvider: FoldingProvider = {
  execute: async ({ document }) => {
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    const foldings: FoldingRange[] = [];
    const sections: LatexSection[] = document.tree.sections;
    for (let i = 0; i < sections.length; i++) {
      const current = sections[i];
      let next: LatexSection | undefined;
      for (let j = i + 1; j < sections.length; j++) {
        next = sections[j];
        if (current.level >= sections[j].level) {
          break;
        }
      }

      if (next !== undefined) {
        foldings.push({
          startLine: current.command.end.line,
          startCharacter: current.command.end.character,
          endLine: next.command.start.line - 1,
          endCharacter: 0,
          kind: FoldingRangeKind.Region,
        });
      }
    }
    return foldings;
  },
};
