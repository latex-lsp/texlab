import { FoldingRange, FoldingRangeKind } from 'vscode-languageserver';
import { runSingleFile } from '../workspaceBuilder';
import { LatexSectionFoldingProvider } from './latexSection';

describe('LatexSectionFoldingProvider', () => {
  const provider = LatexSectionFoldingProvider;

  it('should provide foldings for sections with multiple levels of nesting', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text:
        '\\section{Foo}\nfoo\n\\subsection{Bar}\nbar\n\\section{Baz}\nbaz\n\\section{Qux}',
      line: 0,
      character: 0,
    });

    const expected: FoldingRange[] = [
      {
        startLine: 0,
        startCharacter: 13,
        endLine: 3,
        endCharacter: 0,
        kind: FoldingRangeKind.Region,
      },
      {
        startLine: 2,
        startCharacter: 16,
        endLine: 3,
        endCharacter: 0,
        kind: FoldingRangeKind.Region,
      },
      {
        startLine: 4,
        startCharacter: 13,
        endLine: 5,
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
