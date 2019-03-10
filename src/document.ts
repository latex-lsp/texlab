import { BibtexSyntaxTree } from './syntax/bibtex/analysis';
import { LatexSyntaxTree } from './syntax/latex/analysis';
import { Uri } from './uri';

export interface Document {
  uri: Uri;
  tree: LatexSyntaxTree | BibtexSyntaxTree;
}
