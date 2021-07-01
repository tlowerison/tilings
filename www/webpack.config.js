const BundleAnalyzerPlugin = require("webpack-bundle-analyzer").BundleAnalyzerPlugin;
const CompressionPlugin = require("compression-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const TerserPlugin = require("terser-webpack-plugin");
const path = require("path");
const pkg = require("./package.json");
const reactRefresh = require("react-refresh/babel");
const webpack = require("webpack");

const isDevelopment = process.env.NODE_ENV !== "production";
const publicPath = "/tilings/";
const shouldProfile = process.env.PROFILE === "true";

module.exports = {
  devServer: {
    contentBase: path.join(__dirname, "dist"),
    compress: true,
    historyApiFallback: {
      index: path.join(publicPath, "index.html"),
      disableDotRule: true,
    },
    hot: true,
    port: pkg.port,
  },
  entry: "./src/index.tsx",
  experiments: {
    asyncWebAssembly: true,
  },
  output: {
    publicPath,
    path: path.join(__dirname, "dist", publicPath),
    filename: "[name].js",
    library: pkg.name,
    libraryTarget: "umd",
    umdNamedDefine: true,
  },
  mode: isDevelopment ? "development" : "production",
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: [
          "style-loader",
          "css-loader",
          "sass-loader",
        ],
      },
      {
        test: /\.[jt]sx?$/,
        exclude: /node_modules/,
        use: [
          {
            loader: "babel-loader",
            options: {
              plugins: [
                isDevelopment && reactRefresh,
              ].filter(Boolean),
            },
          },
        ],
      },
    ],
  },
  optimization: {
    minimizer: [
      new TerserPlugin(),
    ],
    splitChunks: {
     chunks: "all",
    },
  },
  plugins: [
    shouldProfile && new BundleAnalyzerPlugin(),
    new CompressionPlugin(),
    new HtmlWebpackPlugin({
      template: path.resolve("./public/index.html"),
    }),
    isDevelopment && new webpack.HotModuleReplacementPlugin(),
    isDevelopment && (() => {
      // will fail if imported in linux
      const ReactRefreshWebpackPlugin = require("@pmmmwh/react-refresh-webpack-plugin");
      return new ReactRefreshWebpackPlugin();
    })(),
  ].filter(Boolean),
  resolve: {
    alias: {
      "react": "preact/compat",
      "react-dom/test-utils": "preact/test-utils",
      "react-dom": "preact/compat",
    },
    extensions: ["*", ".js", ".jsx", ".ts", ".tsx"],
    modules: [path.resolve(__dirname, "src"), "node_modules"],
  },
  target: "web",
};
