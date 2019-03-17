import { runSingleFile } from '../../../../workspaceBuilder';
import { LatexComponentSource } from '../component';
import { LatexCommandSymbolCompletionProvider } from './command';

describe('LatexCommandSymbolCompletionProvider', () => {
  it('should provide \\VarDelta if amsmath.sty is included', async () => {
    const database: LatexComponentSource = {
      relatedComponents: () => {
        return [
          {
            fileNames: ['amsmath.sty'],
            commands: [],
            environments: [],
            references: [],
          },
        ];
      },
    };
    const provider = LatexCommandSymbolCompletionProvider(database);
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\',
      line: 0,
      character: 1,
    });
    expect(items.map(x => x.label)).toContain('varDelta');
  });
});
