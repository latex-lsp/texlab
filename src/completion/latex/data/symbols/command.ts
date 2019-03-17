import { CompletionItem } from 'vscode-languageserver';
import * as factory from '../../../factory';
import { CompletionProvider } from '../../../provider';
import { LatexCommandCompletionProvider } from '../../command';
import { LatexComponentSource } from '../component';
import { SYMBOLS } from './database';

type Factory = (database: LatexComponentSource) => CompletionProvider;

export const LatexCommandSymbolCompletionProvider: Factory = database =>
  LatexCommandCompletionProvider({
    execute: async ({ relatedDocuments }) => {
      const items: CompletionItem[] = [];
      const components = database.relatedComponents(relatedDocuments);
      SYMBOLS.commands.forEach(({ command, component, image }) => {
        if (component === null) {
          items.push(factory.createCommandSymbol(command, undefined, image));
        } else if (components.some(x => x.fileNames.includes(component))) {
          items.push(factory.createCommandSymbol(command, component, image));
        }
      });
      return items;
    },
  });
