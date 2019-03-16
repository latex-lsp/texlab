import { FoldingRange, FoldingRangeKind } from 'vscode-languageserver';
import { runSingleFile } from '../workspaceBuilder';
import { BibtexDeclarationFoldingProvider } from './bibtexDeclaration';

describe('BibtexDeclarationFoldingProvider', () => {
  const provider = BibtexDeclarationFoldingProvider;

  it('should provide foldings for entries', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = baz\n}',
      line: 0,
      character: 0,
    });

    const expected: FoldingRange[] = [
      {
        startLine: 0,
        startCharacter: 0,
        endLine: 1,
        endCharacter: 0,
        kind: FoldingRangeKind.Region,
      },
    ];
    expect(actual).toEqual(expected);
  });

  it('should provide foldings for strings', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@string{foo = "bar"}',
      line: 0,
      character: 0,
    });

    const expected: FoldingRange[] = [
      {
        startLine: 0,
        startCharacter: 0,
        endLine: 0,
        endCharacter: 19,
        kind: FoldingRangeKind.Region,
      },
    ];
    expect(actual).toEqual(expected);
  });

  it('should not provide foldings for LaTeX documents', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });

  it('should not provide foldings for invalid entries', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });
});
