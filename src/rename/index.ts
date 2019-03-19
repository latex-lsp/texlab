import { choice } from '../provider';
import { BibtexEntryRenameProvider } from './bibtexEntry';
import { RenameProvider } from './provider';

export const renameProvider: RenameProvider = choice(BibtexEntryRenameProvider);
