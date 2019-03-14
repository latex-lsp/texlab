import { MarkupKind } from 'vscode-languageserver';
import { Language } from '../language';
import { getTypeDocumentation } from '../metadata/bibtexType';
import * as range from '../range';
import { HoverProvider } from './provider';

export const BibtexEntryTypeHoverProvider: HoverProvider = {
  execute: async context => {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return undefined;
    }

    const entry = document.tree.entries.find(x =>
      range.contains(x.type.range, params.position),
    );

    if (entry === undefined) {
      return undefined;
    }

    const type = entry.type.text.substring(1).toLowerCase();
    return {
      contents: {
        kind: MarkupKind.Markdown,
        value: getTypeDocumentation(type)!,
      },
    };
  },
};
