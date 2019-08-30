const path = require('path');
const webpack = require('webpack');

const config = {
  target: 'web',
  entry: ['babel-polyfill', './src/index.js'],
  mode: 'production',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'citeproc.js',
    library: 'citeproc',
    libraryTarget: 'var',
    devtoolModuleFilenameTemplate: '../[resource-path]',
  },
  resolve: {
    extensions: ['.js'],
  },
  plugins: [
    new webpack.BannerPlugin({ raw: true, banner: "var console = {};"}),
  ],
  module: {
    rules: [
      {
        use: {
          loader: 'babel-loader',
          options: {
            presets: ["@babel/preset-env"]
          }
        },
        test: /\.js$/,
      },
      {
        // Map browser dependencies to an empty module
        test: /node_modules[/\\](sync-request|isomorphic-fetch|ws)[/\\]/,
        use: 'null-loader',
      },
    ],
  },
};

module.exports = config;
