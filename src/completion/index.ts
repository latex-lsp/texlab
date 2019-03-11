import { CompletionItem, CompletionParams } from 'vscode-languageserver';
import { FeatureContext, LanguageFeature } from '../feature';
import { Language } from '../language';
import { BIBTEX_FIELDS } from '../metadata/bibtexField';
import { BIBTEX_TYPES } from '../metadata/bibtexType';
import * as range from '../range';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import * as factory from './factory';

export type CompletionProvider = LanguageFeature<
  CompletionParams,
  CompletionItem[]
>;

export class BibtexFieldNameProvider implements CompletionProvider {
  private static ITEMS = BIBTEX_FIELDS.map(factory.createFieldName);

  public async execute(
    context: FeatureContext<CompletionParams>,
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

    return field || entry ? BibtexFieldNameProvider.ITEMS : [];
  }
}

export class BibtexEntryTypeProvider implements CompletionProvider {
  private static ITEMS = BIBTEX_TYPES.map(factory.createEntryType);

  public async execute(
    context: FeatureContext<CompletionParams>,
  ): Promise<CompletionItem[]> {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    for (const node of document.tree.root.children) {
      if (node.kind !== BibtexSyntaxKind.Comment) {
        if (range.contains(node.type.range, params.position)) {
          return BibtexEntryTypeProvider.ITEMS;
        }
      }
    }
    return [];
  }
}

export const CompletionProvider = LanguageFeature.concat(
  new BibtexFieldNameProvider(),
  new BibtexEntryTypeProvider(),
);
