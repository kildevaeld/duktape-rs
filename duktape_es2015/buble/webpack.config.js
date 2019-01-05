const Path = require('path');


module.exports = {
    entry: "./index.js",
    mode: 'none',
    output: {
        path: Path.join(__dirname, 'dist'),
        filename: "buble.js",
        libraryTarget: 'commonjs'
    },
    module: {
        rules: [{
            test: /\.m?js$/,
            //exclude: /(node_modules|bower_components)/,
            use: {
                loader: 'babel-loader',
                options: {
                    presets: ['@babel/preset-env']
                }
            }
        }]
    }

}