import { Hover, MarkupKind } from 'vscode-languageserver';
import {
  BibtexEntryTypeHoverProvider,
  BibtexFieldHoverProvider,
} from './hover';
import { BibtexField, getFieldDocumentation } from './metadata/bibtexField';
import { getTypeDocumentation } from './metadata/bibtexType';
import { runSingleFile } from './workspaceBuilder';

describe('BibtexEntryTypeHoverProvider', () => {
  const provider = new BibtexEntryTypeHoverProvider();

  it('should provide documentation when hovering over the entry type', async () => {
    const expected: Hover = {
      contents: {
        kind: MarkupKind.Markdown,
        value: getTypeDocumentation('article')!,
      },
    };
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,}',
      line: 0,
      character: 3,
    });
    expect(actual).toEqual(expected);
  });

  it('should not provide documentation when hovering over the entry key', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,}',
      line: 0,
      character: 11,
    });
    expect(actual).toBeUndefined();
  });

  it('should not provide documentation when hovering over LaTeX commands', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 1,
    });
    expect(actual).toBeUndefined();
  });
});

describe('BibtexFieldHoverProvider', () => {
  const provider = new BibtexFieldHoverProvider();

  it('should provide documentation when hovering over known fields', async () => {
    const expected: Hover = {
      contents: {
        kind: MarkupKind.Markdown,
        value: getFieldDocumentation(BibtexField.Author),
      },
    };
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, author = bar}',
      line: 0,
      character: 15,
    });
    expect(actual).toEqual(expected);
  });

  it('should not provide documentation when hovering over unknown fields', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = baz}',
      line: 0,
      character: 15,
    });
    expect(actual).toBeUndefined();
  });

  it('should not provide documentation when hovering over entry keys', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = baz}',
      line: 0,
      character: 11,
    });
    expect(actual).toBeUndefined();
  });

  it('should not provide documentation when hovering over LaTeX commands', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 1,
    });
    expect(actual).toBeUndefined();
  });
});
