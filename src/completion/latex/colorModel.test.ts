import { runSingleFile } from '../../workspaceBuilder';
import { COLOR_MODELS, LatexColorModelCompletionProvider } from './colorModel';

describe('LatexColorModelCompletionProvider', () => {
  const provider = LatexColorModelCompletionProvider;

  it('should provide completion for \\definecolor', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\definecolor{foo}{}',
      line: 0,
      character: 18,
    });
    expect(items.map(x => x.label)).toEqual(COLOR_MODELS);
  });

  it('should provide completion for \\definecolorset', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\definecolorset{}',
      line: 0,
      character: 16,
    });
    expect(items.map(x => x.label)).toEqual(COLOR_MODELS);
  });
});
