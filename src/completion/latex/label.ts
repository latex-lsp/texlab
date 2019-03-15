import { CompletionItem } from 'vscode-languageserver';
import { Language } from '../../language';
import { LABEL_REFERENCE_COMMANDS } from '../../syntax/latex/analysis';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

export const LatexLabelCompletionProvider: CompletionProvider = LatexArgumentCompletionProvider(
  {
    commandNames: LABEL_REFERENCE_COMMANDS,
    argumentIndex: 0,
    execute: async context => {
      const items: CompletionItem[] = [];
      context.relatedDocuments.forEach(document => {
        if (document.tree.language === Language.Latex) {
          document.tree.labelDefinitions.forEach(label => {
            items.push(factory.createLabel(label.name.text));
          });
        }
      });
      return items;
    },
  },
);
