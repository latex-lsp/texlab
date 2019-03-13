import { CompletionItem } from 'vscode-languageserver';
import { runSingleFile } from '../workspaceBuilder';
import { COMPLETION_LIMIT, LimitedCompletionProvider } from './limited';

describe('LimitedCompletionProvider', () => {
  it('should return at most COMPLETION_LIMIT items', async () => {
    const provider = LimitedCompletionProvider({
      execute: async () => {
        const items: CompletionItem[] = [];
        for (let i = 0; i <= COMPLETION_LIMIT; i++) {
          items.push({ label: i.toString() });
        }
        return items;
      },
    });

    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '',
      line: 0,
      character: 0,
    });

    expect(actual).toHaveLength(COMPLETION_LIMIT);
  });
});
