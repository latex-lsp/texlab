import {
  CancellationToken,
  CompletionItem,
  Position,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { Language } from '../../language';
import { FeatureContext } from '../../provider';
import * as range from '../../range';
import {
  LatexCommandSyntax,
  LatexSyntaxKind,
  LatexSyntaxNode,
} from '../../syntax/latex/ast';
import { CompletionProvider } from '../provider';

export interface LatexArgumentCompletionProvider {
  commandNames: string[];
  argumentIndex: number;
  execute(
    context: FeatureContext<TextDocumentPositionParams>,
    command: LatexCommandSyntax,
    cancellationToken?: CancellationToken,
  ): Promise<CompletionItem[]>;
}

type LatexArgumentCompletionProviderFactory = (
  provider: LatexArgumentCompletionProvider,
) => CompletionProvider;

export const LatexArgumentCompletionProvider: LatexArgumentCompletionProviderFactory = provider => ({
  execute: async (context, cancellationToken) => {
    const { document, params } = context;
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    const nodes = document.tree.descendants
      .filter(x => range.contains(x.range, params.position))
      .reverse();

    const command =
      findNonEmptyCommand(provider, nodes, params.position) ||
      findEmptyCommand(provider, nodes, params.position);

    if (command !== undefined) {
      return provider.execute(context, command, cancellationToken);
    }

    return [];
  },
});

function findNonEmptyCommand(
  provider: LatexArgumentCompletionProvider,
  nodes: LatexSyntaxNode[],
  position: Position,
): LatexCommandSyntax | undefined {
  return nodes.length >= 3 && nodes[0].kind === LatexSyntaxKind.Text
    ? findCommand(provider, nodes, 1, position)
    : undefined;
}

function findEmptyCommand(
  provider: LatexArgumentCompletionProvider,
  nodes: LatexSyntaxNode[],
  position: Position,
): LatexCommandSyntax | undefined {
  return nodes.length >= 2
    ? findCommand(provider, nodes, 0, position)
    : undefined;
}

function findCommand(
  { commandNames, argumentIndex }: LatexArgumentCompletionProvider,
  nodes: LatexSyntaxNode[],
  index: number,
  position: Position,
): LatexCommandSyntax | undefined {
  const group = nodes[index];
  const command = nodes[index + 1];
  if (
    group.kind === LatexSyntaxKind.Group &&
    command.kind === LatexSyntaxKind.Command &&
    commandNames.includes(command.name.text) &&
    command.args.length > argumentIndex &&
    command.args[argumentIndex] === group &&
    range.containsExclusive(group.range, position)
  ) {
    return command;
  }

  return undefined;
}
