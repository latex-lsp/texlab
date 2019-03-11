import * as path from 'path';
import { Document } from './document';
import { Language } from './language';
import { Uri } from './uri';

const EXTENSIONS = ['.tex', '.sty', '.cls', '.bib'];

export class Workspace {
  private _documents: Document[] = [];

  public get documents(): ReadonlyArray<Document> {
    return this._documents;
  }

  public put(document: Document) {
    this._documents = [
      ...this.documents.filter(x => !x.uri.equals(document.uri)),
      document,
    ];
  }

  public findParent(uri: Uri): Document | undefined {
    for (const document of this.relatedDocuments(uri)) {
      if (
        document.tree.language === Language.Latex &&
        document.tree.isStandalone
      ) {
        return document;
      }
    }
    return this.documents.find(x => x.uri.equals(uri));
  }

  public relatedDocuments(uri: Uri): Document[] {
    interface Edge {
      document1: Document;
      document2: Document;
    }

    const edges: Edge[] = [];
    this.documents.forEach(parent => {
      if (parent.uri.isFile() && parent.tree.language === Language.Latex) {
        parent.tree.includes.forEach(include => {
          const child = this.resolveDocument(parent.uri, include.path);
          if (child !== undefined) {
            edges.push({ document1: parent, document2: child });
            edges.push({ document1: child, document2: parent });
          }
        });
      }
    });

    const results: Document[] = [];
    const start = this.documents.find(x => x.uri.equals(uri));
    if (start === undefined) {
      return [];
    }

    const visited: Document[] = [];
    const stack = [start];
    while (stack.length > 0) {
      const current = stack.pop()!;
      if (visited.some(x => x.uri.equals(current.uri))) {
        continue;
      }
      visited.push(current);

      results.push(current);
      this.documents.forEach(document => {
        if (edges.some(x => x.document1 === document)) {
          stack.push(document);
        }
      });
    }
    return results;
  }

  public resolveDocument(uri: Uri, relativePath: string): Document | undefined {
    for (const target of this.resolveLinkTargets(uri, relativePath)) {
      const document = this.documents.find(x => x.uri.equals(target));
      if (document !== undefined && document.uri.isFile()) {
        return document;
      }
    }
    return undefined;
  }

  public resolveLinkTargets(uri: Uri, relativePath: string): Uri[] {
    if (!uri.isFile()) {
      return [];
    }

    const targets: string[] = [];
    const basePath = path.dirname(uri.fsPath);
    const fullPath = path.normalize(path.resolve(basePath, relativePath));
    targets.push(fullPath);
    EXTENSIONS.forEach(extension => targets.push(fullPath + extension));
    return targets.map(x => Uri.file(x));
  }
}
