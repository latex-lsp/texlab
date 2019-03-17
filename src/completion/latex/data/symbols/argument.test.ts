import { runSingleFile } from '../../../../workspaceBuilder';
import { LatexArgumentSymbolCompletionProvider } from './argument';

describe('LatexArgumentSymbolCompletionProvider', () => {
  const provider = LatexArgumentSymbolCompletionProvider;

  it('should provide completion for \\mathbb', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\mathbb{}',
      line: 0,
      character: 8,
    });
    expect(items.map(x => x.label).sort()).toEqual([
      'A',
      'B',
      'C',
      'D',
      'E',
      'F',
      'G',
      'H',
      'I',
      'J',
      'K',
      'L',
      'M',
      'N',
      'O',
      'P',
      'Q',
      'R',
      'S',
      'T',
      'U',
      'V',
      'W',
      'X',
      'Y',
      'Z',
    ]);
  });
});
