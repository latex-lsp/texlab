import { TextDocumentPositionParams } from 'vscode-languageserver';
import { Document } from '../document';
import { BibtexFormatter } from '../formatting/bibtex';
import { Language } from '../language';
import { generateBibliography } from '../metadata/bibtexEntry';
import * as range from '../range';
import { HoverProvider } from './provider';

export const LatexCitationHoverProvider: HoverProvider = {
  execute: async ({ document, relatedDocuments, params }) => {
    const name = getEntryName(document, params);
    if (name === undefined) {
      return undefined;
    }

    const entry = getEntry(relatedDocuments, name);
    if (entry === undefined) {
      return undefined;
    }

    const formatter = new BibtexFormatter(true, 4, -1);
    const contents = generateBibliography(formatter.format(entry));
    return contents === undefined ? undefined : { contents };
  },
};

function getEntry(relatedDocuments: Document[], name: string) {
  for (const { tree } of relatedDocuments) {
    if (tree.language === Language.Bibtex) {
      for (const entry of tree.entries) {
        if (entry.name !== undefined && entry.name.text === name) {
          return entry;
        }
      }
    }
  }
  return undefined;
}

function getEntryName(
  { tree }: Document,
  { position }: TextDocumentPositionParams,
): string | undefined {
  switch (tree.language) {
    case Language.Latex:
      for (const { command, name } of tree.citations) {
        if (range.contains(command.range, position)) {
          return name.text;
        }
      }
      break;
    case Language.Bibtex:
      for (const { name } of tree.entries) {
        if (name !== undefined && range.contains(name.range, position)) {
          return name.text;
        }
      }
      break;
  }
  return undefined;
}
