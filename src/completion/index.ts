import {
  CompletionItem,
  CompletionParams,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { Language } from '../language';
import { BIBTEX_FIELDS } from '../metadata/bibtexField';
import { BIBTEX_TYPES } from '../metadata/bibtexType';
import { concat, FeatureContext, FeatureProvider } from '../provider';
import * as range from '../range';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import * as factory from './factory';

export type CompletionProvider = FeatureProvider<
  TextDocumentPositionParams,
  CompletionItem[]
>;

export class BibtexFieldNameCompletionProvider implements CompletionProvider {
  private static ITEMS = BIBTEX_FIELDS.map(factory.createFieldName);

  public async execute(
    context: FeatureContext<TextDocumentPositionParams>,
  ): Promise<CompletionItem[]> {
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

    return field || entry ? BibtexFieldNameCompletionProvider.ITEMS : [];
  }
}

export class BibtexEntryTypeCompletionProvider implements CompletionProvider {
  private static ITEMS = BIBTEX_TYPES.map(factory.createEntryType);

  public async execute(
    context: FeatureContext<TextDocumentPositionParams>,
  ): Promise<CompletionItem[]> {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    for (const node of document.tree.root.children) {
      if (node.kind !== BibtexSyntaxKind.Comment) {
        if (range.contains(node.type.range, params.position)) {
          return BibtexEntryTypeCompletionProvider.ITEMS;
        }
      }
    }
    return [];
  }
}

export const completionProvider = concat(
  new BibtexFieldNameCompletionProvider(),
  new BibtexEntryTypeCompletionProvider(),
);
