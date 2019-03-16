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
      text: '\\definecolorset{rgb}',
      line: 0,
      character: 16,
    });
    expect(items.map(x => x.label)).toEqual(COLOR_MODELS);
  });

  it('should not provide completion for BibTeX documents', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '',
      line: 0,
      character: 0,
    });
    expect(items).toEqual([]);
  });
});
