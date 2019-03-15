import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexCommandCompletionProvider } from './command';

const SNIPPET = factory.createSnippet(
  'begin',
  undefined,
  'begin{$1}\n\t$0\n\\end{$1}',
);

export const LatexBeginCommandCompletionProvider: CompletionProvider = LatexCommandCompletionProvider(
  {
    execute: async () => [SNIPPET],
  },
);
