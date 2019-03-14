import { MarkupKind } from 'vscode-languageserver';
import { Language } from '../language';
import { getFieldDocumentation, parseFieldName } from '../metadata/bibtexField';
import * as range from '../range';
import { BibtexFieldSyntax } from '../syntax/bibtex/ast';
import { HoverProvider } from './provider';

export const BibtexFieldHoverProvider: HoverProvider = {
  execute: async context => {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return undefined;
    }

    const node = document.tree.descendants
      .filter(BibtexFieldSyntax.is)
      .find(x => range.contains(x.name.range, params.position));

    if (node === undefined) {
      return undefined;
    }

    const field = parseFieldName(node.name.text);
    if (field === undefined) {
      return undefined;
    }

    return {
      contents: {
        kind: MarkupKind.Markdown,
        value: getFieldDocumentation(field),
      },
    };
  },
};
