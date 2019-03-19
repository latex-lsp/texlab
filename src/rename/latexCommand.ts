import { TextEdit } from 'vscode-languageserver';
import { Language } from '../language';
import * as range from '../range';
import { LatexCommandSyntax } from '../syntax/latex/ast';
import { RenameProvider } from './provider';

export const LatexCommandRenameProvider: RenameProvider = {
  execute: async ({ document, relatedDocuments, params }) => {
    if (document.tree.language !== Language.Latex) {
      return undefined;
    }

    const command = document.tree.descendants
      .filter(LatexCommandSyntax.is)
      .find(x => range.contains(x.name.range, params.position));

    if (command === undefined) {
      return undefined;
    }

    const changes: { [uri: string]: TextEdit[] } = {};
    for (const { uri, tree } of relatedDocuments) {
      if (tree.language !== Language.Latex) {
        continue;
      }

      const edits = tree.descendants
        .filter(LatexCommandSyntax.is)
        .filter(x => x.name.text === command.name.text)
        .map(x => ({ range: x.name.range, newText: '\\' + params.newName }));

      changes[uri.toString()] = edits;
    }
    return { changes };
  },
};
