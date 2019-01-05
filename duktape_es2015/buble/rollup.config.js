const resolve = require('rollup-plugin-node-resolve'),
    commonjs = require('rollup-plugin-commonjs'),
    babel = require('rollup-plugin-babel'),
    json = require('rollup-plugin-json'),
    builtins = require('rollup-plugin-node-builtins');
//typescript = require('rollup-plugin-typescript');


module.exports = [
    // browser-friendly UMD build
    {
        input: './index.js',
        output: [{
            file: "dist/cheerio.js",
            format: 'cjs',
            name: 'cheerio',
            preferBuiltins: false,
        }],
        plugins: [
            // typescript({
            //     exclude: ['node_modules/**'],
            //     typescript: require('typescript'),
            //     declaration: false,
            //     module: 'es2015'
            // }),
            builtins(),
            resolve({
                preferBuiltins: true
            }),
            commonjs(),

            json(),
            babel({
                //exclude: ['node_modules/**']
            }),
        ]
    }
];