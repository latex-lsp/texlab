import { choice } from '../provider';
import { BibtexEntryTypeHoverProvider } from './bibtexEntryType';
import { BibtexFieldHoverProvider } from './bibtexField';

export const hoverProvider = choice(
  BibtexEntryTypeHoverProvider,
  BibtexFieldHoverProvider,
);
