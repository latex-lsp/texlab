import { FuseBox, Plugin, QuantumPlugin } from 'fuse-box';
import { context, task } from 'fuse-box/sparky';

const BUNDLE_NAME = 'texlab';
const INSTRUCTIONS = '> main.ts';

interface Context {
  createFuse(isProduction: boolean): FuseBox;
}

context(
  class implements Context {
    public createFuse(isProduction: boolean): FuseBox {
      const sourceMaps = isProduction
        ? false
        : { inline: false, vendor: false };

      const plugins: Plugin[] = [];
      if (isProduction) {
        plugins.push(
          QuantumPlugin({
            uglify: true,
            treeshake: true,
            bakeApiIntoBundle: BUNDLE_NAME,
            api: core => {
              core.solveComputed('vscode-languageserver/lib/files.js');
            },
          }),
        );
      }

      return FuseBox.init({
        homeDir: 'src',
        target: 'server@es6',
        output: 'dist/$name.js',
        sourceMaps,
        plugins,
      });
    }
  },
);

task('default', async ({ createFuse }: Context) => {
  const fuse = createFuse(false);
  fuse
    .bundle(BUNDLE_NAME)
    .instructions(INSTRUCTIONS)
    .watch();

  await fuse.run();
});

task('dist', async ({ createFuse }: Context) => {
  const fuse = createFuse(true);
  fuse.bundle(BUNDLE_NAME).instructions(INSTRUCTIONS);

  await fuse.run();
});
