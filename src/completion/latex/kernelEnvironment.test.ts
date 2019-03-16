import { runSingleFile } from '../../workspaceBuilder';
import { KERNEL_ENVIRONMENTS } from '../kernel';
import { LatexKernelEnvironmentCompletionProvider } from './kernelEnvironment';

describe('LatexKernelEnvironmentCompletionProvider', () => {
  const provider = LatexKernelEnvironmentCompletionProvider;

  it('should provide completion inside \\begin{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\begin{}',
      line: 0,
      character: 7,
    });
    expect(items.map(x => x.label)).toEqual(KERNEL_ENVIRONMENTS);
  });

  it('should not provide completion inside \\Begin{}', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\Begin{}',
      line: 0,
      character: 7,
    });
    expect(items).toEqual([]);
  });
});
