const path = require('path');

module.exports = {
  mode: 'development',
  entry: {
    index: './src'
  },
  output: {
    filename: 'main.js',
    path: path.resolve(__dirname, 'dist')
  },
  resolve: {
    extensions: [".js", ".ts", ".tsx"],
  },
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true
  },
  devtool: "source-map",
  module: {
    rules: [
      // all files with a `.ts`, `.cts`, `.mts` or `.tsx` extension will be handled by `ts-loader`
      {
        test: /\.([cm]?ts|tsx)$/,
        options: {
          compilerOptions: {
            target: "es2020",
            module: "es2020",
            moduleResolution: "node",
            jsx: "react",
            strict: true,
            noEmit: false,
            esModuleInterop: true,
            isolatedModules: true,
            skipLibCheck: true,
            forceConsistentCasingInFileNames: true
          }
        },
        loader: "ts-loader"
      },
      {
        test: /\.wasm$/,
        type: "asset/inline",
      },
      {
        test: /\.s[ac]ss$/i,
        use: [
          "style-loader",
          "css-loader",
          "sass-loader",
        ],
      },
    ]
  },
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
    },
    client: {
      overlay: { errors: true, warnings: false }
    },
    compress: true,
    port: 9000
  }
}