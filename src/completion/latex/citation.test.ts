import { WorkspaceBuilder } from '../../workspaceBuilder';
import { LatexCitationCompletionProvider } from './citation';

describe('LatexCitationCompletionProvider', () => {
  const provider = LatexCitationCompletionProvider;

  it('should provide completion inside \\cite{}', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document(
      'foo.tex',
      '\\bibliography{bar.bib}\n\\cite{}',
    );
    builder.document('bar.bib', '@article{foo,}\n@article{bar,}\n@article');
    const context = builder.context(uri, 1, 6);
    const items = await provider.execute(context);
    expect(items.map(x => x.label)).toEqual(['foo', 'bar']);
  });
});
