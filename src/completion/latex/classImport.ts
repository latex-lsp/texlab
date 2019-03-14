import * as path from 'path';
import { TexResolver } from '../../resolver';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

type LatexClassImportCompletionProviderFactory = (
  resolver: TexResolver,
) => CompletionProvider;

export const LatexClassImportCompletionProvider: LatexClassImportCompletionProviderFactory = resolver => {
  const items = [...resolver.filesByName.values()]
    .filter(x => path.extname(x) === '.cls')
    .map(x => factory.createClass(path.basename(x, '.cls')));

  return LatexArgumentCompletionProvider({
    commandNames: ['\\documentclass'],
    argumentIndex: 0,
    execute: async () => items,
  });
};
