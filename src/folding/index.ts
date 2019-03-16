import { concat } from '../provider';
import { BibtexDeclarationFoldingProvider } from './bibtexDeclaration';
import { FoldingProvider } from './provider';

export const foldingProvider: FoldingProvider = concat(
  BibtexDeclarationFoldingProvider,
);
