import { BIBTEX_FIELDS, getFieldName } from '../metadata/bibtexField';
import { BIBTEX_TYPES } from '../metadata/bibtexType';
import { runSingleFile } from '../workspaceBuilder';
import {
  BibtexEntryTypeCompletionProvider,
  BibtexFieldNameCompletionProvider,
} from './index';

describe('BibtexFieldNameCompletionProvider', () => {
  const FIELDS = BIBTEX_FIELDS.map(getFieldName);
  const provider = new BibtexFieldNameCompletionProvider();

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
});

describe('BibtexEntryTypeCompletionProvider', () => {
  const provider = new BibtexEntryTypeCompletionProvider();

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
});
