import { runSingleFile } from '../../workspaceBuilder';
import { LatexTikzLibraryCompletionProvider, LIBRARIES } from './tikzLibrary';

describe('LatexTikzLibraryCompletionProvider', () => {
  const provider = LatexTikzLibraryCompletionProvider;

  it('should provide completion inside \\usetikzlibrary{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\usetikzlibrary{}',
      line: 0,
      character: 16,
    });
    expect(items.map(x => x.label)).toEqual(LIBRARIES);
  });
});
