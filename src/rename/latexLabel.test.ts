import { Range, WorkspaceEdit } from 'vscode-languageserver';
import { WorkspaceBuilder } from '../workspaceBuilder';
import { LatexLabelRenameProvider } from './latexLabel';

describe('LatexLabelRenameProvider', () => {
  const provider = LatexLabelRenameProvider;

  it('should be able to rename a label definition', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.tex', '\\label{foo}\n\\include{bar}');
    const uri2 = builder.document('bar.tex', '\\ref{foo}');
    const context = builder.context(uri1, 0, 7, 'bar');
    const actual = await provider.execute(context);
    const expected: WorkspaceEdit = {
      changes: {
        [uri1.toString()]: [
          { range: Range.create(0, 7, 0, 10), newText: 'bar' },
        ],
        [uri2.toString()]: [
          { range: Range.create(0, 5, 0, 8), newText: 'bar' },
        ],
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should be able to rename a label reference', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document(
      'foo.tex',
      '\\label{foo}\n\\include{bar}\n\\addbibresource{baz.bib}',
    );
    const uri2 = builder.document('bar.tex', '\\ref{foo}');
    builder.document('baz.bib', '');
    const context = builder.context(uri2, 0, 5, 'bar');
    const actual = await provider.execute(context);
    const expected: WorkspaceEdit = {
      changes: {
        [uri1.toString()]: [
          { range: Range.create(0, 7, 0, 10), newText: 'bar' },
        ],
        [uri2.toString()]: [
          { range: Range.create(0, 5, 0, 8), newText: 'bar' },
        ],
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should not rename unrelated structures', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\foo{bar}');
    const context = builder.context(uri, 0, 6, 'baz');
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });

  it('should not process BibTeX documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.bib', '');
    const context = builder.context(uri, 0, 0, 'foo');
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });
});

// val builder = WorkspaceBuilder()
//                 .document("foo.tex", "\\label{foo}\n\\include{bar}")
//                 .document("bar.tex", "\\ref{foo}")
//         val edit = builder
//                 .rename(document, line, character, "bar")
//                 .let { LatexLabelRenamer.get(it)!! }

//         Assertions.assertEquals(2, edit.changes.size)

//         val document1 = builder.uri("foo.tex").toString()
//         val change1 = edit.changes.getValue(document1)
//         assertEquals(1, change1.size)
//         assertEquals(Range(Position(0, 7), Position(0, 10)), change1[0].range)
//         assertEquals("bar", change1[0].newText)

//         val document2 = builder.uri("bar.tex").toString()
//         val change2 = edit.changes.getValue(document2)
//         assertEquals(1, change2.size)
//         assertEquals(Range(Position(0, 5), Position(0, 8)), change2[0].range)
//         assertEquals("bar", change2[0].newText)
