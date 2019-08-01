import path from 'path';
import { Configuration } from 'webpack';

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
