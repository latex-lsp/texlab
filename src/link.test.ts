import { DocumentLink, Range } from 'vscode-languageserver';
import { LatexIncludeLinkProvider } from './link';
import { WorkspaceBuilder } from './workspaceBuilder';

describe('LatexIncludeLinkProvider', () => {
  it('should resolve valid includes', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.tex', '\\include{bar}');
    const uri2 = builder.document('bar.tex', '');
    const context = builder.context(uri1, 0, 0);
    const actual = await LatexIncludeLinkProvider.execute(context, undefined);
    const expected: DocumentLink[] = [
      {
        range: Range.create(0, 9, 0, 12),
        target: uri2.toString(),
      },
    ];
    expect(actual).toEqual(expected);
  });

  it('should ignore invalid includes', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\include{bar}');
    const context = builder.context(uri, 0, 0);
    const actual = await LatexIncludeLinkProvider.execute(context, undefined);
    expect(actual).toEqual([]);
  });
});
