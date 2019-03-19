import { Range, WorkspaceEdit } from 'vscode-languageserver';
import { WorkspaceBuilder } from '../workspaceBuilder';
import { LatexEnvironmentRenameProvider } from './latexEnvironment';

describe('LatexEnvironmentRenameProvider', () => {
  const provider = LatexEnvironmentRenameProvider;

  it('should rename unmatched environments', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\begin{foo}\n\\end{bar}');
    const context = builder.context(uri, 0, 8, 'baz');
    const actual = await provider.execute(context);
    const expected: WorkspaceEdit = {
      changes: {
        [uri.toString()]: [
          { range: Range.create(0, 7, 0, 10), newText: 'baz' },
          { range: Range.create(1, 5, 1, 8), newText: 'baz' },
        ],
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should not rename unrelated environments', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\begin{foo}\n\\end{bar}');
    const context = builder.context(uri, 0, 5, 'baz');
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });

  it('should not process BibTeX documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.bib', '');
    const context = builder.context(uri, 0, 0, 'baz');
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });
});
