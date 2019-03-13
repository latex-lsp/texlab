import { BIBTEX_TYPES } from '../../metadata/bibtexType';
import { runSingleFile } from '../../workspaceBuilder';
import { BibtexEntryTypeCompletionProvider } from './entryType';

describe('BibtexEntryTypeCompletionProvider', () => {
  const provider = BibtexEntryTypeCompletionProvider;

  it('should provide completion after @', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@',
      line: 0,
      character: 1,
    });
    expect(items.map(x => x.label)).toEqual(BIBTEX_TYPES);
  });

  it('should not provide completion inside entries', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,}',
      line: 0,
      character: 11,
    });
    expect(items).toEqual([]);
  });

  it('should not provide completion inside comments', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: 'foo',
      line: 0,
      character: 2,
    });
    expect(items).toEqual([]);
  });

  it('should not provide completion inside LaTeX commands', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 1,
    });
    expect(items).toEqual([]);
  });
});
