import { choice } from '../provider';
import { BibtexEntryTypeHoverProvider } from './bibtexEntryType';
import { BibtexFieldHoverProvider } from './bibtexField';
import { LatexComponentHoverProvider } from './latexComponent';

export const hoverProvider = choice(
  BibtexEntryTypeHoverProvider,
  BibtexFieldHoverProvider,
  LatexComponentHoverProvider,
);
