import { Language } from '../language';
import * as range from '../range';
import { RenameProvider } from './provider';

export const LatexEnvironmentRenameProvider: RenameProvider = {
  execute: async ({ uri, document, params }) => {
    if (document.tree.language !== Language.Latex) {
      return undefined;
    }

    for (const environment of document.tree.environments) {
      const begin = range.contains(environment.beginNameRange, params.position);
      const end = range.contains(environment.endNameRange, params.position);
      if (begin || end) {
        return {
          changes: {
            [uri.toString()]: [
              { range: environment.beginNameRange, newText: params.newName },
              { range: environment.endNameRange, newText: params.newName },
            ],
          },
        };
      }
    }
    return undefined;
  },
};
