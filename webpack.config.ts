import CleanWebpackPlugin from 'clean-webpack-plugin';
import * as path from 'path';
import { BannerPlugin, Configuration } from 'webpack';

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
  externals: {
    'mathjax-node': 'commonjs mathjax-node',
    'citation-js': 'commonjs citation-js',
    turndown: 'commonjs turndown',
  },
  resolve: {
    extensions: ['.ts', '.js', '.json'],
  },
  plugins: [
    new CleanWebpackPlugin(),
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
    ],
  },
};

export default config;
