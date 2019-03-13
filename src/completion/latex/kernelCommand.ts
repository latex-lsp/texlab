import * as factory from '../factory';
import { KERNEL_COMMANDS } from '../kernel';
import { CompletionProvider } from '../provider';
import { LatexCommandCompletionProvider } from './command';

const ITEMS = KERNEL_COMMANDS.map(x => factory.createCommand(x, undefined));

export const LatexKernelCommandProvider: CompletionProvider = LatexCommandCompletionProvider(
  {
    execute: async () => ITEMS,
  },
);
