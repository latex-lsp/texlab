import { Location } from 'vscode-languageserver';
import { Language } from '../language';
import * as range from '../range';
import { BibtexEntrySyntax } from '../syntax/bibtex/ast';
import { ReferenceProvider } from './provider';

export const BibtexEntryReferenceProvider: ReferenceProvider = {
  execute: async ({ document, params, relatedDocuments }) => {
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    const definitionName = document.tree.root.children
      .filter(BibtexEntrySyntax.is)
      .map(x => x.name)
      .find(x => x !== undefined && range.contains(x.range, params.position));

    if (definitionName === undefined) {
      return [];
    }

    const references: Location[] = [];
    for (const { uri, tree } of relatedDocuments) {
      if (tree.language === Language.Latex) {
        const locations: Location[] = tree.citations
          .filter(x => x.name.text === definitionName.text)
          .map(x => ({ uri: uri.toString(), range: x.command.range }));

        references.push(...locations);
      }
    }

    return references;
  },
};
