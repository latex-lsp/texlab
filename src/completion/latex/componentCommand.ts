import { CompletionItem } from 'vscode-languageserver';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexCommandCompletionProvider } from './command';
import { LatexComponentDatabase } from './data/component';

type Factory = (database: LatexComponentDatabase) => CompletionProvider;

export const LatexComponentCommandCompletionProvider: Factory = database =>
  LatexCommandCompletionProvider({
    execute: async ({ relatedDocuments }) => {
      const items: CompletionItem[] = [];
      for (const component of database.relatedComponents(relatedDocuments)) {
        component.commands.forEach(x => {
          const item = factory.createCommand(x, component.fileNames.join());
          items.push(item);
        });
      }

      return items;
    },
  });
