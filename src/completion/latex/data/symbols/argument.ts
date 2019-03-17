import { concat } from '../../../../provider';
import * as factory from '../../../factory';
import { CompletionProvider } from '../../../provider';
import { LatexArgumentCompletionProvider } from '../../argument';
import { LatexArgumentSymbolGroup, SYMBOLS } from './database';

export const LatexArgumentSymbolCompletionProvider: CompletionProvider = concat(
  ...SYMBOLS.arguments.map(createProvider),
);

function createProvider(group: LatexArgumentSymbolGroup): CompletionProvider {
  const items = group.arguments.map(({ argument, image }) =>
    factory.createArgumentSymbol(argument, image),
  );

  return LatexArgumentCompletionProvider({
    commandNames: ['\\' + group.command],
    argumentIndex: group.index,
    execute: async () => items,
  });
}
