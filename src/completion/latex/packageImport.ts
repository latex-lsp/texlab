import * as path from 'path';
import { TexResolver } from '../../resolver';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

type Factory = (resolver: TexResolver) => CompletionProvider;

export const LatexPackageImportCompletionProvider: Factory = resolver => {
  const items = [...resolver.filesByName.values()]
    .filter(x => path.extname(x) === '.sty')
    .map(x => factory.createPackage(path.basename(x, '.sty')));

  return LatexArgumentCompletionProvider({
    commandNames: ['\\usepackage'],
    argumentIndex: 0,
    execute: async () => items,
  });
};
