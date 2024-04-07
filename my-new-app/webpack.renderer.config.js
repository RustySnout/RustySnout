const rules = require('./webpack.rules');

rules.push({
  test: /\.css$/,
  use: [{ loader: 'style-loader' }, { loader: 'css-loader' }],
});

module.exports = {
  // Put your normal webpack config below here
  resolve: {
    fallback: {
      path: require.resolve("path-browserify"),
      util: require.resolve("util/"),
    },
  },
  module: {
    rules,
  },
};
