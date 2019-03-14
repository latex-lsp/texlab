import { concat } from '../provider';
import { BibtexEntryDefinitionProvider } from './bibtexEntry';
import { DefinitionProvider } from './provider';

export const definitonProvider: DefinitionProvider = concat(
  BibtexEntryDefinitionProvider,
);
