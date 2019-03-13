import { Language } from '../language';
import { BIBTEX_FIELDS } from '../metadata/bibtexField';
import * as range from '../range';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import * as factory from './factory';
import { CompletionProvider } from './provider';

const ITEMS = BIBTEX_FIELDS.map(factory.createFieldName);

export const BibtexFieldNameCompletionProvider: CompletionProvider = {
  execute: async context => {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    const node = document.tree.find(params.position)!;

    const field =
      node.kind === BibtexSyntaxKind.Field &&
      range.contains(node.name.range, params.position);

    const entry =
      node.kind === BibtexSyntaxKind.Entry &&
      !range.contains(node.type.range, params.position) &&
      node.name !== undefined &&
      !range.contains(node.name.range, params.position);

    return field || entry ? ITEMS : [];
  },
};
