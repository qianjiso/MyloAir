const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

const isProd = process.env.NODE_ENV === 'production';

module.exports = {
  mode: process.env.NODE_ENV || 'development',
  devtool: isProd ? false : 'source-map',
  entry: {
    main: './src/renderer/index.tsx',
  },
  target: 'web',
  output: {
    path: path.resolve(__dirname, 'dist/renderer'),
    filename: isProd ? '[name].[contenthash:8].js' : '[name].js',
    chunkFilename: isProd ? '[name].[contenthash:8].js' : '[name].js',
    clean: true,
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.css$/,
        use: [isProd ? MiniCssExtractPlugin.loader : 'style-loader', 'css-loader'],
      },
      {
        test: /\.(png|jpe?g|gif|svg)$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    minimize: isProd,
    sideEffects: true,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        antd: {
          test: /[\\/]node_modules[\\/](antd|@ant-design)[\\/]/,
          name: 'antd',
          priority: 10,
          reuseExistingChunk: true,
        },
        react: {
          test: /[\\/]node_modules[\\/](react|react-dom|react-router-dom)[\\/]/,
          name: 'react',
          priority: 9,
          reuseExistingChunk: true,
        },
        tauri: {
          test: /[\\/]node_modules[\\/](@tauri-apps)[\\/]/,
          name: 'tauri',
          priority: 8,
          reuseExistingChunk: true,
        },
        crypto: {
          test: /[\\/]node_modules[\\/](crypto-js|jszip)[\\/]/,
          name: 'crypto',
          priority: 7,
          reuseExistingChunk: true,
        },
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          priority: 5,
          reuseExistingChunk: true,
        },
        default: {
          minChunks: 2,
          priority: -10,
          reuseExistingChunk: true,
        },
      },
    },
    runtimeChunk: 'single',
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js', '.jsx'],
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './src/renderer/index.html',
      inject: 'body',
      minify: isProd
        ? {
            removeComments: true,
            collapseWhitespace: true,
            removeRedundantAttributes: true,
            useShortDoctype: true,
            removeEmptyAttributes: true,
            removeStyleLinkTypeAttributes: true,
            keepClosingSlash: true,
            minifyJS: true,
            minifyCSS: true,
            minifyURLs: true,
          }
        : false,
    }),
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify(
        process.env.NODE_ENV || 'development'
      ),
    }),
    isProd && new MiniCssExtractPlugin({
      filename: '[name].[contenthash:8].css',
      chunkFilename: '[name].[contenthash:8].css',
    }),
  ].filter(Boolean),
  performance: {
    hints: isProd ? 'warning' : false,
    maxEntrypointSize: 512000,
    maxAssetSize: 512000,
  },
  devServer: {
    port: 3000,
    hot: false,
    liveReload: false,
    client: false,
    historyApiFallback: true,
    static: {
      directory: path.join(__dirname, 'public'),
    },
    headers: {
      'Content-Security-Policy':
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ws: wss:;",
    },
  },
};
