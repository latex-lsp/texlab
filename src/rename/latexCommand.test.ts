import { Range, WorkspaceEdit } from 'vscode-languageserver';
import { WorkspaceBuilder } from '../workspaceBuilder';
import { LatexCommandRenameProvider } from './latexCommand';

describe('LatexCommandRenameProvider', () => {
  const provider = LatexCommandRenameProvider;

  it('should rename commands in related documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document(
      'foo.tex',
      '\\addbibresource{bar.bib}\n\\include{baz}\n\\foo',
    );
    builder.document('bar.bib', '@article{foo, bar = \\foo}');
    const uri2 = builder.document('baz.tex', '\\foo');
    const context = builder.context(uri1, 2, 1, 'bar');
    const actual = await provider.execute(context);
    const expected: WorkspaceEdit = {
      changes: {
        [uri1.toString()]: [
          { range: Range.create(2, 0, 2, 4), newText: '\\bar' },
        ],
        [uri2.toString()]: [
          { range: Range.create(0, 0, 0, 4), newText: '\\bar' },
        ],
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should ignore an empty document', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '');
    const context = builder.context(uri, 0, 0, 'foo');
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });

  it('should ignore BibTeX documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.bib', '');
    const context = builder.context(uri, 0, 0, 'bar');
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });
});
