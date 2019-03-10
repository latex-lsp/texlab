import { Language } from '../../language';

export class BibtexSyntaxTree {
  public readonly language: Language.Bibtex;

  constructor(public readonly text: string) {
    this.language = Language.Bibtex;
  }
}
