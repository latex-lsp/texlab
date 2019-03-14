import { DocumentLink, DocumentLinkParams } from 'vscode-languageserver';
import { Language } from './language';
import { FeatureProvider } from './provider';

type LinkProvider = FeatureProvider<DocumentLinkParams, DocumentLink[]>;

export const LatexIncludeLinkProvider: LinkProvider = {
  execute: async context => {
    const { uri, document, workspace } = context;
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    const links: DocumentLink[] = [];
    document.tree.includes.forEach(include => {
      const range = include.command.args[0].children[0].range;
      const target = workspace.resolveDocument(uri, include.path);
      if (target !== undefined) {
        links.push({
          range,
          target: target.uri.toString(),
        });
      }
    });
    return links;
  },
};

export const linkProvider: LinkProvider = LatexIncludeLinkProvider;
