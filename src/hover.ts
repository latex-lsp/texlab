import {
  Hover,
  MarkupKind,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { Language } from './language';
import { getTypeDocumentation } from './metadata/bibtexType';
import { choice, FeatureContext, FeatureProvider } from './provider';
import * as range from './range';

export type HoverProvider = FeatureProvider<
  TextDocumentPositionParams,
  Hover | undefined
>;

export class BibtexEntryTypeHoverProvider implements HoverProvider {
  public async execute(
    context: FeatureContext<TextDocumentPositionParams>,
  ): Promise<Hover | undefined> {
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
  }
}

export const hoverProvider = choice(new BibtexEntryTypeHoverProvider());
