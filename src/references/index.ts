import { concat } from '../provider';
import { LatexLabelReferenceProvider } from './latexLabel';
import { ReferenceProvider } from './provider';

export const referenceProvider: ReferenceProvider = concat(
  LatexLabelReferenceProvider,
);
