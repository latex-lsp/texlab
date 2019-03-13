import { CompletionProvider } from './provider';

type DistinctCompletionProviderFactory = (
  provider: CompletionProvider,
) => CompletionProvider;

export const DistinctCompletionProvider: DistinctCompletionProviderFactory = provider => ({
  execute: (context, cancellationToken) =>
    provider
      .execute(context, cancellationToken)
      .then(items => distinctBy(items, item => item.label)),
});

function distinctBy<T, S>(items: T[], selector: (item: T) => S) {
  const results: T[] = [];
  const keys = new Set();
  items.forEach(item => {
    const key = selector(item);
    if (!keys.has(key)) {
      keys.add(key);
      results.push(item);
    }
  });
  return results;
}
