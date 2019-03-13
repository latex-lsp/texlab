import { concat } from '../provider';
import { BibtexEntryTypeCompletionProvider } from './bibtexEntryType';
import { BibtexFieldNameCompletionProvider } from './bibtexFieldName';

export const completionProvider = concat(
  BibtexFieldNameCompletionProvider,
  BibtexEntryTypeCompletionProvider,
);
