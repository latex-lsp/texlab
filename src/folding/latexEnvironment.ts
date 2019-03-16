import { FoldingRange, FoldingRangeKind } from 'vscode-languageserver';
import { Language } from '../language';
import { FoldingProvider } from './provider';

export const LatexEnvironmentFoldingProvider: FoldingProvider = {
  execute: async ({ document }) => {
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    const foldings: FoldingRange[] = [];
    document.tree.environments.forEach(environment => {
      foldings.push({
        startLine: environment.begin.end.line,
        startCharacter: environment.begin.end.character,
        endLine: environment.end.start.line,
        endCharacter: environment.end.start.character,
        kind: FoldingRangeKind.Region,
      });
    });
    return foldings;
  },
};
