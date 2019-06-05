import path from 'path';
import { Configuration, ContextReplacementPlugin } from 'webpack';

const config: Configuration = {
  target: 'node',
  entry: './src/main.ts',
  mode: 'production',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'citeproc.js',
    libraryTarget: 'commonjs2',
    devtoolModuleFilenameTemplate: '../[resource-path]',
  },
  resolve: {
    extensions: ['.ts', '.js', '.json'],
  },
  plugins: [
    // Do not resolve canvas API of jsdom (jsdom/lib/jsdom/utils.js 186:21-40)
    new ContextReplacementPlugin(/jsdom[/\\]lib/, /^$/),

    // Do not resolve streaming API of parse5 (parse5/lib/index.js 55:23-49)
    new ContextReplacementPlugin(/parse5[/\\]lib/, /^$/),
  ],
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'ts-loader',
          },
        ],
      },
      {
        // Map browser dependencies to an empty module
        test: /node_modules[/\\](sync-request|isomorphic-fetch|ws)[/\\]/,
        use: 'null-loader',
      },
    ],
  },
};

export default config;
