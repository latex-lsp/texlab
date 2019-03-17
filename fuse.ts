import {
  BundleProducer,
  FuseBox,
  JSONPlugin,
  Plugin,
  QuantumPlugin,
} from 'fuse-box';
import { context, task } from 'fuse-box/sparky';
import { EOL } from 'os';

const BUNDLE_NAME = 'texlab';
const INSTRUCTIONS = '> src/main.ts';

interface Context {
  createFuse(isProduction: boolean): FuseBox;
}

class ShebangPlugin implements Plugin {
  public test = /\.js$/;

  public async producerEnd(producer: BundleProducer) {
    for (const bundle of producer.bundles.values()) {
      const code = bundle.generatedCode.toString();
      const buffer = Buffer.from('#!/usr/bin/env node' + EOL + code);
      await bundle.context.output.writeCurrent(buffer);
    }
  }
}

context(
  class implements Context {
    public createFuse(isProduction: boolean): FuseBox {
      return FuseBox.init({
        homeDir: '.',
        target: 'server@es6',
        output: 'dist/$name.js',
        sourceMaps: isProduction ? false : { inline: false, vendor: false },
        plugins: [
          JSONPlugin(),
          ...(isProduction
            ? [
                QuantumPlugin({
                  uglify: true,
                  treeshake: true,
                  bakeApiIntoBundle: BUNDLE_NAME,
                  api: core => {
                    core.solveComputed('vscode-languageserver/lib/files.js');
                  },
                }),
              ]
            : []),
          new ShebangPlugin(),
        ],
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
