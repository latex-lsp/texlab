import { runSingleFile } from '../../workspaceBuilder';
import { COLORS, LatexColorCompletionProvider } from './color';

describe('LatexColorCompletionProvider', () => {
  it('should provide completion inside \\color{}', async () => {
    const items = await runSingleFile({
      provider: LatexColorCompletionProvider,
      file: 'foo.tex',
      text: '\\color{}',
      line: 0,
      character: 7,
    });
    expect(items.map(x => x.label)).toEqual(COLORS);
  });
});
