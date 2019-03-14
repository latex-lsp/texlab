import { TexResolver } from '../../resolver';
import { runSingleFile } from '../../workspaceBuilder';
import { LatexClassImportCompletionProvider } from './classImport';

describe('LatexClassImportCompletionProvider', () => {
  const resolver: TexResolver = {
    filesByName: new Map<string, string>([
      ['article.cls', 'article.cls'],
      ['book.cls', 'book.cls'],
    ]),
  };

  const provider = LatexClassImportCompletionProvider(resolver);

  it('should provide completion inside \\documentclass{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\documentclass{}',
      line: 0,
      character: 15,
    });

    expect(items.map(x => x.label)).toEqual(['article', 'book']);
  });

  it('should not provide completion outside of \\documentclass{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\documentclass{}',
      line: 0,
      character: 14,
    });

    expect(items).toEqual([]);
  });
});
