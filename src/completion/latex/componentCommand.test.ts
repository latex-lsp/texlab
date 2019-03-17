import { runSingleFile } from '../../workspaceBuilder';
import { LatexComponentCommandCompletionProvider } from './componentCommand';
import { LatexComponentSource } from './data/component';

describe('LatexComponentCommandCompletionProvider', () => {
  it('should provide completion inside LaTeX commands', async () => {
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
    const provider = LatexComponentCommandCompletionProvider(database);
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\lipsum',
      line: 0,
      character: 5,
    });
    expect(items.map(x => x.label)).toEqual(['lipsum']);
  });
});
