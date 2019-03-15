import { runSingleFile, WorkspaceBuilder } from '../../workspaceBuilder';
import { LatexUserCommandCompletionProvider } from './userCommand';

describe('LatexUserCommandCompletionProvider', () => {
  const provider = LatexUserCommandCompletionProvider;

  it('should provide completion inside LaTeX commands', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo\n\\bar',
      line: 1,
      character: 1,
    });
    expect(items.map(x => x.label)).toEqual(['foo']);
  });

  it('should ignore commands from BibTeX documents', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\addbibresource{bar.bib}\n\\foo');
    builder.document('bar.bib', '@article{foo, bar = {\\baz}}');
    const context = builder.context(uri, 1, 1);
    const items = await provider.execute(context);
    expect(items.map(x => x.label)).toEqual(['addbibresource']);
  });
});
