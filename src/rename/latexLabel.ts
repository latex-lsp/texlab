import { TextEdit } from 'vscode-languageserver';
import { Language } from '../language';
import * as range from '../range';
import { RenameProvider } from './provider';

export const LatexLabelRenameProvider: RenameProvider = {
  execute: async ({ document, relatedDocuments, params }) => {
    if (document.tree.language !== Language.Latex) {
      return undefined;
    }

    const label = [
      ...document.tree.labelDefinitions,
      ...document.tree.labelReferences,
    ].find(x => range.contains(x.name.range, params.position));

    if (label === undefined) {
      return undefined;
    }

    const changes: { [uri: string]: TextEdit[] } = {};
    for (const { uri, tree } of relatedDocuments) {
      if (tree.language !== Language.Latex) {
        continue;
      }

      const labels = [...tree.labelDefinitions, ...tree.labelReferences];
      changes[uri.toString()] = labels
        .filter(x => x.name.text === label.name.text)
        .map(x => ({ range: x.name.range, newText: params.newName }));
    }
    return { changes };
  },
};
