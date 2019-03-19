import { Range, WorkspaceEdit } from 'vscode-languageserver';
import { WorkspaceBuilder } from '../workspaceBuilder';
import { BibtexEntryRenameProvider } from './bibtexEntry';

describe('BibtexEntryRenameProvider', () => {
  it('should be able to rename an entry', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.bib', '@article{foo,}');
    const uri2 = builder.document(
      'bar.tex',
      '\\addbibresource{foo.bib}\n\\cite{foo}',
    );
    const context = builder.context(uri1, 0, 9, 'bar');
    const actual = await BibtexEntryRenameProvider.execute(context);
    const expected: WorkspaceEdit = {
      changes: {
        [uri1.toString()]: [
          {
            range: Range.create(0, 9, 0, 12),
            newText: 'bar',
          },
        ],
        [uri2.toString()]: [
          {
            range: Range.create(1, 6, 1, 9),
            newText: 'bar',
          },
        ],
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should be able to rename a citation', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.bib', '@article{foo,}');
    const uri2 = builder.document(
      'bar.tex',
      '\\addbibresource{foo.bib}\n\\cite{foo}',
    );
    const context = builder.context(uri2, 1, 6, 'bar');
    const actual = await BibtexEntryRenameProvider.execute(context);
    const expected: WorkspaceEdit = {
      changes: {
        [uri1.toString()]: [
          {
            range: Range.create(0, 9, 0, 12),
            newText: 'bar',
          },
        ],
        [uri2.toString()]: [
          {
            range: Range.create(1, 6, 1, 9),
            newText: 'bar',
          },
        ],
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should not rename unrelated structures (1)', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.bib', '@article{foo,}');
    builder.document('bar.tex', '\\addbibresource{foo.bib}\n\\cite{foo}');
    const context = builder.context(uri, 0, 0);
    const actual = await BibtexEntryRenameProvider.execute(context);
    const expected: WorkspaceEdit = { changes: {} };
    expect(actual).toBeUndefined();
  });

  it('should not rename unrelated structures (2)', async () => {
    const builder = new WorkspaceBuilder();
    builder.document('foo.bib', '@article{foo,}');
    const uri = builder.document(
      'bar.tex',
      '\\addbibresource{foo.bib}\n\\cite{foo}',
    );
    const context = builder.context(uri, 0, 0);
    const actual = await BibtexEntryRenameProvider.execute(context);
    expect(actual).toBeUndefined();
  });
});
