import { runSingleFile } from '../../workspaceBuilder';
import { LatexComponentSource } from './data/component';
import { COMMANDS, LatexTikzCommandCompletionProvider } from './tikzCommand';

describe('LatexTikzCommandCompletionProvider', () => {
  it('should provide completion when TikZ is included', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => {
        return [
          {
            fileNames: ['tikz.sty'],
            commands: [],
            environments: [],
            references: [],
          },
        ];
      },
    };
    const provider = LatexTikzCommandCompletionProvider(database);
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\',
      line: 0,
      character: 1,
    });
    expect(items.map(x => x.label)).toEqual(COMMANDS);
  });

  it('should not provide completion when TikZ is not included', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => [],
    };
    const provider = LatexTikzCommandCompletionProvider(database);
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\',
      line: 0,
      character: 1,
    });
    expect(items).toEqual([]);
  });
});
