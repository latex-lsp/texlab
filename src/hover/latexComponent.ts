import { Language } from '../language';
import { getComponentMetadata } from '../metadata/component';
import * as range from '../range';
import { HoverProvider } from './provider';

export const LatexComponentHoverProvider: HoverProvider = {
  execute: async context => {
    const { document, params } = context;
    if (document.tree.language !== Language.Latex) {
      return undefined;
    }

    const include = document.tree.includes
      .filter(x => x.isUnitImport)
      .find(x => range.contains(x.command.range, params.position));

    if (include === undefined) {
      return undefined;
    }

    const metadata = await getComponentMetadata(include.path);
    if (metadata === undefined) {
      return undefined;
    }

    return {
      contents: metadata.documentation,
    };
  },
};
