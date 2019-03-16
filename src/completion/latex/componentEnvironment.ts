import { CompletionItem } from 'vscode-languageserver';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexComponentSource } from './data/component';
import { LatexEnvironmentCompletionProvider } from './environment';

type Factory = (database: LatexComponentSource) => CompletionProvider;

export const LatexComponentEnvironmentCompletionProvider: Factory = database =>
  LatexEnvironmentCompletionProvider({
    execute: async ({ relatedDocuments }) => {
      const items: CompletionItem[] = [];
      database.relatedComponents(relatedDocuments).forEach(component => {
        component.environments.forEach(environment => {
          const item = factory.createEnvironment(
            environment,
            component.fileNames.join(', '),
          );
          items.push(item);
        });
      });
      return items;
    },
  });
