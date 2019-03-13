import { Language } from '../language';
import { BIBTEX_TYPES } from '../metadata/bibtexType';
import * as range from '../range';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import * as factory from './factory';
import { CompletionProvider } from './provider';

const ITEMS = BIBTEX_TYPES.map(factory.createEntryType);

export const BibtexEntryTypeCompletionProvider: CompletionProvider = {
  execute: async context => {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    for (const node of document.tree.root.children) {
      if (node.kind !== BibtexSyntaxKind.Comment) {
        if (range.contains(node.type.range, params.position)) {
          return ITEMS;
        }
      }
    }
    return [];
  },
};
