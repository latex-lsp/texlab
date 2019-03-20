import { CompletionItem } from 'vscode-languageserver';
import { Language } from '../../language';
import { CITATION_COMMANDS } from '../../syntax/latex/analysis';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

export const LatexCitationCompletionProvider: CompletionProvider = LatexArgumentCompletionProvider(
  {
    commandNames: CITATION_COMMANDS,
    argumentIndex: 0,
    execute: async ({ relatedDocuments }) => {
      const items: CompletionItem[] = [];
      relatedDocuments.forEach(({ tree }) => {
        if (tree.language === Language.Bibtex) {
          tree.entries.forEach(entry => {
            if (entry.name !== undefined) {
              items.push(factory.createCitation(entry));
            }
          });
        }
      });
      return items;
    },
  },
);
