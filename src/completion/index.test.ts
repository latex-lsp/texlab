import { WorkspaceBuilder } from '../workspaceBuilder';
import { CompletionProvider } from './index';

describe('Completion', () => {
  it('should provide completion for BibTeX field names (1)', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.bib', '@article{foo,}');
    const context = builder.completion(uri, 0, 13);
    const items = await CompletionProvider.execute(context, undefined);
    expect(items.map(x => x.label)).toContain('title');
  });

  it('should provide completion for BibTeX field names (1)', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.bib', '@article{foo, bar}');
    const context = builder.completion(uri, 0, 15);
    const items = await CompletionProvider.execute(context, undefined);
    expect(items.map(x => x.label)).toContain('title');
  });
});
