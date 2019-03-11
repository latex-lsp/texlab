import { Language } from '../../language';
import {
  BibtexDocumentSyntax,
  BibtexEntrySyntax,
  BibtexSyntaxNode,
  descendants,
} from './ast';
import { parse } from './parser';

export class BibtexSyntaxTree {
  public readonly language: Language.Bibtex;
  public readonly root: BibtexDocumentSyntax;
  public readonly descendants: BibtexSyntaxNode[];
  public readonly entries: BibtexEntrySyntax[];

  constructor(public readonly text: string) {
    this.language = Language.Bibtex;
    this.root = parse(text);
    this.descendants = descendants(this.root);
    this.entries = this.root.children.filter(BibtexEntrySyntax.is);
  }
}
