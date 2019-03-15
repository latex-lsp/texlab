import { WorkspaceBuilder } from '../../workspaceBuilder';
import { LatexLabelCompletionProvider } from './label';

describe('LatexLabelCompletionProvider', () => {
  const provider = LatexLabelCompletionProvider;

  it('should provide completion inside label commands', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document(
      'foo.tex',
      '\\addbibresource{bar.bib}\\include{baz}\n\\ref{}',
    );
    builder.document('bar.bib', '');
    builder.document('baz.tex', '\\label{foo}\\label{bar}');
    const context = builder.context(uri, 1, 5);
    const items = await provider.execute(context);
    expect(items.map(x => x.label)).toEqual(['foo', 'bar']);
  });
});
