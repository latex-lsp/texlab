import 'array-flat-polyfill';
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

export interface LanguageFeature<T, R> {
  execute(
    context: FeatureContext<T>,
    cancellationToken?: CancellationToken,
  ): Promise<R>;
}

export abstract class LanguageFeature<T, R> {
  public static concat<T, R>(
    ...features: Array<LanguageFeature<T, R[]>>
  ): LanguageFeature<T, R[]> {
    return {
      execute: async (context, cancellationToken) => {
        const results = await Promise.all(
          features.map(x => x.execute(context, cancellationToken)),
        );

        return results.flat();
      },
    };
  }

  public static choice<T, R>(
    ...features: Array<LanguageFeature<T, R | undefined>>
  ): LanguageFeature<T, R | undefined> {
    return {
      execute: async (context, cancellationToken) => {
        for (const feature of features) {
          const result = feature.execute(context, cancellationToken);
          if (result !== undefined) {
            return result;
          }
        }

        return undefined;
      },
    };
  }
}
