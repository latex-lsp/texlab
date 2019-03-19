import { Position, TextEdit } from 'vscode-languageserver';
import { Language } from '../language';
import * as range from '../range';
import { BibtexSyntaxTree } from '../syntax/bibtex/analysis';
import { LatexSyntaxTree } from '../syntax/latex/analysis';
import { Token } from '../syntax/token';
import { RenameProvider } from './provider';

export const BibtexEntryRenameProvider: RenameProvider = {
  execute: async ({ document: mainDocument, relatedDocuments, params }) => {
    let token: Token | undefined;
    switch (mainDocument.tree.language) {
      case Language.Latex:
        token = findCitation(mainDocument.tree, params.position);
        break;
      case Language.Bibtex:
        token = findEntry(mainDocument.tree, params.position);
        break;
    }

    if (token === undefined) {
      return undefined;
    }

    const changes: { [uri: string]: TextEdit[] } = {};
    for (const { uri, tree } of relatedDocuments) {
      switch (tree.language) {
        case Language.Latex:
          changes[uri.toString()] = tree.citations
            .filter(x => x.name.text === token!.text)
            .map(x => ({ range: x.name.range, newText: params.newName }));
          break;
        case Language.Bibtex:
          changes[uri.toString()] = tree.entries
            .filter(x => x.name !== undefined && x.name.text === token!.text)
            .map(x => ({ range: x.name!.range, newText: params.newName }));
          break;
      }
    }
    return { changes };
  },
};

function findCitation(
  tree: LatexSyntaxTree,
  position: Position,
): Token | undefined {
  for (const citation of tree.citations) {
    if (range.contains(citation.name.range, position)) {
      return citation.name;
    }
  }
  return undefined;
}

function findEntry(
  tree: BibtexSyntaxTree,
  position: Position,
): Token | undefined {
  for (const entry of tree.entries) {
    if (
      entry.name !== undefined &&
      range.contains(entry.name.range, position)
    ) {
      return entry.name;
    }
  }
  return undefined;
}
