import { deferred, FeatureProvider } from './provider';
import { runSingleFile, SingleFileRunOptions } from './workspaceBuilder';

describe('Provider', () => {
  it('should eventually provide the source to the given provider', async () => {
    let resolve: any;
    const source = new Promise<number>(x => (resolve = x));

    const factory = (x: number): FeatureProvider<{}, number> => ({
      execute: async () => x,
    });

    const value = 42;
    const defaultValue = -1;
    const provider = deferred(factory, source, defaultValue);
    const options: SingleFileRunOptions<{}, number> = {
      provider,
      file: 'foo.tex',
      text: '\\foo',
      line: 0,
      character: 2,
    };

    expect(await runSingleFile(options)).toEqual(defaultValue);

    resolve(value);
    await source;
    expect(await runSingleFile(options)).toEqual(value);
  });
});
