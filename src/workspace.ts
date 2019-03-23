import fs from 'fs';
import path from 'path';
import { walk } from 'walk';
import { Document } from './document';
import { getLanguageByExtension, Language } from './language';
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

  public async loadDirectory(uri: Uri) {
    if (!uri.isFile()) {
      return;
    }

    return new Promise(resolve => {
      const walker = walk(uri.fsPath);

      walker.on('file', async (root, { name }, next) => {
        if (EXTENSIONS.includes(path.extname(name))) {
          const fileUri = Uri.file(path.join(root, name));
          await this.loadFile(fileUri);
        }

        next();
      });

      walker.on('errors', (_root, _nodeStatsArray, next) => next());

      walker.on('end', () => {
        resolve();
      });
    });
  }

  public async loadFile(uri: Uri): Promise<Document | undefined> {
    if (!uri.isFile() || this.documents.some(x => x.uri.equals(uri))) {
      return undefined;
    }

    let text: string;
    try {
      text = (await fs.promises.readFile(uri.fsPath)).toString();
    } catch (error) {
      return undefined;
    }

    const language = getLanguageByExtension(path.extname(uri.fsPath));
    if (language === undefined) {
      return undefined;
    }

    const document = Document.create(uri, text, language);
    if (document !== undefined) {
      this.put(document);
    }

    return document;
  }

  public async loadIncludes() {
    for (const { tree, uri } of this.documents) {
      if (tree.language !== Language.Latex) {
        continue;
      }

      tree.includes
        .filter(x => this.resolveDocument(uri, x.path) !== undefined)
        .map(x => this.resolveLinkTargets(uri, x.path))
        .reduce<Uri[]>((acc, x) => acc.concat(x), [])
        .forEach(x => this.loadFile(x));
    }
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
        if (
          edges.some(x => x.document1 === document && x.document2 === current)
        ) {
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
