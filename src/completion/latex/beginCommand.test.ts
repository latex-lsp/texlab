import { runSingleFile } from '../../workspaceBuilder';
import { LatexBeginCommandCompletionProvider } from './beginCommand';

describe('LatexBeginCommandCompletionProvider', () => {
  const provider = LatexBeginCommandCompletionProvider;

  it('should provide completion inside LaTeX commands', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\begin',
      line: 0,
      character: 2,
    });
    expect(items).toHaveLength(1);
  });
});
