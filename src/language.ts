export enum Language {
  Latex,
  Bibtex,
}

export function getLanguageById(id: string): Language | undefined {
  switch (id) {
    case 'latex':
      return Language.Latex;
    case 'bibtex':
      return Language.Bibtex;
    default:
      return undefined;
  }
}

export function getLanguageByExtension(
  extension: string,
): Language | undefined {
  switch (extension.toLowerCase()) {
    case '.tex':
    case '.sty':
    case '.cls':
      return Language.Latex;
    case '.bib':
      return Language.Bibtex;
    default:
      return undefined;
  }
}
