import { Language } from './language';
import { BibtexSyntaxTree } from './syntax/bibtex/analysis';
import { LatexSyntaxTree } from './syntax/latex/analysis';
import { Uri } from './uri';

export interface Document {
  uri: Uri;
  tree: LatexSyntaxTree | BibtexSyntaxTree;
}

export abstract class Document {
  public static create(uri: Uri, text: string, language: Language): Document {
    let tree: LatexSyntaxTree | BibtexSyntaxTree;
    switch (language) {
      case Language.Latex:
        tree = new LatexSyntaxTree(text);
        break;
      case Language.Bibtex:
        tree = new BibtexSyntaxTree(text);
        break;
      default:
        throw Error('Unexpected language type: ' + language);
    }

    return { uri, tree };
  }
}
