import * as path from 'path';
import { getLanguageByExtension, Language } from './language';
import { BibtexSyntaxTree } from './syntax/bibtex/analysis';
import { LatexSyntaxTree } from './syntax/latex/analysis';
import { Uri } from './uri';
import { Workspace } from './workspace';

export class WorkspaceBuilder {
  public readonly workspace: Workspace;

  constructor() {
    this.workspace = new Workspace();
  }

  public document(file: string, text: string): Uri {
    const uri = Uri.file(path.resolve(file));
    const language = getLanguageByExtension(path.extname(file));
    let tree: LatexSyntaxTree | BibtexSyntaxTree;
    switch (language) {
      case Language.Bibtex:
        tree = new BibtexSyntaxTree(text);
        break;
      default:
        tree = new LatexSyntaxTree(text);
        break;
    }
    this.workspace.put({ uri, tree });
    return uri;
  }
}
