import {
  BundleProducer,
  FuseBox,
  JSONPlugin,
  Plugin,
  QuantumPlugin,
} from 'fuse-box';
import { EOL } from 'os';

class ShebangPlugin implements Plugin {
  public test = /\.js$/;

  private readonly SHEBANG = '#!/usr/bin/env node' + EOL;

  public async producerEnd(producer: BundleProducer) {
    for (const bundle of producer.bundles.values()) {
      const code = bundle.generatedCode.toString();
      const buffer = Buffer.from(this.SHEBANG + code);
      await bundle.context.output.writeCurrent(buffer);
    }
  }
}

const BUNDLE_NAME = 'texlab';

const fuse = FuseBox.init({
  homeDir: '.',
  target: 'server@es6',
  output: 'dist/$name.js',
  sourceMaps: false,
  plugins: [
    JSONPlugin(),
    QuantumPlugin({
      uglify: true,
      treeshake: true,
      bakeApiIntoBundle: BUNDLE_NAME,
      api: core => {
        core.solveComputed('jsdom/lib/jsdom/utils.js');
        core.solveComputed('parse5/lib/index.js');
      },
    }),
    new ShebangPlugin(),
  ],
});

fuse.bundle(BUNDLE_NAME).instructions('> src/texlab.ts');
fuse.run();
