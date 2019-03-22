import { Location, Range } from 'vscode-languageserver';
import { runSingleFile, WorkspaceBuilder } from '../workspaceBuilder';
import { LatexLabelReferenceProvider } from './latexLabel';

describe('LatexLabelReferenceProvider', () => {
  const provider = LatexLabelReferenceProvider;

  it('it should find labels in related documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.tex', '\\label{foo}');
    const uri2 = builder.document('bar.tex', '\\input{foo.tex}\n\\ref{foo}');
    builder.document('baz.tex', '\\ref{foo}');
    const context = builder.context(uri1, 0, 8);

    const actual = await provider.execute(context);

    const expected: Location[] = [
      {
        uri: uri2.toString(),
        range: Range.create(1, 0, 1, 9),
      },
    ];
    expect(actual).toEqual(expected);
  });

  it('should find no references if the main document language is BibTeX', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });

  it('should find no references if the document only consists of comments', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '%foo',
      line: 0,
      character: 1,
    });
    expect(actual).toEqual([]);
  });
});
