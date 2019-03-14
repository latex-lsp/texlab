import { Language } from '../language';
import * as range from '../range';
import { DefinitionProvider } from './provider';

export const LatexLabelDefinitionProvider: DefinitionProvider = {
  execute: async context => {
    const { relatedDocuments, params } = context;
    if (context.document.tree.language !== Language.Latex) {
      return [];
    }

    const reference = [
      ...context.document.tree.labelDefinitions,
      ...context.document.tree.labelReferences,
    ].find(x => range.contains(x.name.range, params.position));
    if (reference === undefined) {
      return [];
    }

    for (const document of relatedDocuments) {
      if (document.tree.language === Language.Latex) {
        for (const definition of document.tree.labelDefinitions) {
          if (definition.name.text === reference.name.text) {
            return [
              {
                uri: document.uri.toString(),
                range: definition.name.range,
              },
            ];
          }
        }
      }
    }
    return [];
  },
};
