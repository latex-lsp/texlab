import { Language } from '../../language';
import { BibtexSyntaxKind } from '../../syntax/bibtex/ast';
import * as factory from '../factory';
import { KERNEL_COMMANDS } from '../kernel';
import { CompletionProvider } from '../provider';

const ITEMS = KERNEL_COMMANDS.map(x => factory.createCommand(x, undefined));

export const BibtexKernelCommandCompletionProvider: CompletionProvider = {
  execute: async context => {
    const { document, params } = context;
    if (document.tree.language !== Language.Bibtex) {
      return [];
    }

    const command = document.tree.find(params.position);
    return command !== undefined &&
      command.kind === BibtexSyntaxKind.Command &&
      command.token.character !== params.position.character
      ? ITEMS
      : [];
  },
};
