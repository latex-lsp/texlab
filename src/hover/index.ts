import { LatexComponentSource } from '../completion/latex/data/component';
import { choice, deferred } from '../provider';
import { BibtexEntryTypeHoverProvider } from './bibtexEntryType';
import { BibtexFieldHoverProvider } from './bibtexField';
import { LatexCitationHoverProvider } from './latexCitation';
import { LatexCommandHoverProvider } from './latexCommand';
import { LatexComponentHoverProvider } from './latexComponent';
import { LatexMathHoverProvider } from './latexMath';
import { HoverProvider as Provider } from './provider';

type Factory = (database: Promise<LatexComponentSource>) => Provider;

export const HoverProvider: Factory = componentSource =>
  choice(
    BibtexEntryTypeHoverProvider,
    BibtexFieldHoverProvider,
    LatexCitationHoverProvider,
    LatexComponentHoverProvider,
    LatexMathHoverProvider,
    deferred(LatexCommandHoverProvider, componentSource, undefined),
  );
