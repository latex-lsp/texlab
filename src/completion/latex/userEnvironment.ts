import { CompletionItem } from 'vscode-languageserver';
import { Language } from '../../language';
import * as range from '../../range';
import { LatexSyntaxTree } from '../../syntax/latex/analysis';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexEnvironmentCompletionProvider } from './environment';

export const LatexUserEnvironmentCompletionProvider: CompletionProvider = LatexEnvironmentCompletionProvider(
  {
    execute: async context => {
      const { relatedDocuments, params } = context;
      const tree = context.document.tree as LatexSyntaxTree;
      const current = tree.environments.find(
        environment =>
          range.contains(environment.beginNameRange, params.position) ||
          range.contains(environment.endNameRange, params.position),
      );

      const items: CompletionItem[] = [];
      relatedDocuments.forEach(document => {
        if (document.tree.language === Language.Latex) {
          document.tree.environments
            .filter(x => x !== current)
            .forEach(environment => {
              if (environment.beginName !== '') {
                items.push(
                  factory.createEnvironment(
                    environment.beginName,
                    factory.USER_COMPONENT,
                  ),
                );
              }

              if (environment.endName !== '') {
                items.push(
                  factory.createEnvironment(
                    environment.endName,
                    factory.USER_COMPONENT,
                  ),
                );
              }
            });
        }
      });
      return items;
    },
  },
);
