import { Range } from 'vscode-languageserver';
import { runSingleFile, WorkspaceBuilder } from '../workspaceBuilder';
import { BibtexEntryDefinitionProvider } from './bibtexEntry';

describe('BibtexEntryDefintionProvider', () => {
  const provider = BibtexEntryDefinitionProvider;

  it('should return the first entry that matches the key', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document(
      'foo.tex',
      '\\addbibresource{bar.bib}\n\\cite{bar}',
    );
    const uri2 = builder.document('bar.bib', '@article{foo,}\n@article{bar}');
    const context = builder.context(uri1, 1, 7);
    const actual = await provider.execute(context);
    expect(actual).toEqual([
      {
        uri: uri2.toString(),
        range: Range.create(1, 9, 1, 12),
      },
    ]);
  });

  it('should return an empty array if there is no matching entry', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\cite{foo}',
      line: 0,
      character: 7,
    });
    expect(actual).toEqual([]);
  });

  it('should return an empty array if the cursor is not inside of a citation', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 2,
    });
    expect(actual).toEqual([]);
  });

  it('should return an empty array if used inside of a BibTeX document', async () => {
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
