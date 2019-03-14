import { concat } from '../provider';
import { BibtexEntryDefinitionProvider } from './bibtexEntry';
import { LatexLabelDefinitionProvider } from './label';
import { DefinitionProvider } from './provider';

export const definitonProvider: DefinitionProvider = concat(
  BibtexEntryDefinitionProvider,
  LatexLabelDefinitionProvider,
);
