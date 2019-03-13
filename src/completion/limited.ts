import { CompletionProvider } from './provider';

export const COMPLETION_LIMIT = 50;

type LimitedCompletionProviderFactory = (
  provider: CompletionProvider,
) => CompletionProvider;

export const LimitedCompletionProvider: LimitedCompletionProviderFactory = provider => ({
  execute: (context, cancellationToken) =>
    provider
      .execute(context, cancellationToken)
      .then(items => items.slice(0, COMPLETION_LIMIT)),
});
