import { TexResolver } from '../../resolver';
import { runSingleFile } from '../../workspaceBuilder';
import { LatexPackageImportCompletionProvider } from './packageImport';

describe('LatexPackageImportCompletionProvider', () => {
  const resolver: TexResolver = {
    filesByName: new Map<string, string>([
      ['amsmath.sty', 'amsmath.sty'],
      ['lipsum.sty', 'lipsum.sty'],
    ]),
  };

  const provider = LatexPackageImportCompletionProvider(resolver);

  it('should provide completion inside \\usepackage{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\usepackage{}',
      line: 0,
      character: 12,
    });
    expect(items.map(x => x.label)).toEqual(['amsmath', 'lipsum']);
  });

  it('should not provide completion outside of \\usepackage{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\usepackage{}',
      line: 0,
      character: 10,
    });
    expect(items).toEqual([]);
  });
});
