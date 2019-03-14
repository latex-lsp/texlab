import { Definition, Range } from 'vscode-languageserver';
import { runSingleFile, WorkspaceBuilder } from '../workspaceBuilder';
import { LatexLabelDefinitionProvider } from './label';

describe('LatexLabelDefinitionProvider', () => {
  const provider = LatexLabelDefinitionProvider;

  it('should find the first definition of the label', async () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document(
      'foo.tex',
      '\\addbibresource{baz.bib}\n\\include{bar}\n\\ref{bar}\n',
    );
    const uri2 = builder.document('bar.tex', '\\label{foo}\n\\label{bar}');
    builder.document('baz.bib', '');
    const context1 = builder.context(uri1, 2, 7);
    const context2 = builder.context(uri2, 1, 8);
    const result1 = await provider.execute(context1);
    const result2 = await provider.execute(context2);
    const expected: Definition[] = [
      {
        uri: uri2.toString(),
        range: Range.create(1, 7, 1, 10),
      },
    ];
    expect(result1).toEqual(expected);
    expect(result2).toEqual(expected);
  });

  it('should return an empty array if there is no matching label', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\ref{foo}',
      line: 0,
      character: 6,
    });
    expect(actual).toEqual([]);
  });

  it('should return an empty array if the cursor is not inside of a label', async () => {
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
