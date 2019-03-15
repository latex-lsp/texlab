import { CompletionItem } from 'vscode-languageserver';
import { Language } from '../../language';
import { LatexCommandSyntax } from '../../syntax/latex/ast';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexCommandCompletionProvider } from './command';

export const LatexUserCommandCompletionProvider: CompletionProvider = LatexCommandCompletionProvider(
  {
    execute: async (context, command) => {
      const items: CompletionItem[] = [];
      context.relatedDocuments.forEach(document => {
        if (document.tree.language === Language.Latex) {
          document.tree.descendants
            .filter(LatexCommandSyntax.is)
            .filter(x => x !== command)
            .forEach(x => items.push(createItem(x)));
        }
      });
      return items;
    },
  },
);

function createItem(command: LatexCommandSyntax): CompletionItem {
  const name = command.name.text.substring(1);
  return factory.createCommand(name, factory.USER_COMPONENT);
}
