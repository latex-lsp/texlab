import { Document } from './document';
import { Uri } from './uri';

export class Workspace {
  public readonly documents: Document[] = [];

  public resolveDocument(uri: Uri, relativePath: string): Document | undefined {
    return undefined; // TODO
  }

  public resolveLinkTargets(uri: Uri, relativePath: string): string[] {
    return []; // TODO
  }

  public relatedDocuments(uri: Uri): Document[] {
    return []; // TODO
  }

  public findParent(childUri: Uri): Document | undefined {
    return undefined; // TODO
  }
}
