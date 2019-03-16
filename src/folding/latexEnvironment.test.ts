import { FoldingRange, FoldingRangeKind } from 'vscode-languageserver';
import { runSingleFile } from '../workspaceBuilder';
import { LatexEnvironmentFoldingProvider } from './latexEnvironment';

describe('LatexEnvironmentFoldingProvider', () => {
  const provider = LatexEnvironmentFoldingProvider;

  it('should provide foldings for multiline environments', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\begin{foo}\n\\end{foo}',
      line: 0,
      character: 0,
    });

    const expected: FoldingRange[] = [
      {
        startLine: 0,
        startCharacter: 11,
        endLine: 1,
        endCharacter: 0,
        kind: FoldingRangeKind.Region,
      },
    ];
    expect(actual).toEqual(expected);
  });

  it('should not provide foldings for BibTeX documents', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });
});
