import CleanWebpackPlugin from 'clean-webpack-plugin';
import path from 'path';
import { BannerPlugin, Configuration, ContextReplacementPlugin } from 'webpack';

const config: Configuration = {
  target: 'node',
  entry: './src/texlab.ts',
  mode: 'production',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'texlab.js',
    libraryTarget: 'commonjs2',
    devtoolModuleFilenameTemplate: '../[resource-path]',
  },
  resolve: {
    extensions: ['.ts', '.js', '.json'],
  },
  plugins: [
    new CleanWebpackPlugin(),

    // Do not resolve canvas API of jsdom (jsdom/lib/jsdom/utils.js 186:21-40)
    new ContextReplacementPlugin(/jsdom[/\\]lib/, /^$/),

    // Do not resolve streaming API of parse5 (parse5/lib/index.js 55:23-49)
    new ContextReplacementPlugin(/parse5[/\\]lib/, /^$/),

    // (mathjax3/util/AsyncLoad.js 8:15-34)
    new ContextReplacementPlugin(/mathjax3[/\\]util/, /^$/),

    new BannerPlugin({ banner: '#!/usr/bin/env node', raw: true }),
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
      {
        // Ignore browser import in mathjax3/util/asyncLoad/system.js
        test: /node_modules[/\\]mathjax3[/\\]/,
        parser: {
          system: true,
        },
      },
    ],
  },
};

export default config;
