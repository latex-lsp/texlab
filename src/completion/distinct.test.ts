import { runSingleFile } from '../workspaceBuilder';
import { DistinctCompletionProvider } from './distinct';

describe('DistinctCompletionProvider', () => {
  it('should filter out duplicates', async () => {
    const provider = DistinctCompletionProvider({
      execute: async () => {
        return [
          { label: 'foo', data: 1 },
          { label: 'bar', data: 2 },
          { label: 'foo', data: 3 },
        ];
      },
    });

    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '',
      line: 0,
      character: 0,
    });

    expect(actual).toEqual([
      { label: 'foo', data: 1 },
      { label: 'bar', data: 2 },
    ]);
  });
});
