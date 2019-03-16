import { concat } from '../provider';
import { BibtexDeclarationFoldingProvider } from './bibtexDeclaration';
import { LatexEnvironmentFoldingProvider } from './latexEnvironment';
import { FoldingProvider } from './provider';

export const foldingProvider: FoldingProvider = concat(
  BibtexDeclarationFoldingProvider,
  LatexEnvironmentFoldingProvider,
);
