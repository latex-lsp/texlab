import { Location } from 'vscode-languageserver';
import { Language } from '../language';
import * as range from '../range';
import { ReferenceProvider } from './provider';

export const LatexLabelReferenceProvider: ReferenceProvider = {
  execute: async ({ document, params, relatedDocuments }) => {
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    const defintion = document.tree.labelDefinitions.find(x =>
      range.contains(x.command.range, params.position),
    );

    if (defintion === undefined) {
      return [];
    }

    const references: Location[] = [];
    for (const { uri, tree } of relatedDocuments) {
      if (tree.language === Language.Latex) {
        const locations: Location[] = tree.labelReferences
          .filter(x => x.name.text === defintion.name.text)
          .map(x => ({ uri: uri.toString(), range: x.command.range }));

        references.push(...locations);
      }
    }

    return references;
  },
};
