import { CancellationToken } from 'vscode-jsonrpc';
import {
  CompletionItem,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { FeatureContext } from '../../provider';
import { ENVIRONMENT_COMMANDS } from '../../syntax/latex/analysis';
import { LatexCommandSyntax } from '../../syntax/latex/ast';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

export interface LatexEnvironmentCompletionProvider {
  execute(
    context: FeatureContext<TextDocumentPositionParams>,
    command: LatexCommandSyntax,
    cancellationToken?: CancellationToken,
  ): Promise<CompletionItem[]>;
}

type LatexEnvironmentCompletionProviderFactory = (
  provider: LatexEnvironmentCompletionProvider,
) => CompletionProvider;

export const LatexEnvironmentCompletionProvider: LatexEnvironmentCompletionProviderFactory = provider =>
  LatexArgumentCompletionProvider({
    commandNames: ENVIRONMENT_COMMANDS,
    argumentIndex: 0,
    execute: provider.execute,
  });
