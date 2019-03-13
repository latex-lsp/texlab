import { runSingleFile } from '../../workspaceBuilder';
import { KERNEL_COMMANDS } from '../kernel';
import { BibtexKernelCommandCompletionProvider } from './kernelCommand';

describe('BibtexKernelCommandProvider', () => {
  const provider = BibtexKernelCommandCompletionProvider;

  it('should provide completion inside BibTeX commands', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = \\baz}',
      line: 0,
      character: 23,
    });
    expect(items.map(x => x.label)).toEqual(KERNEL_COMMANDS);
  });

  it('should not provide completion inside BibTeX keys', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = \\baz}',
      line: 0,
      character: 11,
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
