import {
  CancellationToken,
  CompletionItem,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { Language } from '../../language';
import { FeatureContext } from '../../provider';
import { LatexCommandSyntax, LatexSyntaxKind } from '../../syntax/latex/ast';
import { CompletionProvider } from '../provider';

export interface LatexCommandCompletionProvider {
  execute(
    context: FeatureContext<TextDocumentPositionParams>,
    command: LatexCommandSyntax,
    cancellationToken?: CancellationToken,
  ): Promise<CompletionItem[]>;
}

type Factory = (provider: LatexCommandCompletionProvider) => CompletionProvider;

export const LatexCommandCompletionProvider: Factory = provider => ({
  execute: async (context, cancellationToken) => {
    const { document, params } = context;
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    const command = document.tree.find(params.position);
    return command !== undefined &&
      command.kind === LatexSyntaxKind.Command &&
      command.name.character !== params.position.character
      ? provider.execute(context, command, cancellationToken)
      : [];
  },
});
