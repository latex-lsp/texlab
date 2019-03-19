import { Hover, MarkupKind } from 'vscode-languageserver';
import { LatexComponentSource } from '../completion/latex/data/component';
import { runSingleFile } from '../workspaceBuilder';
import { LatexCommandHoverProvider } from './latexCommand';

describe('LatexCommandHoverProvider', () => {
  it('should show the component when hovering over a command', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => {
        return [
          {
            fileNames: ['lipsum.sty'],
            commands: ['lipsum'],
            environments: [],
            references: [],
          },
        ];
      },
    };
    const provider = LatexCommandHoverProvider(database);
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\lipsum',
      line: 0,
      character: 0,
    });
    const expected: Hover = {
      contents: {
        kind: MarkupKind.PlainText,
        value: 'lipsum.sty',
      },
    };
    expect(actual).toEqual(expected);
  });

  it('should show nothing when hovering over text', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => [],
    };
    const provider = LatexCommandHoverProvider(database);
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: 'foo',
      line: 0,
      character: 0,
    });
    expect(actual).toBeUndefined();
  });

  it('should show nothing when hovering over unknown commands', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => [],
    };
    const provider = LatexCommandHoverProvider(database);
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 0,
    });
    expect(actual).toBeUndefined();
  });

  it('should show nothing when hovering over BibTeX commands', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => [],
    };
    const provider = LatexCommandHoverProvider(database);
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = \n\\baz}',
      line: 1,
      character: 1,
    });
    expect(actual).toBeUndefined();
  });
});
