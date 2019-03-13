import { Position } from 'vscode-languageserver';
import { Document } from '../document';
import { Language } from '../language';
import * as range from '../range';
import { BibtexSyntaxKind } from '../syntax/bibtex/ast';
import {
  LatexCommandSyntax,
  LatexSyntaxKind,
  LatexSyntaxNode,
} from '../syntax/latex/ast';
import { CompletionProvider } from './provider';

type OrderByQualityCompletionProviderFactory = (
  provider: CompletionProvider,
) => CompletionProvider;

export const OrderByQualityCompletionProvider: OrderByQualityCompletionProviderFactory = provider => ({
  execute: async (context, cancellationToken) => {
    const { document, params } = context;
    const query = getQuery(document, params.position);
    const items = await provider.execute(context, cancellationToken);
    return items.sort(
      (x, y) => getQuality(query, y.label) - getQuality(query, x.label),
    );
  },
});

function getQuery(document: Document, position: Position): string | undefined {
  switch (document.tree.language) {
    case Language.Latex: {
      let node: LatexSyntaxNode | undefined = document.tree.descendants
        .filter(LatexCommandSyntax.is)
        .reverse()
        .find(x => range.contains(x.name.range, position));
      if (node === undefined) {
        node = document.tree.find(position);
      }
      if (node === undefined) {
        return undefined;
      }
      switch (node.kind) {
        case LatexSyntaxKind.Document:
          return undefined;
        case LatexSyntaxKind.Group:
          return '';
        case LatexSyntaxKind.Command:
          return node.name.text.substring(1);
        case LatexSyntaxKind.Text:
          return node.words[node.words.length - 1].text;
      }
      break;
    }
    case Language.Bibtex: {
      const node = document.tree.find(position);
      if (node === undefined) {
        return undefined;
      }
      switch (node.kind) {
        case BibtexSyntaxKind.Document:
          return '';
        case BibtexSyntaxKind.Preamble:
        case BibtexSyntaxKind.String:
        case BibtexSyntaxKind.Entry:
          return range.contains(node.type.range, position)
            ? node.type.text.substring(1)
            : '';
        case BibtexSyntaxKind.Comment:
          return node.token.text;
        case BibtexSyntaxKind.Field:
          return range.contains(node.name.range, position)
            ? node.name.text
            : '';
        case BibtexSyntaxKind.Word:
          return node.token.text;
        case BibtexSyntaxKind.Command:
          return node.token.text.substring(1);
        case BibtexSyntaxKind.QuotedContent:
        case BibtexSyntaxKind.BracedContent:
        case BibtexSyntaxKind.Concat:
          return '';
      }
      break;
    }
  }
}

function getQuality(query: string | undefined, label: string): number {
  if (query === undefined) {
    return 0;
  }

  if (label === query) {
    return 7;
  }

  if (label.toLowerCase() === query.toLowerCase()) {
    return 6;
  }

  if (label.startsWith(query)) {
    return 5;
  }

  if (label.toLowerCase().startsWith(query.toLowerCase())) {
    return 4;
  }

  if (label.includes(query)) {
    return 3;
  }

  if (label.toLowerCase().includes(query.toLowerCase())) {
    return 2;
  }

  return 1;
}
