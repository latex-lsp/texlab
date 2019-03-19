import { concat } from '../provider';
import { BibtexEntryReferenceProvider } from './bibtexEntry';
import { LatexLabelReferenceProvider } from './latexLabel';
import { ReferenceProvider } from './provider';

export const referenceProvider: ReferenceProvider = concat(
  BibtexEntryReferenceProvider,
  LatexLabelReferenceProvider,
);
