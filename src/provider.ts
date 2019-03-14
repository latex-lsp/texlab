import { CancellationToken } from 'vscode-languageserver';
import { Document } from './document';
import { Uri } from './uri';
import { Workspace } from './workspace';

export class FeatureContext<T> {
  public readonly document: Document;
  public readonly relatedDocuments: Document[];

  constructor(
    public readonly uri: Uri,
    public readonly workspace: Workspace,
    public readonly params: T,
  ) {
    this.document = workspace.documents.find(x => x.uri.equals(uri))!;
    this.relatedDocuments = workspace.relatedDocuments(uri);
  }
}

export interface FeatureProvider<T, R> {
  execute(
    context: FeatureContext<T>,
    cancellationToken?: CancellationToken,
  ): Promise<R>;
}

export function concat<T, R>(
  ...providers: Array<FeatureProvider<T, R[]>>
): FeatureProvider<T, R[]> {
  return {
    execute: async (context, cancellationToken) => {
      const results = await Promise.all(
        providers.map(x => x.execute(context, cancellationToken)),
      );

      return new Array<R>().concat(...results);
    },
  };
}

export function choice<T, R>(
  ...providers: Array<FeatureProvider<T, R | undefined>>
): FeatureProvider<T, R | undefined> {
  return {
    execute: async (context, cancellationToken) => {
      for (const provider of providers) {
        const result = await provider.execute(context, cancellationToken);
        if (result !== undefined) {
          return result;
        }
      }

      return undefined;
    },
  };
}

export function deferred<S, T, R>(
  factory: (source: S) => FeatureProvider<T, R>,
  source: Promise<S>,
  defaultValue: R,
): FeatureProvider<T, R> {
  let provider: FeatureProvider<T, R>;
  source.then(x => (provider = factory(x)));

  return {
    execute: async (context, cancellationToken) => {
      return provider !== undefined
        ? provider.execute(context, cancellationToken)
        : defaultValue;
    },
  };
}
