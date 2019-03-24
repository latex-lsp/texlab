import mock from 'mock-fs';
import { runSingleFile } from '../../workspaceBuilder';
import { LatexIncludeCompletionProvider } from './include';

describe('LatexIncludeCompletionProvider', () => {
  beforeEach(() => {
    mock({
      'file1.tex': '',
      'file2.tex': '',
      foo: {
        'file3.tex': '',
        'file4.bib': '',
      },
      bar: {
        'file5.png': '',
        'file6.jpg': '',
        baz: {
          'file7.pdf': '',
          'file8.svg': '',
          file9: '',
        },
      },
    });
  });

  afterEach(() => {
    mock.restore();
  });

  it('should provide completion for \\include{}', async () => {
    const items = await runSingleFile({
      provider: LatexIncludeCompletionProvider,
      file: 'foo.tex',
      line: 0,
      character: 9,
      text: '\\include{}',
    });
    expect(items.map(x => x.label)).toEqual(['bar', 'file1', 'file2', 'foo']);
  });

  it('should provide completion for \\input{}', async () => {
    const items = await runSingleFile({
      provider: LatexIncludeCompletionProvider,
      file: 'foo.tex',
      line: 0,
      character: 7,
      text: '\\input{}',
    });
    expect(items.map(x => x.label)).toEqual([
      'bar',
      'file1.tex',
      'file2.tex',
      'foo',
    ]);
  });

  it('should provide completion for \\includegraphics', async () => {
    const items = await runSingleFile({
      provider: LatexIncludeCompletionProvider,
      file: 'foo.tex',
      line: 0,
      character: 21,
      text: '\\includegraphics{bar/}',
    });
    expect(items.map(x => x.label)).toEqual(['baz', 'file5.png', 'file6.jpg']);
  });

  it('should provide completion for \\bibliography', async () => {
    const items = await runSingleFile({
      provider: LatexIncludeCompletionProvider,
      file: 'foo.tex',
      line: 0,
      character: 21,
      text: '\\bibliography{foo/file}',
    });
    expect(items.map(x => x.label)).toEqual(['file4.bib']);
  });

  it('should provide completion for \\includesvg', async () => {
    const items = await runSingleFile({
      provider: LatexIncludeCompletionProvider,
      file: 'foo.tex',
      line: 0,
      character: 20,
      text: '\\includesvg{bar/baz/}',
    });
    expect(items.map(x => x.label)).toEqual(['file8']);
  });

  it('should not provide completion for invalid paths', async () => {
    const items = await runSingleFile({
      provider: LatexIncludeCompletionProvider,
      file: 'foo.tex',
      line: 0,
      character: 17,
      text: '\\include{foo/bar/}',
    });
    expect(items.map(x => x.label)).toEqual([]);
  });
});
