import { Language } from './language';
import { Uri } from './uri';

export interface LatexDocument {
  language: Language.Latex;
  uri: Uri;
  text: string;
  tree: any; // TODO
}

export interface BibtexDocument {
  language: Language.Bibtex;
  uri: Uri;
  text: string;
  tree: any; // TODO
}

export type Document = LatexDocument | BibtexDocument;
