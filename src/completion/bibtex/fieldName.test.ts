import { BIBTEX_FIELDS, getFieldName } from '../../metadata/bibtexField';
import { runSingleFile } from '../../workspaceBuilder';
import { BibtexFieldNameCompletionProvider } from './fieldName';

describe('BibtexFieldNameCompletionProvider', () => {
  const provider = BibtexFieldNameCompletionProvider;
  const FIELDS = BIBTEX_FIELDS.map(getFieldName);

  it('should provide completion inside entries', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,}',
      line: 0,
      character: 13,
    });
    expect(items.map(x => x.label)).toEqual(FIELDS);
  });

  it('should provide completion inside fields', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar}',
      line: 0,
      character: 15,
    });
    expect(items.map(x => x.label)).toEqual(FIELDS);
  });

  it('should not provide completion inside keys', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,}',
      line: 0,
      character: 12,
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
