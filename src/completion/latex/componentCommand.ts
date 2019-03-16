import { CompletionItem } from 'vscode-languageserver';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexCommandCompletionProvider } from './command';
import { LatexComponentSource } from './data/component';

type Factory = (database: LatexComponentSource) => CompletionProvider;

export const LatexComponentCommandCompletionProvider: Factory = database =>
  LatexCommandCompletionProvider({
    execute: async ({ relatedDocuments }) => {
      const items: CompletionItem[] = [];
      for (const component of database.relatedComponents(relatedDocuments)) {
        component.commands.forEach(command => {
          const item = factory.createCommand(
            command,
            component.fileNames.join(', '),
          );
          items.push(item);
        });
      }

      return items;
    },
  });
