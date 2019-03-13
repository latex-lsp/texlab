import { runSingleFile } from '../../workspaceBuilder';
import { KERNEL_COMMANDS } from '../kernel';
import { LatexKernelCommandProvider } from './kernelCommand';

describe('LatexKernelCommandProvider', () => {
  const provider = LatexKernelCommandProvider;

  it('should provide completion inside LaTeX commands', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 2,
    });
    expect(items.map(x => x.label)).toEqual(KERNEL_COMMANDS);
  });

  it('should not provide completion at the start of a LaTeX command', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 0,
    });
    expect(items).toEqual([]);
  });

  it('should not provide completion inside LaTeX words', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: 'foo',
      line: 0,
      character: 1,
    });
    expect(items).toEqual([]);
  });

  it('should not provide completion inside BibTeX commands', async () => {
    const items = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = \\baz}',
      line: 0,
      character: 23,
    });
    expect(items).toEqual([]);
  });
});
