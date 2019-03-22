import { Location, Range } from 'vscode-languageserver';
import { runSingleFile, WorkspaceBuilder } from '../workspaceBuilder';
import { BibtexEntryReferenceProvider } from './bibtexEntry';

describe('BibtexEntryReferenceProvider', () => {
  const provider = BibtexEntryReferenceProvider;

  it('should find citations in related documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.bib', '@article{foo, bar = {baz}}');
    const uri2 = builder.document(
      'bar.tex',
      '\\addbibresource{foo.bib}\n\\cite{foo}',
    );
    builder.document('baz.tex', '\\cite{foo}');
    const context = builder.context(uri1, 0, 9);

    const actual = await provider.execute(context);

    const expected: Location[] = [
      {
        uri: uri2.toString(),
        range: Range.create(1, 0, 1, 10),
      },
    ];
    expect(actual).toEqual(expected);
  });

  it('should find no references if the main document language is LaTeX', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });

  it('should find no references if the document only consists of comments', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: 'foo',
      line: 0,
      character: 1,
    });
    expect(actual).toEqual([]);
  });
});
