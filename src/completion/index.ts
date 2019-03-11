import { CompletionItem, CompletionParams } from 'vscode-languageserver';
import { FeatureContext, LanguageFeature } from '../feature';
import { Language } from '../language';
import * as range from '../range';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import { BIBTEX_FIELDS } from './constants';
import * as factory from './factory';

type CompletionProvider = LanguageFeature<CompletionParams, CompletionItem[]>;

class BibtexFieldNameProvider implements CompletionProvider {
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

export const CompletionProvider = LanguageFeature.concat(
  new BibtexFieldNameProvider(),
);
