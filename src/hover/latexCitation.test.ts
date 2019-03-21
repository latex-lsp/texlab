import { runSingleFile, WorkspaceBuilder } from '../workspaceBuilder';
import { LatexCitationHoverProvider } from './latexCitation';

describe('LatexCitationHoverProvider', () => {
  const provider = LatexCitationHoverProvider;

  it('should show the bibliography when hovering over citations', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document(
      'foo.tex',
      '\\addbibresource{bar.bib}\n\\cite{bar}\n\\cite{foo}',
    );
    builder.document('bar.bib', '@article{foo, author = {bar}}');
    const context = builder.context(uri, 2, 8);
    const actual = await provider.execute(context);
    expect(actual).toBeDefined();
  });

  it('should show the bibliography when hovering over entries', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, author = {bar}}',
      line: 0,
      character: 11,
    });
    expect(actual).toBeDefined();
  });

  it('should not show the bibliography when hovering over unknown citations', async () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document(
      'foo.tex',
      '\\addbibresource{bar.bib}\n\\cite{bar}',
    );
    builder.document('bar.bib', '@article{foo, author = {bar}}');
    const context = builder.context(uri, 1, 8);
    const actual = await provider.execute(context);
    expect(actual).toBeUndefined();
  });

  it('should not show the bibliography when hovering over comments (1)', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: 'foo',
      line: 0,
      character: 1,
    });
    expect(actual).toBeUndefined();
  });

  it('should not show the bibliography when hovering over comments (2)', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '%foo',
      line: 0,
      character: 1,
    });
    expect(actual).toBeUndefined();
  });

  it('should not show the bibliography when the entry is invalid', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,}',
      line: 0,
      character: 11,
    });
    expect(actual).toBeUndefined();
  });
});
