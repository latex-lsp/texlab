import { Language } from './language';
import { BibtexSyntaxTree } from './syntax/bibtex/analysis';
import { LatexSyntaxTree } from './syntax/latex/analysis';
import { Uri } from './uri';

export interface LatexDocument {
  uri: Uri;
  text: string;
  tree: LatexSyntaxTree;
}

export interface BibtexDocument {
  uri: Uri;
  text: string;
  tree: BibtexSyntaxTree;
}

export type Document = LatexDocument | BibtexDocument;
