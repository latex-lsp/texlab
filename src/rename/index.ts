import { choice } from '../provider';
import { BibtexEntryRenameProvider } from './bibtexEntry';
import { LatexEnvironmentRenameProvider } from './latexEnvironment';
import { LatexLabelRenameProvider } from './latexLabel';
import { RenameProvider } from './provider';

export const renameProvider: RenameProvider = choice(
  BibtexEntryRenameProvider,
  LatexEnvironmentRenameProvider,
  LatexLabelRenameProvider,
);
