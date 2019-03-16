import { runSingleFile, WorkspaceBuilder } from '../../workspaceBuilder';
import { LatexUserEnvironmentCompletionProvider } from './userEnvironment';

describe('LatexUserEnvironmentCompletionProvider', () => {
  const provider = LatexUserEnvironmentCompletionProvider;

  it('should provide completion inside environments', async () => {
    const builder = new WorkspaceBuilder();
    builder.document('foo.tex', '\\addbibresource{bar.bib}\n\\include{baz}');
    builder.document('bar.bib', '');
    const uri = builder.document(
      'baz.tex',
      '\\begin{foo}\\begin{foo}\n\\end{bar}',
    );
    const context = builder.context(uri, 0, 8);
    const items = await provider.execute(context);
    expect(items.map(x => x.label)).toEqual(['foo', 'bar']);
  });

  it('should ignore empty environments', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\begin{foo}\\end{}\n\\begin{}',
      line: 1,
      character: 8,
    });
    expect(items).toEqual([]);
  });
});
