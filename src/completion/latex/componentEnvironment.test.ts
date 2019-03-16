import { runSingleFile } from '../../workspaceBuilder';
import { LatexComponentEnvironmentCompletionProvider } from './componentEnvironment';
import { LatexComponentSource } from './data/component';

describe('LatexComponentEnvironmentCompletionProvider', () => {
  it('should provide completion inside LaTeX environments', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => {
        return [
          {
            fileNames: ['amsmath.sty'],
            commands: [],
            environments: ['gather'],
            references: [],
          },
        ];
      },
    };
    const provider = LatexComponentEnvironmentCompletionProvider(database);
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\begin{}',
      line: 0,
      character: 7,
    });
    expect(items.map(x => x.label)).toEqual(['gather']);
  });
});
