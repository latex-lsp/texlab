import os from 'os';
import { MarkupKind } from 'vscode-languageserver';
import { LatexComponentSource } from '../completion/latex/data/component';
import { Language } from '../language';
import * as range from '../range';
import { LatexCommandSyntax } from '../syntax/latex/ast';
import { HoverProvider } from './provider';

type Factory = (database: LatexComponentSource) => HoverProvider;

export const LatexCommandHoverProvider: Factory = database => ({
  execute: async context => {
    const { document, params, relatedDocuments } = context;
    if (document.tree.language !== Language.Latex) {
      return undefined;
    }

    const command = document.tree.descendants
      .filter(LatexCommandSyntax.is)
      .find(x => range.contains(x.name.range, params.position));

    if (command === undefined) {
      return undefined;
    }

    const components = database
      .relatedComponents(relatedDocuments)
      .filter(x => x.commands.includes(command.name.text.substring(1)))
      .reduce((acc, x) => acc.concat(x.fileNames), [] as string[]);

    if (components.length === 0) {
      return undefined;
    }

    return {
      contents: {
        kind: MarkupKind.PlainText,
        value: components.join(', '),
      },
    };
  },
});
