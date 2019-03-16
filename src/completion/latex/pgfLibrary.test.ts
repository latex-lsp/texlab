import { runSingleFile } from '../../workspaceBuilder';
import { LatexPgfLibraryCompletionProvider, LIBRARIES } from './pgfLibrary';

describe('LatexPgfLibraryCompletionProvider', () => {
  const provider = LatexPgfLibraryCompletionProvider;

  it('should provide completion inside \\usepgflibrary{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\usepgflibrary{}',
      line: 0,
      character: 15,
    });
    expect(items.map(x => x.label)).toEqual(LIBRARIES);
  });
});
