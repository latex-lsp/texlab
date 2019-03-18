import {
  BundleProducer,
  FuseBox,
  JSONPlugin,
  Plugin,
  QuantumPlugin,
  WorkFlowContext,
} from 'fuse-box';
import { context, task } from 'fuse-box/sparky';
import { EOL } from 'os';

const BUNDLE_NAME = 'texlab';
const INSTRUCTIONS = '> src/main.ts';

interface Context {
  createFuse(isProduction: boolean): FuseBox;
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
                }),
              ]
            : []),
          new ShebangPlugin(isProduction),
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

class ShebangPlugin implements Plugin {
  public test = /\.js$/;

  private readonly SHEBANG = '#!/usr/bin/env node' + EOL;

  constructor(private isProduction: boolean) {}

  public preBundle(ctx: WorkFlowContext) {
    if (this.isProduction) {
      return;
    }

    ctx.source.addContent(this.SHEBANG);
  }

  public async producerEnd(producer: BundleProducer) {
    if (!this.isProduction) {
      return;
    }

    for (const bundle of producer.bundles.values()) {
      const code = bundle.generatedCode.toString();
      const buffer = Buffer.from(this.SHEBANG + code);
      await bundle.context.output.writeCurrent(buffer);
    }
  }
}
